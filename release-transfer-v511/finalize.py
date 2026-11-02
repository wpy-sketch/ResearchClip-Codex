"""Finalize only the exact accepted v5.1.1 draft after full-byte verification."""
import concurrent.futures
import hashlib
import json
import subprocess
import tempfile
import time
import urllib.request
from pathlib import Path
from transfer import BASE, REPO, SOURCE, manifest, verify_file

RELEASE_ID = 431008505
INSTALLER_SHA = 'af39eeaf503f34d5be350fadb94ab3bbc07a0bb3ad434cb1d74b6256f6c6a80f'


def check_release(value, expected, draft):
    if value.get('id') != RELEASE_ID or value.get('draft') is not draft or value.get('prerelease') is not False or value.get('tag_name') != 'v5.1.1' or value.get('target_commitish') != SOURCE:
        raise ValueError('Release identity/state differs')
    current = {}
    for asset in value['assets']:
        name = asset['name']
        item = expected.get(name)
        if name in current or not item or asset['state'] != 'uploaded' or asset['size'] != item['size'] or asset.get('digest') != 'sha256:' + item['sha256']:
            raise ValueError('Asset metadata differs')
        current[name] = asset
    if set(current) != set(expected) or expected['ResearchClip_5.1.1_x64-setup.exe']['sha256'] != INSTALLER_SHA:
        raise ValueError('Accepted installer or complete asset set differs')
    return current


def gh(*args):
    result = subprocess.run(['gh', *args], capture_output=True, text=True, encoding='utf-8', timeout=300)
    if result.returncode:
        raise ValueError('GitHub operation failed; credentials/raw response not logged')
    return result.stdout


def api(path):
    return json.loads(gh('api', 'repos/' + REPO + path))


def public_hash(asset, item):
    url = asset['browser_download_url']
    if not url.startswith('https://github.com/' + REPO + '/releases/download/v5.1.1/'):
        raise ValueError('Public URL is outside the exact release')
    for attempt in range(3):
        try:
            digest = hashlib.sha256()
            size = 0
            # No Authorization header: this proves actual anonymous public bytes.
            request = urllib.request.Request(url, headers={'User-Agent':'ResearchClip-Anonymous-Public-Verification'})
            with urllib.request.urlopen(request, timeout=120) as response:
                if not response.geturl().startswith('https://'):
                    raise ValueError('Non-HTTPS redirect')
                while chunk := response.read(1024 * 1024):
                    size += len(chunk)
                    if size > item['size']:
                        raise ValueError('Public file exceeds accepted size')
                    digest.update(chunk)
            if size != item['size'] or digest.hexdigest() != item['sha256']:
                raise ValueError('Anonymous public bytes differ')
            return item['name']
        except OSError:
            if attempt == 2:
                raise
            time.sleep(3)


def main():
    expected = manifest(json.loads((BASE / 'manifest.json').read_text(encoding='utf-8')))
    if api('/commits/v5.1.1')['sha'] != SOURCE:
        raise ValueError('Immutable source tag differs')
    remote = check_release(api('/releases/' + str(RELEASE_ID)), expected, True)
    with tempfile.TemporaryDirectory(prefix='researchclip-v511-readback-') as tmp:
        def draft_one(index, name):
            folder = Path(tmp) / str(index)
            folder.mkdir()
            gh('release', 'download', 'v5.1.1', '--repo', REPO, '--pattern', name, '--dir', str(folder))
            verify_file(folder / name, expected[name])
            return name

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            futures = [pool.submit(draft_one, index, name) for index, name in enumerate(expected)]
            for count, future in enumerate(concurrent.futures.as_completed(futures), 1):
                print('DRAFT_FULL_HASH_PASS', count, '/34', future.result(), flush=True)
    fresh = check_release(api('/releases/' + str(RELEASE_ID)), expected, True)
    if any(fresh[name]['id'] != asset['id'] for name, asset in remote.items()):
        raise ValueError('Draft asset identity changed during readback')
    gh('release', 'edit', 'v5.1.1', '--repo', REPO, '--draft=false', '--latest')
    try:
        release = api('/releases/' + str(RELEASE_ID))
        published = check_release(release, expected, False)
        if any(published[name]['id'] != asset['id'] for name, asset in remote.items()):
            raise ValueError('Published asset identity differs')
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            futures = [pool.submit(public_hash, published[name], item) for name, item in expected.items()]
            for count, future in enumerate(concurrent.futures.as_completed(futures), 1):
                print('ANONYMOUS_FULL_HASH_PASS', count, '/34', future.result(), flush=True)
        if api('/releases/latest')['id'] != RELEASE_ID:
            raise ValueError('Latest release differs')
        report = {'result':'PUBLISHED_AND_PUBLIC_HASHES_VERIFIED','repository':'https://github.com/' + REPO,'version':'5.1.1','tag':'v5.1.1','releaseId':RELEASE_ID,'sourceCommit':SOURCE,'releaseUrl':release['html_url'],'installerUrl':published['ResearchClip_5.1.1_x64-setup.exe']['browser_download_url'],'assets':[{**item,'url':published[name]['browser_download_url']} for name, item in expected.items()],'verificationMode':'actual GitHub runner full draft readback and anonymous HTTPS SHA256 of all34 files; local32-file draft readback recorded separately','updater':'DISABLED_MANUAL_DOWNLOAD'}
        (BASE / 'publication-verified.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        print(report['result'], report['releaseUrl'], flush=True)
    except Exception:
        subprocess.run(['gh','release','edit','v5.1.1','--repo',REPO,'--draft=true','--latest=false'], capture_output=True, timeout=120)
        raise ValueError('Public verification failed; attempted return of only this release to draft') from None


if __name__ == '__main__':
    main()
