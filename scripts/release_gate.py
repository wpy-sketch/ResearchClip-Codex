"""Read-only release gate. Actual native QA and the chosen delivery policy are mandatory.

This tool never creates keys, tags or releases and never rewrites existing assets.
"""
import argparse, hashlib, json, os, re, subprocess
from pathlib import Path
from urllib.parse import urlsplit

ROOT=Path(__file__).resolve().parent.parent
REQUIRED_TESTS=('typecheck','eslint','vitest','rust','clippy','frontendBuild','tauriBuild','databaseMigration','markdownReopen','pdfExcerpt','zoteroConnection','zoteroAllTypes','extensionInstall','ocrInference','offlineOcr','signaturesAndHashes','windowsInstall','upgradeDataPreserved')
MANUAL_MODE='MANUAL_BUILTIN_CATALOG'
MANUAL_REQUIRED_TESTS=tuple(k for k in REQUIRED_TESTS if k!='signaturesAndHashes')+('componentHashes','manualUpdateMode')

def git(*args):
    return subprocess.check_output(['git',*args],cwd=ROOT,text=True,encoding='utf-8').strip()

def repository(origin):
    match=re.fullmatch(r'(?:https://github\.com/|git@github\.com:)([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+?)(?:\.git)?',origin)
    if not match: raise ValueError('origin must identify an unambiguous GitHub repository without embedded credentials')
    return '/'.join(match.groups())

def version():
    a=json.loads((ROOT/'package.json').read_text(encoding='utf-8'))['version']
    b=json.loads((ROOT/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))['version']
    c=re.search(r'^version = "([^"]+)"', (ROOT/'src-tauri/Cargo.toml').read_text(encoding='utf-8'),re.M)[1]
    if a!=b or b!=c or not re.fullmatch(r'\d+\.\d+\.\d+',a):raise ValueError('Application versions disagree or are not a stable SemVer release')
    return a

def application_digest():
    roots=['src','src-tauri','browser-extension','zotero-connector','package.json','package-lock.json','vite.config.ts','docs/USER_GUIDE.md','docs/images','docs/licenses']
    paths=subprocess.check_output(['git','ls-files','-c','-o','--exclude-standard','-z','--',*roots],cwd=ROOT).decode('utf-8').split('\0')
    h=hashlib.sha256()
    for name in sorted(set(filter(None,paths))):
        path=ROOT/name
        if path.is_file():h.update(name.encode()+b'\0'+hashlib.sha256(path.read_bytes()).digest())
    return h.hexdigest()

def trust_ready(value,repo):
    if value.get('schemaVersion')!=1:raise ValueError('Unknown trust configuration version')
    for key in ('catalogPublicKey','updaterPublicKey'):
        if not isinstance(value.get(key),str) or not value[key].strip():raise ValueError(f'{key} is UNPUBLISHED; do not create unsigned update metadata')
    for key,filename in (('catalogUrl','catalog.json'),('updaterUrl','latest.json')):
        p=urlsplit(value.get(key) or '')
        if p.scheme!='https' or p.hostname!='github.com' or p.username or p.password or p.port not in (None,443) or p.query or p.fragment or not p.path.startswith('/'+repo+'/releases/') or not p.path.endswith('/'+filename):
            raise ValueError(f'{key} must refer to the verified origin repository over HTTPS')

def validate_receipt(receipt,source_digest,installer_hash,xpi_hash,delivery_mode=None):
    if receipt.get('result')!='PASS' or receipt.get('applicationDigest')!=source_digest or receipt.get('installerSha256')!=installer_hash or receipt.get('zoteroXpiSha256')!=xpi_hash:
        raise ValueError('Actual acceptance receipt does not match these exact source/installer/XPI bytes')
    if receipt.get('mode')!='actual-independent-windows-and-zotero' or receipt.get('isolatedProfile') is not True or receipt.get('zoteroVersion')!='10.0.6':
        raise ValueError('Mock/browser-only checks cannot substitute for independent Windows and Zotero acceptance')
    required=MANUAL_REQUIRED_TESTS if delivery_mode==MANUAL_MODE else REQUIRED_TESTS
    if any(receipt.get('checks',{}).get(name) is not True for name in required):raise ValueError('Required release acceptance is incomplete')
    if receipt.get('unresolvedP0')!=0 or receipt.get('unresolvedP1')!=0:raise ValueError('P0/P1 issues block stable publication')

def preflight(assets,receipt_path):
    repo=repository(git('remote','get-url','origin'));v=version()
    if os.environ.get('GITHUB_REPOSITORY',repo).lower()!=repo.lower():raise ValueError('Workflow repository differs from origin')
    metadata=json.loads((assets/'release-build.json').read_text(encoding='utf-8'))
    trust=json.loads((ROOT/'src-tauri/resources/package-trust.json').read_text(encoding='utf-8'))
    if metadata.get('deliveryMode')==MANUAL_MODE:
        if any(trust.get(k) for k in ('catalogPublicKey','catalogUrl','updaterPublicKey','updaterUrl')):raise ValueError('Manual release cannot claim remote catalog/updater configuration')
    else:trust_ready(trust,repo)
    if git('status','--porcelain'):raise ValueError('Uncommitted files must be reviewed and committed before publication')
    if metadata['repository']!=repo or metadata['version']!=v or metadata['tag']!='v'+v or not re.fullmatch('[a-f0-9]{40}',metadata['sourceCommit']):raise ValueError('Build target/version/commit mismatch')
    if subprocess.run(['git','merge-base','--is-ancestor',metadata['sourceCommit'],'HEAD'],cwd=ROOT).returncode:raise ValueError('Built commit is not in the current verified history')
    changed=git('diff','--name-only',metadata['sourceCommit'],'HEAD').splitlines()
    if any(not name.startswith(('docs/qa/','docs/release/')) for name in changed):raise ValueError('Only QA/release documentation may follow the exact built commit')
    digest=application_digest()
    if metadata['applicationDigest']!=digest:raise ValueError('Application source changed after the build')
    for item in metadata['files']:
        if not re.fullmatch('[A-Za-z0-9_.-]{1,120}',item['name']) or Path(item['name']).name!=item['name']:raise ValueError('Asset path is not a flat filename')
        path=assets/item['name']
        if not path.is_file() or path.is_symlink() or path.stat().st_size!=item['size'] or hashlib.sha256(path.read_bytes()).hexdigest()!=item['sha256']:raise ValueError('Asset hash/size mismatch')
    validate_asset_links(assets,metadata)
    validate_receipt(json.loads(receipt_path.read_text(encoding='utf-8')),digest,metadata['installerSha256'],metadata['zoteroXpiSha256'],metadata.get('deliveryMode'))
    return metadata

def validate_asset_links(assets,metadata):
    if metadata.get('deliveryMode')==MANUAL_MODE:return validate_manual_asset_links(assets,metadata)
    v=metadata['version'];repo=metadata['repository'];tag=metadata['tag'];name=f'ResearchClip_{v}_x64-setup.exe'
    base='https://github.com/'+repo+'/releases/download/'+tag+'/'
    latest=json.loads((assets/'latest.json').read_text(encoding='utf-8'))
    if latest.get('version')!=v or set(latest.get('platforms',{}))!={'windows-x86_64'}:raise ValueError('Updater version/platform mismatch')
    update=latest['platforms']['windows-x86_64']
    if update.get('url')!=base+name or update.get('signature')!=(assets/(name+'.sig')).read_text(encoding='utf-8').strip():raise ValueError('Updater URL/signature does not match the real installer asset')
    envelope=json.loads((assets/'catalog.json').read_text(encoding='utf-8'));catalog=json.loads(envelope['payload']);files={p['name']:p for p in metadata['files']}
    ids=set();xpi=set()
    for package in catalog['packages']:
        identity=package['id']
        if identity not in ('model.ocr-line','plugin.zotero-connector') or identity in ids or package['appVersion']!='='+v:raise ValueError('Unverified, duplicate or incompatible optional component in production catalog')
        ids.add(identity)
        if json.loads((assets/(identity+'.manifest.json')).read_text(encoding='utf-8'))!=package:raise ValueError('Manifest differs from signed catalog payload')
        for item in package['files']:
            declared=files.get(item['path'])
            if not declared or item['url']!=base+item['path'] or item['sha256']!=declared['sha256'] or item['size']!=declared['size']:raise ValueError('Catalog URL/hash does not match a staged real asset')
            if identity=='plugin.zotero-connector':xpi.add(item['path'])
    if ids!={'model.ocr-line','plugin.zotero-connector'} or len(xpi)!=1:raise ValueError('Required real OCR/Connector assets are missing')
    update_index=json.loads((assets/'zotero-updates.json').read_text(encoding='utf-8'))
    plugin=next(p for p in catalog['packages'] if p['id']=='plugin.zotero-connector');filename=next(iter(xpi))
    expected={'addons':{'researchclip-connector@wpy-sketch.github.io':{'updates':[{'version':plugin['version'],'update_link':base+filename,'update_hash':'sha256:'+files[filename]['sha256'],'applications':{'zotero':{'strict_min_version':'10.0.6','strict_max_version':'10.0.6'}}}]}}}
    if update_index!=expected:raise ValueError('Zotero update metadata does not match accepted XPI bytes/version/URL')
    allowed={name,name+'.sig','rec.onnx','characters.txt','catalog.json','catalog.payload.json','catalog.payload.json.sig','catalog.minisig','model.ocr-line.manifest.json','plugin.zotero-connector.manifest.json','compatibility.json','latest.json','COMPONENT-SOURCES.txt','OCR-LICENSE-APACHE-2.0.txt','zotero-updates.json',*xpi}
    if set(files)-allowed:raise ValueError('Unexpected assets are not authorized release components')

def validate_manual_asset_links(assets,metadata):
    v=metadata['version'];repo=metadata['repository'];tag=metadata['tag'];exe=f'ResearchClip_{v}_x64-setup.exe'
    if tag!='v'+v:raise ValueError('Manual version/tag mismatch')
    if (assets/'catalog.json').read_bytes()!=(ROOT/'src-tauri/resources/package-builtin.json').read_bytes():raise ValueError('Release catalog differs from the compiled component definition')
    catalog=json.loads((assets/'catalog.json').read_text(encoding='utf-8'));files={p['name']:p for p in metadata['files']}
    packages={p['id']:p for p in catalog['packages']}
    if set(packages)!={'model.ocr-line','plugin.zotero-connector'}:raise ValueError('Unexpected manual components')
    allowed={exe,'catalog.json','delivery.json','rec.onnx','characters.txt','compatibility.json','zotero-updates.json','COMPONENT-SOURCES.txt','OCR-LICENSE-APACHE-2.0.txt'}
    for identity,p in packages.items():
        manifest=identity+'.manifest.json';allowed.add(manifest)
        if p['appVersion']!='='+v or json.loads((assets/manifest).read_text(encoding='utf-8'))!=p:raise ValueError('Manual component manifest/version mismatch')
        for a in p['files']:
            declared=files.get(a['path'])
            if not declared or declared['size']!=a['size'] or declared['sha256']!=a['sha256']:raise ValueError('Manual component hash mismatch')
            if identity=='plugin.zotero-connector':
                allowed.add(a['path'])
                if a['url']!=f'https://github.com/{repo}/releases/download/{tag}/{a["path"]}':raise ValueError('Connector URL differs from verified origin/tag')
    if set(files)!=allowed:raise ValueError('Missing or unauthorized manual release assets; updater signatures/latest.json must not be fabricated')
    delivery=json.loads((assets/'delivery.json').read_text(encoding='utf-8'))
    if delivery!={'schemaVersion':1,'mode':MANUAL_MODE,'updater':'DISABLED_MANUAL_DOWNLOAD','catalog':'EXACT_COMPILED_MANIFEST','signingKeysRequired':False}:raise ValueError('Manual delivery policy mismatch')
    from zotero_connector_release import update_manifest
    p=packages['plugin.zotero-connector'];a=p['files'][0]
    expected=update_manifest({'version':p['version'],'pluginId':'researchclip-connector@wpy-sketch.github.io','sha256':a['sha256']},repo,tag)
    if json.loads((assets/'zotero-updates.json').read_text(encoding='utf-8'))!=expected:raise ValueError('Connector update index differs from exact manual XPI')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--assets',type=Path);parser.add_argument('--receipt',type=Path);args=parser.parse_args()
    try:
        if args.assets:
            if not args.receipt:raise ValueError('A real acceptance receipt is required')
            result=preflight(args.assets.resolve(),args.receipt.resolve());print(json.dumps({'result':'PASS','version':result['version'],'repository':result['repository']}))
        else:print(json.dumps({'repository':repository(git('remote','get-url','origin')),'version':version(),'head':git('rev-parse','HEAD'),'applicationDigest':application_digest(),'dirty':bool(git('status','--porcelain')),'publication':'NOT_ATTEMPTED'}))
    except (ValueError,KeyError,OSError) as error:parser.exit(1,str(error)+'\n')
