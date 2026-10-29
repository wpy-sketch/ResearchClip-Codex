"""Prepare unpublished, reproducible package metadata. Never publishes or installs code.

Production keys are supplied by the release owner; development signing is explicitly
labelled and does not change the application's compiled trust roots.
"""
from pathlib import Path
import argparse, base64, hashlib, json, os, re, subprocess, time
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024*1024), b''):
            h.update(chunk)
    return h.hexdigest()

def https(url):
    p = urlsplit(url)
    host = p.hostname or ''
    assert p.scheme == 'https' and not p.username and not p.password and p.port in (None,443)
    assert host == 'github.com' or host.endswith('.githubusercontent.com') or host == 'huggingface.co' or host.endswith('.hf.co') or host == 'www.modelscope.cn'

def prepare(candidate, output, assets, sequence, key=None, development=False, installer=None):
    assert not output.exists(), 'Use a new output directory; existing files are retained.'
    assert sequence > 0
    packages = candidate['packages']
    assert candidate['status'] == 'UNPUBLISHED' and candidate['catalogPublicKey'] is None and candidate['catalogUrl'] is None
    names = set()
    for p in packages:
        assert re.fullmatch(r'[a-z0-9][a-z0-9.-]{0,47}', p['id']) and p['id'] not in names
        names.add(p['id'])
        assert p['license'] and p['publisher'] and p['platforms'] == ['windows-x86_64']
        https(p['source']); https(p['licenseUrl'])
        for asset in p['files']:
            assert re.fullmatch(r'[a-zA-Z0-9_.-]{1,100}', asset['path']) and asset['path'] not in ('.','..')
            https(asset['url'])
            source = assets / asset['path']
            assert source.is_file() and not source.is_symlink(), f'Missing prepared asset: {asset["path"]}'
            assert source.stat().st_size == asset['size'] and digest(source) == asset['sha256'], f'Asset hash mismatch: {asset["path"]}'
    output.mkdir(parents=True)
    (output/'packages').mkdir()
    payload = {'schemaVersion':1, 'sequence':sequence, 'expiresAt':int(time.time())+180*86400, 'packages':packages}
    body = json.dumps(payload,ensure_ascii=False,separators=(',',':'))
    file = output/'catalog.payload.json';file.write_text(body,encoding='utf-8')
    for p in packages:
        (output/'packages'/(p['id']+'.manifest.json')).write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    signed = False
    if key:
        assert key.is_file(), 'Provide an existing signing key path; never a private key value.'
        # No shell interpolation, download scripts, key generation or secret stdout.
        command=['node',str(ROOT/'scripts/with-rust.mjs'),'tauri','signer','sign','--private-key-path',str(key.resolve()),str(file.resolve())]
        result=subprocess.run(command,cwd=ROOT,env=os.environ.copy(),stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120)
        if result.returncode: raise RuntimeError('Catalog signing failed; no trusted output was claimed.')
        signature=base64.b64decode(Path(str(file)+'.sig').read_bytes(),validate=True).decode('utf-8')
        (output/'catalog.minisig').write_text(signature,encoding='utf-8')
        (output/'catalog.json').write_text(json.dumps({'payload':body,'signature':signature},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        signed=True
    compatibility=[{'id':p['id'],'version':p['version'],'appVersion':p['appVersion'],'platforms':p['platforms'],'adapter':p['adapter'],'dependencies':p['dependencies']} for p in packages]
    (output/'compatibility.json').write_text(json.dumps(compatibility,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    installer_info = None
    if installer:
        assert installer.is_file() and not installer.is_symlink()
        installer_info={'file':installer.name,'size':installer.stat().st_size,'sha256':digest(installer),'published':False,'updaterSignature':None}
    publication={'status':'UNPUBLISHED','developmentSignature':bool(key and development),'signed':signed,'catalogUrl':None,'updaterUrl':None,'installer':installer_info,'blockedComponents':candidate.get('blockedComponents',[]),'note':'Local preparation only. Test keys must never become production trust. Upstream assets exist; this catalog is not hosted.'}
    (output/'publication.json').write_text(json.dumps(publication,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    sums=[]
    for path in sorted(output.rglob('*')):
        if path.is_file(): sums.append(digest(path)+'  '+path.relative_to(output).as_posix())
    (output/'SHA256SUMS').write_text('\n'.join(sums)+'\n',encoding='ascii')
    return {'status':'UNPUBLISHED','signed':signed,'development':development,'packages':len(packages)}

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate',type=Path,default=ROOT/'docs/release/package-catalog.candidate.json')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--asset-dir',type=Path,required=True)
    parser.add_argument('--sequence',type=int,required=True)
    parser.add_argument('--signing-key',type=Path)
    parser.add_argument('--development',action='store_true')
    parser.add_argument('--installer',type=Path)
    args=parser.parse_args()
    print(json.dumps(prepare(json.loads(args.candidate.read_text(encoding='utf-8')),args.output,args.asset_dir,args.sequence,args.signing_key,args.development,args.installer)))
