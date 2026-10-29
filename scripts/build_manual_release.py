"""Manual release: compiled component definitions and SHA-256, no signing keys/updater."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
from release_gate import ROOT,application_digest,git,repository,version
from zotero_connector_release import update_manifest

MODE='MANUAL_BUILTIN_CATALOG'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def build(output,cache=None):
    repo=repository(git('remote','get-url','origin'));v=version();tag='v'+v
    trust=json.loads((ROOT/'src-tauri/resources/package-trust.json').read_text(encoding='utf-8'))
    if any(trust.get(k) for k in ('catalogPublicKey','catalogUrl','updaterPublicKey','updaterUrl')):
        raise ValueError('Manual mode must not claim configured remote catalogs or automatic updates')
    if output.exists():raise ValueError('Use a new staging path; existing assets are preserved')
    subprocess.run(['python',str(ROOT/'scripts/build-zotero-connector.py')],cwd=ROOT,check=True)
    x=json.loads((ROOT/'artifacts/zotero-connector-build/latest.json').read_text(encoding='utf-8'))
    catalog_path=ROOT/'src-tauri/resources/package-builtin.json'
    catalog=json.loads(catalog_path.read_text(encoding='utf-8'));packages={p['id']:p for p in catalog['packages']}
    if set(packages)!={'model.ocr-line','plugin.zotero-connector'}:raise ValueError('Only reviewed real resources can ship')
    connector=packages['plugin.zotero-connector'];asset=connector['files'][0]
    if connector['version']!=x['version'] or asset['sha256']!=x['sha256'] or asset['size']!=x['size'] or asset['url']!=f'https://github.com/{repo}/releases/download/{tag}/{Path(x["file"]).name}':raise ValueError('Compiled connector differs from exact current XPI/origin')
    if any(p['appVersion']!='='+v for p in packages.values()):raise ValueError('Compiled compatibility mismatch')
    subprocess.run(['node',str(ROOT/'scripts/with-rust.mjs'),'tauri','build','--bundles','nsis'],cwd=ROOT,check=True)
    installer=ROOT/f'src-tauri/target/release/bundle/nsis/ResearchClip_{v}_x64-setup.exe'
    output.mkdir(parents=True);shutil.copyfile(installer,output/installer.name);shutil.copyfile(ROOT/x['file'],output/asset['path'])
    for asset in packages['model.ocr-line']['files']:
        dest=output/asset['path'];source=cache/asset['path'] if cache else None
        if source:
            if source.stat().st_size!=asset['size'] or sha(source)!=asset['sha256']:raise ValueError('Real cached model mismatch')
            shutil.copyfile(source,dest)
        else:
            from build_signed_release import download
            download(asset,dest)
    shutil.copyfile(catalog_path,output/'catalog.json');shutil.copyfile(ROOT/'docs/licenses/PaddleOCR-Apache-2.0.txt',output/'OCR-LICENSE-APACHE-2.0.txt')
    for p in packages.values():(output/(p['id']+'.manifest.json')).write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    (output/'compatibility.json').write_text(json.dumps([{'id':p['id'],'appVersion':p['appVersion'],'platforms':p['platforms']} for p in packages.values()],indent=2)+'\n',encoding='utf-8')
    (output/'zotero-updates.json').write_text(json.dumps(update_manifest(x,repo,tag),indent=2)+'\n',encoding='utf-8')
    (output/'delivery.json').write_text(json.dumps({'schemaVersion':1,'mode':MODE,'updater':'DISABLED_MANUAL_DOWNLOAD','catalog':'EXACT_COMPILED_MANIFEST','signingKeysRequired':False},indent=2)+'\n',encoding='utf-8')
    (output/'COMPONENT-SOURCES.txt').write_text('Manual installation release; no Catalog or Updater signing keys. catalog.json is informational; clients use exact compiled definitions, HTTPS and SHA-256.\nPP-OCRv4: SWHL/RapidOCR revision1cfba2e90fc938db55889873735088de210cc173, Apache-2.0. Dictionary: PaddleOCRv2.7.3, Apache-2.0.\nConnector: original MIT code; tested Zotero10.0.6. Formula/semantic/browser install packages excluded pending complete implementation/licensing. Runtime licenses ship inside NSIS.\n',encoding='utf-8')
    files=[{'name':p.name,'size':p.stat().st_size,'sha256':sha(p)} for p in sorted(output.iterdir()) if p.is_file()]
    (output/'SHA256SUMS').write_text(''.join(f'{p["sha256"]}  {p["name"]}\n' for p in files),encoding='ascii')
    metadata={'status':'MANUAL_STAGED_NOT_PUBLISHED','deliveryMode':MODE,'repository':repo,'version':v,'tag':tag,'sourceCommit':git('rev-parse','HEAD'),'applicationDigest':application_digest(),'installerSha256':sha(output/installer.name),'zoteroXpiSha256':x['sha256'],'hashVerification':'PASS','files':files}
    (output/'release-build.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':metadata['status'],'assets':len(files),'version':v}))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--verified-cache',type=Path);a=p.parse_args()
    try:build(a.output.resolve(),a.verified_cache.resolve() if a.verified_cache else None)
    except (ValueError,OSError,subprocess.CalledProcessError) as e:p.exit(1,str(e)+'\n')
