"""One-release transport of exact accepted bytes. Never publishes or replaces assets."""
import hashlib
import json
import re
import subprocess
import tempfile
from pathlib import Path

BASE = Path(__file__).resolve().parent
REPO = 'wpy-sketch/ResearchClip-Codex'
SOURCE = '9c31486df8f99193f95fe309cae8a8b37af07c7a'


def manifest(value):
    if (value.get('repository'), value.get('releaseId'), value.get('tag'), value.get('sourceCommit')) != (REPO, 431008505, 'v5.1.1', SOURCE):
        raise ValueError('Unexpected release identity')
    expected = {}
    for item in value['expected']:
        name = item['name']
        if not re.fullmatch(r'[A-Za-z0-9_.-]{1,120}', name) or name in ('.', '..') or name in expected or type(item['size']) is not int or item['size'] <= 0 or not re.fullmatch('[a-f0-9]{64}', item['sha256']):
            raise ValueError('Unsafe or duplicate asset')
        expected[name] = item
    if len(expected) != 34 or len(set(value['payload'])) != len(value['payload']) or not set(value['payload']) <= set(expected):
        raise ValueError('Unexpected asset set')
    return expected


def remote_assets(release, expected):
    if release.get('id') != 431008505 or release.get('draft') is not True or release.get('prerelease') is not False or release.get('tag_name') != 'v5.1.1' or release.get('target_commitish') != SOURCE:
        raise ValueError('Only the exact existing draft is eligible')
    current = {}
    for asset in release['assets']:
        name = asset['name']
        item = expected.get(name)
        if not item or name in current or asset['state'] != 'uploaded' or asset['size'] != item['size'] or asset.get('digest') != 'sha256:' + item['sha256']:
            raise ValueError('Existing asset differs; nothing will be overwritten')
        current[name] = asset
    return current


def verify_file(path, item):
    if path.is_symlink() or not path.is_file() or path.stat().st_size != item['size'] or hashlib.sha256(path.read_bytes()).hexdigest() != item['sha256']:
        raise ValueError('Transport file differs from accepted bytes')


def assemble(name, value, destination):
    parts = value['parts'][name]
    if not 1 <= len(parts) <= 64:
        raise ValueError('Part count exceeded')
    with destination.open('xb') as output:
        for part in parts:
            if not re.fullmatch(r'[0-9]{2}-[0-9]{4}\.bin', part['file']) or type(part['size']) is not int or not 0 < part['size'] <= 524288:
                raise ValueError('Unsafe part')
            path = BASE / 'parts' / part['file']
            verify_file(path, part)
            output.write(path.read_bytes())


def gh(*args):
    result = subprocess.run(['gh', *args], capture_output=True, text=True, encoding='utf-8', timeout=300)
    if result.returncode:
        raise ValueError('GitHub operation failed; exact draft retained')
    return result.stdout


def main():
    value = json.loads((BASE / 'manifest.json').read_text(encoding='utf-8'))
    expected = manifest(value)
    if json.loads(gh('api', 'repos/' + REPO + '/commits/v5.1.1'))['sha'] != SOURCE:
        raise ValueError('Immutable source tag differs')

    def snapshot():
        return remote_assets(json.loads(gh('api', 'repos/' + REPO + '/releases/431008505')), expected)

    current = snapshot()
    with tempfile.TemporaryDirectory(prefix='researchclip-v511-exact-') as temporary:
        for name in value['payload']:
            path = Path(temporary) / name
            assemble(name, value, path)
            verify_file(path, expected[name])
            if name in current:
                print('EXISTING_EXACT', name, flush=True)
                continue
            gh('release', 'upload', 'v5.1.1', str(path), '--repo', REPO)
            current = snapshot()
            if name not in current:
                raise ValueError('Uploaded asset missing')
            print('EXACT_TRANSFER_VERIFIED', name, flush=True)
    if set(snapshot()) != set(expected):
        raise ValueError('Draft is still incomplete')
    print('ALL_34_EXACT_DRAFT_ASSETS_PRESENT; NOT_PUBLISHED', flush=True)


if __name__ == '__main__':
    main()
