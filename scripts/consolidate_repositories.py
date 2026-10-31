"""One-off, hash-pinned ResearchClip -> ResearchClip-Codex migration.

Historical files are data: never unpacked into the application or executed.
Only this target repository and this plan's owned releases may be written.
The old repository is deliberately never modified by this program.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import ssl
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

from release_asset_io import read_assets, verified, interrupted_upload

ROOT = Path(__file__).resolve().parent.parent
SOURCE = 'wpy-sketch/ResearchClip'
TARGET = 'wpy-sketch/ResearchClip-Codex'
API = 'https://api.github.com/repos/' + TARGET
MAX_FILE = 512 * 1024**2
SHARDS = 8
RESERVED = {'RESEARCHCLIP_HISTORY_INDEX.json', 'README_HISTORY.md'}


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def name_ok(value):
    return isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_.-]{1,200}', value) and value not in ('.', '..')


def ref_name_ok(value):
    return value == 'refs/heads/main' or isinstance(value, str) and re.fullmatch(r'refs/tags/(?:v[0-9]+(?:\.[0-9]+){2}(?:[-.a-zA-Z0-9]*)?|untagged-[a-f0-9]{20,40})', value)


def load_plan(path):
    plan = json.loads(path.read_text(encoding='utf-8'))
    assert plan['schemaVersion'] == 1 and plan['source'] == SOURCE and plan['target'] == TARGET
    assert plan['archiveTag'] == 'researchclip-history-20261030'
    assert re.fullmatch('[a-f0-9]{40}', plan['sourceMain'])
    assert all(ref_name_ok(r['name']) and re.fullmatch('[a-f0-9]{40}', r['sha']) for r in plan['sourceRefs'])
    unique = plan['uniqueAssets']
    assert 0 < len(unique) <= 1000 and len({a['sha256'] for a in unique}) == len(unique)
    for a in unique:
        assert re.fullmatch('[a-f0-9]{64}', a['sha256']) and 0 < a['size'] <= MAX_FILE
        assert name_ok(a['originalName']) and name_ok(a['targetName'])
        u = urllib.parse.urlsplit(a['sourceUrl'])
        assert u.scheme == 'https' and u.netloc == 'github.com' and not u.query and not u.fragment
        assert u.path.startswith('/' + SOURCE + '/releases/download/')
        if a['member'] is not None:
            assert name_ok(a['member'])
    assigned = list(plan['standalone']) + [a for v in plan['volumes'] for a in v['members']]
    assert sorted(a['sha256'] for a in assigned) == sorted(a['sha256'] for a in unique)
    for v in plan['volumes']:
        assert name_ok(v['name']) and 0 < sum(a['size'] + 1024 for a in v['members']) < MAX_FILE
        assert len({a['member'] for a in v['members']}) == len(v['members'])
        assert all(a['targetName'] == v['name'] for a in v['members'])
    names = [a['targetName'] for a in plan['standalone']] + [v['name'] for v in plan['volumes']]
    assert len(names) == len(set(names)) and not set(names) & RESERVED
    known = {a['sha256']: a['size'] for a in unique}
    for r in plan['releases']:
        assert re.fullmatch(r'v[0-9]+(?:\.[0-9]+){2}(?:[-.a-zA-Z0-9]*)?', r['tag'])
        assert all(known[a['sha256']] == a['size'] and name_ok(a['name']) for a in r['assets'])
    identity = hashlib.sha256(json.dumps(plan, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()).hexdigest()
    return plan, identity


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


class GitHub:
    def __init__(self):
        assert os.environ.get('GITHUB_REPOSITORY', TARGET) == TARGET
        self.token = os.environ['GH_TOKEN']
        self.opener = urllib.request.build_opener(urllib.request.HTTPSHandler(context=ssl.create_default_context()), NoRedirect())

    def request(self, url, method='GET', data=None, binary=None):
        u = urllib.parse.urlsplit(url)
        assert u.scheme == 'https' and u.hostname in ('api.github.com', 'uploads.github.com')
        assert u.path.startswith('/repos/' + TARGET + '/') or method == 'GET' and u.path.startswith('/repos/' + SOURCE + '/')
        assert not (data is not None and binary is not None)
        body = binary if binary is not None else json.dumps(data).encode() if data is not None else None
        headers = {'Authorization': 'Bearer ' + self.token, 'Accept': 'application/vnd.github+json', 'User-Agent': 'ResearchClip-Consolidation', 'X-GitHub-Api-Version': '2022-11-28'}
        if body is not None:
            headers['Content-Type'] = 'application/octet-stream' if binary is not None else 'application/json'
        for attempt in range(4):
            try:
                with self.opener.open(urllib.request.Request(url, data=body, method=method, headers=headers), timeout=300) as response:
                    result = response.read()
                    return json.loads(result) if result else None
            except urllib.error.HTTPError as e:
                if method == 'GET' and e.code in (429, 500, 502, 503, 504) and attempt < 3:
                    time.sleep(min(60, max(2, int(e.headers.get('Retry-After', '3')))))
                    continue
                raise RuntimeError('GitHub ' + method + ' HTTP ' + str(e.code)) from None
            except OSError:
                if method != 'GET' or attempt == 3:
                    raise RuntimeError('GitHub connection failed; preserve existing releases and retry safely') from None
                time.sleep(3)

    def release(self, tag):
        for page in range(1, 5):
            rows = self.request(API + '/releases?per_page=100&page=' + str(page))
            found = next((r for r in rows if r['tag_name'] == tag), None)
            if found:
                return found
            if len(rows) < 100:
                return None
        raise ValueError('Release enumeration exceeded migration budget')


def owned_release(client, plan, identity):
    r = client.release(plan['archiveTag'])
    assert r and r['name'] == 'ResearchClip 历史附件去重归档' and 'Plan-SHA256: ' + identity in r['body']
    return r


def remote_items(client, release):
    return {a['name']: a for a in read_assets(client.request, API, release['id'])}


def public_download(url, destination, size, sha):
    u = urllib.parse.urlsplit(url)
    assert u.scheme == 'https' and u.netloc == 'github.com'
    assert u.path.startswith('/' + SOURCE + '/releases/download/') or u.path.startswith('/' + TARGET + '/releases/download/')
    if destination.exists():
        assert destination.stat().st_size == size and digest(destination) == sha
        return
    for attempt in range(3):
        stage = destination.with_name(destination.name + '.attempt-' + str(attempt))
        h = hashlib.sha256()
        total = 0
        try:
            # No token or user cookies are ever sent to a release CDN.
            with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'ResearchClip-History-Public-Verify'}), timeout=180) as response, stage.open('xb') as output:
                assert response.geturl().startswith('https://')
                while chunk := response.read(1024 * 1024):
                    total += len(chunk)
                    assert total <= size
                    h.update(chunk)
                    output.write(chunk)
            assert total == size and h.hexdigest() == sha, 'Historical source/download digest mismatch'
            stage.rename(destination)
            return
        except OSError:
            if attempt == 2:
                raise
            time.sleep(3)


def volume_check(path, volume):
    assert path.stat().st_size <= MAX_FILE
    expected = {a['member']: a for a in volume['members']}
    with zipfile.ZipFile(path) as z:
        assert len(z.infolist()) == len(expected) and set(z.namelist()) == set(expected)
        for info in z.infolist():
            a = expected[info.filename]
            assert name_ok(info.filename) and info.compress_type == zipfile.ZIP_STORED and not info.flag_bits & 1
            assert info.file_size == a['size'] and not info.is_dir()
            h = hashlib.sha256()
            total = 0
            with z.open(info) as stream:
                while chunk := stream.read(1024 * 1024):
                    total += len(chunk)
                    assert total <= a['size']
                    h.update(chunk)
            assert total == a['size'] and h.hexdigest() == a['sha256']


def make_volume(volume, output, cache):
    with zipfile.ZipFile(output, 'x', compression=zipfile.ZIP_STORED) as z:
        for a in volume['members']:
            path = cache / a['sha256']
            public_download(a['sourceUrl'], path, a['size'], a['sha256'])
            info = zipfile.ZipInfo(a['member'], date_time=(2026, 10, 30, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            with path.open('rb') as source, z.open(info, 'w') as target:
                shutil.copyfileobj(source, target, 1024 * 1024)
            path.unlink()  # Only this job's hash-verified scratch copy.
    volume_check(output, volume)


def upload(client, release, path, existing):
    expected = {'name': path.name, 'size': path.stat().st_size, 'sha256': digest(path)}
    assert name_ok(path.name) and 0 < expected['size'] <= MAX_FILE
    if path.name in existing:
        return verified(expected, existing[path.name])
    assert release['draft'], 'Never change an already public archive'
    for attempt in range(3):
        try:
            url = 'https://uploads.github.com/repos/' + TARGET + '/releases/' + str(release['id']) + '/assets?name=' + urllib.parse.quote(path.name)
            return verified(expected, client.request(url, 'POST', binary=path.read_bytes()))
        except (RuntimeError, AssertionError):
            actual = remote_items(client, release).get(path.name)
            if actual and actual['state'] == 'uploaded':
                return verified(expected, actual)
            if actual and interrupted_upload(expected, actual, release['draft']):
                client.request(API + '/releases/assets/' + str(actual['id']), 'DELETE')
            elif actual:
                raise ValueError('Existing remote asset differs; refusing replacement') from None
            if attempt == 2:
                raise
            time.sleep(5)


def tasks(plan):
    return [('file', a) for a in plan['standalone']] + [('zip', v) for v in plan['volumes']]


def prepare(client, plan, identity):
    old = client.release(plan['archiveTag'])
    if old:
        owned_release(client, plan, identity)
        return
    head = os.environ.get('GITHUB_SHA') or subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    assert re.fullmatch('[a-f0-9]{40}', head)
    client.request(API + '/releases', 'POST', {'tag_name': plan['archiveTag'], 'target_commitish': head, 'name': 'ResearchClip 历史附件去重归档', 'draft': True, 'prerelease': True, 'make_latest': 'false', 'body': '历史数据归档，不是应用安装版本。\n\nPlan-SHA256: ' + identity + '\n原仓库保留，全部字节核验完成后再归档。'})


def refs(plan):
    assert os.environ.get('GITHUB_REPOSITORY') == TARGET, 'Reference import runs only in the controlled target workflow'
    subprocess.run(['git', 'fetch', '--no-tags', 'https://github.com/' + SOURCE + '.git', 'refs/heads/main:refs/consolidation-source/main', 'refs/tags/*:refs/consolidation-source/tags/*'], cwd=ROOT, check=True)
    subprocess.run(['git', 'merge-base', '--is-ancestor', plan['sourceMain'], 'HEAD'], cwd=ROOT, check=True)
    raw = subprocess.check_output(['git', 'ls-remote', '--refs', 'origin'], cwd=ROOT, text=True)
    target = {line.split()[1]: line.split()[0] for line in raw.splitlines()}
    push = []
    for ref in plan['sourceRefs']:
        name = ref['name']
        assert ref_name_ok(name)
        local = 'refs/consolidation-source/main' if name == 'refs/heads/main' else 'refs/consolidation-source/tags/' + name.removeprefix('refs/tags/')
        assert subprocess.check_output(['git', 'rev-parse', local], cwd=ROOT, text=True).strip() == ref['sha']
        dest = 'refs/heads/archive/researchclip-main-20261030' if name == 'refs/heads/main' else name
        if dest in target:
            assert target[dest] == ref['sha'], 'Reference collision; no force push permitted'
        else:
            push.append(local + ':' + dest)
    if push:
        # Checkout's repository-scoped GITHUB_TOKEN prevents replay of old workflows.
        subprocess.run(['git', 'push', 'origin', *push], cwd=ROOT, check=True)


def copy_shard(client, plan, identity, shard, output):
    r = owned_release(client, plan, identity)
    existing = remote_items(client, r)
    allowed = {a['targetName'] for a in plan['uniqueAssets']} | RESERVED
    assert set(existing) <= allowed
    done = []
    for index, (kind, item) in enumerate(tasks(plan)):
        if index % SHARDS != shard:
            continue
        name = item['targetName'] if kind == 'file' else item['name']
        if name in existing and kind == 'zip':
            a = existing[name]
            assert a['state'] == 'uploaded' and a.get('digest') and a['size'] <= MAX_FILE
        else:
            with tempfile.TemporaryDirectory(prefix='researchclip-history-copy-') as temp:
                cache = Path(temp)
                path = cache / name
                if kind == 'file':
                    public_download(item['sourceUrl'], path, item['size'], item['sha256'])
                else:
                    make_volume(item, path, cache)
                a = upload(client, r, path, existing)
        done.append({'name': name, 'size': a['size'], 'sha256': a['digest'][7:]})
        print('Copied and hashed', name, flush=True)
    output.write_text(json.dumps({'plan': identity, 'shard': shard, 'phase': 'UPLOADED_PENDING_PUBLIC_VERIFICATION', 'assets': done}, indent=2), encoding='utf-8')


def publish_archive(client, plan, identity):
    r = owned_release(client, plan, identity)
    items = remote_items(client, r)
    required = {a['targetName'] for a in plan['uniqueAssets']}
    assert required <= set(items) <= required | RESERVED
    for a in plan['standalone']:
        verified({'name': a['targetName'], 'size': a['size'], 'sha256': a['sha256']}, items[a['targetName']])
    for v in plan['volumes']:
        a = items[v['name']]
        assert a['state'] == 'uploaded' and 0 < a['size'] <= MAX_FILE and re.fullmatch('sha256:[a-f0-9]{64}', a.get('digest', ''))
    index = {**plan, 'status': 'CONTENT_HASH_MAP', 'planSha256': identity, 'archiveAssets': [{'name': a['name'], 'size': a['size'], 'sha256': a['digest'][7:]} for name, a in items.items() if name not in RESERVED]}
    text = '# ResearchClip历史发布附件\n\n47个原版本的附件按SHA-256去重保存。EXE/XPI可直接下载，其余文件位于互相独立的ZIP卷中；各卷无需拼接。索引中的member是卷内文件名。历史分卷压缩包的原始分片保留原字节，使用时先按索引提取。所有历史文件仅作归档，不是本版组件安装清单。\n\n完整映射见RESEARCHCLIP_HISTORY_INDEX.json。原始文件名、版本、日期、源URL、大小和SHA均保留。当前安装版请使用仓库标记的Latest Release。\n\nPlan-SHA256: ' + identity + '\n'
    with tempfile.TemporaryDirectory(prefix='researchclip-history-index-') as temp:
        root = Path(temp)
        for name, data in [('RESEARCHCLIP_HISTORY_INDEX.json', json.dumps(index, ensure_ascii=False, indent=2)), ('README_HISTORY.md', text)]:
            p = root / name
            p.write_text(data, encoding='utf-8', newline='\n')
            upload(client, r, p, items)
    if r['draft']:
        client.request(API + '/releases/' + str(r['id']), 'PATCH', {'draft': False, 'prerelease': True, 'make_latest': 'false'})


def verify_shard(client, plan, identity, shard, output):
    r = owned_release(client, plan, identity)
    assert not r['draft']
    items = remote_items(client, r)
    assert set(items) == {a['targetName'] for a in plan['uniqueAssets']} | RESERVED
    if shard == 0:
        for name in sorted(RESERVED):
            a = items[name]
            with tempfile.TemporaryDirectory(prefix='researchclip-history-map-') as temp:
                p = Path(temp) / name
                public_download(a['browser_download_url'], p, a['size'], a['digest'][7:])
                if name.endswith('.json'):
                    index = json.loads(p.read_text(encoding='utf-8'))
                    assert index['planSha256'] == identity
                    assert index['uniqueAssets'] == plan['uniqueAssets'] and index['releases'] == plan['releases']
    result = []
    for index, (kind, item) in enumerate(tasks(plan)):
        if index % SHARDS != shard:
            continue
        name = item['targetName'] if kind == 'file' else item['name']
        a = items[name]
        assert a['state'] == 'uploaded' and re.fullmatch('sha256:[a-f0-9]{64}', a['digest'])
        with tempfile.TemporaryDirectory(prefix='researchclip-history-anonymous-') as temp:
            p = Path(temp) / name
            public_download(a['browser_download_url'], p, a['size'], a['digest'][7:])
            if kind == 'zip':
                volume_check(p, item)
                hashes = [m['sha256'] for m in item['members']]
            else:
                assert digest(p) == item['sha256']
                hashes = [item['sha256']]
        result.extend(hashes)
        print('Anonymous bytes and contents verified', name, flush=True)
    output.write_text(json.dumps({'result': 'PASS', 'plan': identity, 'shard': shard, 'sourceHashes': result}, indent=2), encoding='utf-8')


def finalize(client, plan, identity, receipts, output):
    proofs = [json.loads(p.read_text()) for p in receipts.rglob('verify-*.json')]
    assert len(proofs) == SHARDS and {p['shard'] for p in proofs} == set(range(SHARDS))
    assert all(p['result'] == 'PASS' and p['plan'] == identity for p in proofs)
    hashes = [h for p in proofs for h in p['sourceHashes']]
    assert sorted(hashes) == sorted(a['sha256'] for a in plan['uniqueAssets'])
    archive = owned_release(client, plan, identity)
    assert not archive['draft']
    indexed = {a['sha256']: a for a in plan['uniqueAssets']}
    base = 'https://github.com/' + TARGET + '/releases/download/' + plan['archiveTag'] + '/'
    migrated = []
    for row in plan['releases']:
        ref = client.request(API + '/commits/' + urllib.parse.quote(row['tag'], safe=''))
        # Identical Git object IDs preserve the original tag and its resolved commit.
        # Verify against the audited source reference rather than refetching mutable source API state.
        pinned = next(r['sha'] for r in plan['sourceRefs'] if r['name'] == 'refs/tags/' + row['tag'])
        target_ref = client.request(API + '/git/ref/tags/' + urllib.parse.quote(row['tag'], safe=''))
        assert target_ref['object']['sha'] == pinned
        table = '\n'.join('| [' + a['name'] + '](' + base + urllib.parse.quote(indexed[a['sha256']]['targetName']) + ') | ' + (indexed[a['sha256']]['member'] or '直接下载') + ' | ' + a['sha256'] + ' |' for a in row['assets'])
        intro = '历史版本，原发布日期：' + (row['publishedAt'] or '未提供') + '。附件已迁入本仓库去重归档；ZIP卷内文件名及摘要见下表。当前安装版请使用Latest Release。\n\n原发布页：' + row['sourceUrl'] + '\n\n'
        body = intro + row['body'] + '\n\n## 本仓库历史附件目录\n\n| 原文件名 / 下载 | ZIP内文件名 | 原SHA-256 |\n| --- | --- | --- |\n' + table + '\n\nPlan-SHA256: ' + identity
        if len(body) > 120000:
            body = intro + row['body'] + '\n\n本版全部附件映射：' + base + 'RESEARCHCLIP_HISTORY_INDEX.json\n\nPlan-SHA256: ' + identity
        assert len(body) <= 120000
        existing = client.release(row['tag'])
        if existing:
            assert existing['body'] == body and not existing['draft'], 'Existing release differs; no overwrite permitted'
            created = existing
        else:
            created = client.request(API + '/releases', 'POST', {'tag_name': row['tag'], 'target_commitish': ref['sha'], 'name': row['name'] or row['tag'], 'body': body, 'draft': False, 'prerelease': row['prerelease'], 'make_latest': 'false'})
        migrated.append({'tag': row['tag'], 'url': created['html_url'], 'sourceAssets': len(row['assets'])})
        print('Historical release mapped', row['tag'], flush=True)
    output.write_text(json.dumps({'result': 'PASS', 'plan': identity, 'source': SOURCE, 'target': TARGET, 'archiveUrl': archive['html_url'], 'uniqueAssets': len(hashes), 'uniqueBytes': sum(a['size'] for a in plan['uniqueAssets']), 'originalReferences': sum(len(r['assets']) for r in plan['releases']), 'releases': migrated, 'oldRepositoryArchived': False}, indent=2), encoding='utf-8')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('mode', choices=['audit', 'prepare', 'refs', 'copy', 'publish-archive', 'verify', 'finalize'])
    p.add_argument('--plan', type=Path, default=ROOT / 'docs/release/researchclip-consolidation.json')
    p.add_argument('--shard', type=int, default=0)
    p.add_argument('--output', type=Path, default=Path('consolidation-result.json'))
    p.add_argument('--receipts', type=Path, default=Path('receipts'))
    a = p.parse_args()
    plan, identity = load_plan(a.plan)
    assert 0 <= a.shard < SHARDS
    if a.mode == 'audit':
        print(json.dumps({'result': 'PASS', 'plan': identity, 'uniqueAssets': len(plan['uniqueAssets']), 'volumes': len(plan['volumes']), 'standalone': len(plan['standalone'])}))
        return
    if a.mode == 'refs':
        refs(plan)
        return
    client = GitHub()
    if a.mode == 'prepare':
        prepare(client, plan, identity)
    elif a.mode == 'copy':
        copy_shard(client, plan, identity, a.shard, a.output)
    elif a.mode == 'publish-archive':
        publish_archive(client, plan, identity)
    elif a.mode == 'verify':
        verify_shard(client, plan, identity, a.shard, a.output)
    else:
        finalize(client, plan, identity, a.receipts, a.output)


if __name__ == '__main__':
    main()
