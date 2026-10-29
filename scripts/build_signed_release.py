"""Build signed Windows assets from configured production trust. Never publishes.

Private keys come only from CI/environment. Missing keys abort before metadata is made.
"""
import argparse,base64,copy,datetime,hashlib,importlib.util,json,os,shutil,subprocess,tempfile,urllib.request
from pathlib import Path
from urllib.parse import quote
from zotero_connector_release import update_manifest
from release_gate import ROOT,application_digest,git,repository,trust_ready,version
_spec=importlib.util.spec_from_file_location('researchclip_package_catalog',ROOT/'scripts/prepare-package-catalog.py')
_catalog=importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_catalog)
https=_catalog.https

def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
    return h.hexdigest()

def download(asset,destination):
    https(asset['url']);size=0;h=hashlib.sha256()
    with urllib.request.urlopen(asset['url'],timeout=90) as response,destination.open('xb') as target:
        https(response.geturl())
        for chunk in iter(lambda:response.read(65536),b''):
            size+=len(chunk)
            if size>asset['size']:raise ValueError('Upstream model exceeds reviewed size')
            h.update(chunk);target.write(chunk)
    if size!=asset['size'] or h.hexdigest()!=asset['sha256']:raise ValueError('Upstream model hash/size mismatch')

def run(args,env=None):
    subprocess.run(args,cwd=ROOT,env=env,check=True)

def build(output):
    repo=repository(git('remote','get-url','origin'));v=version();tag='v'+v
    trust=json.loads((ROOT/'src-tauri/resources/package-trust.json').read_text(encoding='utf-8'));trust_ready(trust,repo)
    for key in ('TAURI_SIGNING_PRIVATE_KEY','CATALOG_SIGNING_PRIVATE_KEY'):
        if not os.environ.get(key):raise ValueError(key+' is missing; production signing is NOT_READY')
    if output.exists():raise ValueError('Release staging already exists; refusing overwrite')
    output.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix='researchclip-release-') as temp:
        override=Path(temp)/'updater-build.json';override.write_text(json.dumps({'bundle':{'createUpdaterArtifacts':True},'plugins':{'updater':{'pubkey':trust['updaterPublicKey'],'endpoints':[trust['updaterUrl']],'requireSignedVersion':True}}}),encoding='utf-8')
        run(['node',str(ROOT/'scripts/with-rust.mjs'),'tauri','build','--bundles','nsis','--config',str(override)])
    name=f'ResearchClip_{v}_x64-setup.exe';installer=ROOT/'src-tauri/target/release/bundle/nsis'/name
    for file in (installer,Path(str(installer)+'.sig')):
        if not file.is_file():raise ValueError('Official installer or updater signature was not produced')
        shutil.copyfile(file,output/file.name)
    run(['python',str(ROOT/'scripts/build-zotero-connector.py')])
    xm=json.loads((ROOT/'artifacts/zotero-connector-build/latest.json').read_text(encoding='utf-8'))
    xpi=ROOT/xm['file']
    if digest(xpi)!=xm['sha256'] or xpi.stat().st_size!=xm['size']:raise ValueError('XPI build metadata mismatch')
    shutil.copyfile(xpi,output/xpi.name)
    (output/'zotero-updates.json').write_text(json.dumps(update_manifest(xm,repo,tag),indent=2)+'\n',encoding='utf-8')
    candidate=json.loads((ROOT/'docs/release/package-catalog.candidate.json').read_text(encoding='utf-8'))
    ocr=copy.deepcopy(next(p for p in candidate['packages'] if p['id']=='model.ocr-line'))
    ocr['licenseUrl']='https://huggingface.co/SWHL/RapidOCR/blob/1cfba2e90fc938db55889873735088de210cc173/README.md'
    base=f'https://github.com/{repo}/releases/download/{tag}/'
    for asset in ocr['files']:
        download(asset,output/asset['path']);asset['url']=base+quote(asset['path'],safe='')
    connector={'id':'plugin.zotero-connector','name':'ResearchClip Zotero Connector','description':'Zotero10.0.6专用插件；下载后必须按Zotero机制安装，不等于宿主已安装。','version':xm['version'],'publisher':'ResearchClip contributors','license':'MIT','licenseUrl':f'https://github.com/{repo}/blob/{tag}/zotero-connector/LICENSE','source':f'https://github.com/{repo}','kind':'EXTERNAL_PLUGIN','appVersion':'='+v,'platforms':['windows-x86_64'],'dependencies':[],'adapter':'external','files':[{'path':xpi.name,'url':base+quote(xpi.name,safe=''),'size':xm['size'],'sha256':xm['sha256'],'entries':[]}]}
    shutil.copyfile(ROOT/'docs/licenses/PaddleOCR-Apache-2.0.txt',output/'OCR-LICENSE-APACHE-2.0.txt')
    packages=[ocr,connector]
    # Only reviewed real components. Formula/semantic weights and undeclared-license
    # browser archives never become installable by being copied from an older release.
    payload=json.dumps({'schemaVersion':1,'sequence':int(datetime.datetime.now(datetime.timezone.utc).timestamp()),'expiresAt':int(datetime.datetime.now(datetime.timezone.utc).timestamp())+180*86400,'packages':packages},ensure_ascii=False,separators=(',',':'))
    (output/'catalog.payload.json').write_text(payload,encoding='utf-8')
    with tempfile.TemporaryDirectory(prefix='researchclip-catalog-sign-') as temp:
        key=Path(temp)/'catalog.key';key.write_text(os.environ['CATALOG_SIGNING_PRIVATE_KEY'],encoding='utf-8')
        env={**os.environ,'TAURI_SIGNING_PRIVATE_KEY_PASSWORD':os.environ.get('CATALOG_SIGNING_PRIVATE_KEY_PASSWORD','')}
        env.pop('TAURI_SIGNING_PRIVATE_KEY',None)
        result=subprocess.run(['node',str(ROOT/'scripts/with-rust.mjs'),'tauri','signer','sign','--private-key-path',str(key),str(output/'catalog.payload.json')],cwd=ROOT,env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        if result.returncode:raise ValueError('Catalog signing failed; no private key or signer output logged')
    encoded=(output/'catalog.payload.json.sig').read_text(encoding='utf-8').strip();signature=base64.b64decode(encoded,validate=True).decode('utf-8')
    (output/'catalog.json').write_text(json.dumps({'payload':payload,'signature':signature},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    (output/'catalog.minisig').write_text(signature,encoding='utf-8')
    for p in packages:(output/(p['id']+'.manifest.json')).write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    (output/'compatibility.json').write_text(json.dumps([{'id':p['id'],'appVersion':p['appVersion'],'platforms':p['platforms']} for p in packages],ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    latest={'version':v,'notes':'See RELEASE_NOTES.md and the verified release report.','pub_date':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platforms':{'windows-x86_64':{'signature':(output/(name+'.sig')).read_text(encoding='utf-8').strip(),'url':base+quote(name,safe='')}}}
    (output/'latest.json').write_text(json.dumps(latest,indent=2)+'\n',encoding='utf-8')
    (output/'COMPONENT-SOURCES.txt').write_text('PP-OCRv4: SWHL/RapidOCR fixed revision 1cfba2e90fc938db55889873735088de210cc173, model card Apache-2.0. Dictionary: PaddleOCR v2.7.3 Apache-2.0.\nZotero Connector: original ResearchClip MIT code; target10.0.6.\nFormula/semantic model distribution unverified: excluded. Browser extension project license undeclared: source guide only, excluded.\nONNX Runtime and dependency licenses are bundled with the main program.\n',encoding='utf-8')
    run(['node',str(ROOT/'scripts/with-rust.mjs'),'cargo','run','--manifest-path','src-tauri/Cargo.toml','--locked','--example','verify_release_signatures','--',str(output)])
    files=[{'name':p.name,'size':p.stat().st_size,'sha256':digest(p)} for p in sorted(output.iterdir()) if p.is_file()]
    (output/'SHA256SUMS').write_text(''.join(f'{p["sha256"]}  {p["name"]}\n' for p in files),encoding='ascii')
    metadata={'status':'SIGNED_STAGED_NOT_PUBLISHED','repository':repo,'version':v,'tag':tag,'sourceCommit':git('rev-parse','HEAD'),'applicationDigest':application_digest(),'installerSha256':digest(output/name),'zoteroXpiSha256':xm['sha256'],'signatureVerification':'PASS','files':files}
    (output/'release-build.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':metadata['status'],'repository':repo,'version':v,'assets':len(files)}))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    try:build(args.output.resolve())
    except (ValueError,OSError,subprocess.CalledProcessError) as error:parser.exit(1,str(error)+'\n')
