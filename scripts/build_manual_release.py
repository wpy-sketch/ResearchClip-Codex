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
    if set(packages)!={'model.ocr-line','plugin.zotero-connector','model.semantic','plugin.browser-clipper'}:raise ValueError('Only reviewed real resources can ship')
    connector=packages['plugin.zotero-connector'];asset=connector['files'][0]
    if connector['version']!=x['version'] or asset['sha256']!=x['sha256'] or asset['size']!=x['size'] or asset['url']!=f'https://github.com/{repo}/releases/download/{tag}/{Path(x["file"]).name}':raise ValueError('Compiled connector differs from exact current XPI/origin')
    if any(p['appVersion']!='='+v for p in packages.values()):raise ValueError('Compiled compatibility mismatch')
    import tempfile
    from build_clipper_component import build as build_clipper, NAME as clipper_name
    with tempfile.TemporaryDirectory(prefix='researchclip-clipper-release-') as temp:
        clipper=Path(temp)/clipper_name
        build_clipper(clipper)
        expected=packages['plugin.browser-clipper']['files'][0]
        if clipper.stat().st_size!=expected['size'] or sha(clipper)!=expected['sha256']:raise ValueError('Reproducible clipper differs from reviewed compiled archive')
        # Keep verified bytes through NSIS build without using a mutable shared artifact.
        clipper_bytes=clipper.read_bytes()
    subprocess.run(['node',str(ROOT/'scripts/with-rust.mjs'),'tauri','build','--bundles','nsis'],cwd=ROOT,check=True)
    installer=ROOT/f'src-tauri/target/release/bundle/nsis/ResearchClip_{v}_x64-setup.exe'
    output.mkdir(parents=True);shutil.copyfile(installer,output/installer.name);shutil.copyfile(ROOT/x['file'],output/asset['path'])
    (output/packages['plugin.browser-clipper']['files'][0]['path']).write_bytes(clipper_bytes)
    for package in (packages['model.ocr-line'],packages['model.semantic']):
        for asset in package['files']:
            dest=output/asset['path'];source=cache/asset['path'] if cache else None
            if source and source.is_file():
                if source.stat().st_size!=asset['size'] or sha(source)!=asset['sha256']:raise ValueError('Real cached model mismatch')
                shutil.copyfile(source,dest)
            else:
                from build_signed_release import download
                download(asset,dest)
    for filename in ['APACHE-2.0.txt','CLIP-MIT.txt','NOTICE.txt']:
        shutil.copyfile(ROOT/'semantic-search/licenses'/filename,output/('SEMANTIC-'+filename))
    shutil.copyfile(ROOT/'browser-extension/LICENSE',output/'CLIPPER-LICENSE-MIT.txt')
    history=json.loads((ROOT/'docs/release/researchclip-consolidation.json').read_text(encoding='utf-8'))
    history['verifiedMigration']=json.loads((ROOT/'docs/qa/evidence/consolidation-20261030/complete.json').read_text(encoding='utf-8'))
    (output/'HISTORY-INDEX.json').write_text(json.dumps(history,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    import zipfile
    names=['README.md',*(str(p.relative_to(ROOT)).replace('\\','/') for p in (ROOT/'docs').rglob('*.md')),*(str(p.relative_to(ROOT)).replace('\\','/') for p in (ROOT/'docs/images').iterdir() if p.is_file()),*(str(p.relative_to(ROOT)).replace('\\','/') for p in (ROOT/'docs/licenses').iterdir() if p.is_file()),'browser-extension/README.md','browser-extension/PROTOCOL.md','browser-extension/LICENSE','semantic-search/README.md','semantic-search/models.json','math-ocr/README.md','math-ocr/models.json']
    with zipfile.ZipFile(output/'USER-GUIDE.zip','x',compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('START_HERE.md','# ResearchClip offline documentation\n\nOpen docs/USER_GUIDE.md first. Images are local. These files are a build-time snapshot; historical QA reports retain their original status and dates. Final publication, downloads, hashes and acceptance scope: https://github.com/wpy-sketch/ResearchClip-Codex/blob/main/docs/qa/FINAL_RELEASE_REPORT.md\n')
        for name in sorted(set(names)):
            entry=zipfile.ZipInfo(name,(2026,10,30,0,0,0));entry.create_system=3;entry.external_attr=0o100644<<16;entry.compress_type=zipfile.ZIP_DEFLATED
            archive.writestr(entry,(ROOT/name).read_bytes())
    for name in ['INSTALLATION_GUIDE.md','EXTENSION_GUIDE.md','RELEASE_NOTES.md']:shutil.copyfile(ROOT/'docs/release'/name,output/name)
    shutil.copyfile(catalog_path,output/'catalog.json');shutil.copyfile(ROOT/'docs/licenses/PaddleOCR-Apache-2.0.txt',output/'OCR-LICENSE-APACHE-2.0.txt')
    for p in packages.values():(output/(p['id']+'.manifest.json')).write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    (output/'compatibility.json').write_text(json.dumps([{'id':p['id'],'appVersion':p['appVersion'],'platforms':p['platforms']} for p in packages.values()],indent=2)+'\n',encoding='utf-8')
    (output/'zotero-updates.json').write_text(json.dumps(update_manifest(x,repo,tag),indent=2)+'\n',encoding='utf-8')
    (output/'delivery.json').write_text(json.dumps({'schemaVersion':1,'mode':MODE,'updater':'DISABLED_MANUAL_DOWNLOAD','catalog':'EXACT_COMPILED_MANIFEST','signingKeysRequired':False},indent=2)+'\n',encoding='utf-8')
    (output/'COMPONENT-SOURCES.txt').write_text('Manual installation release; no Catalog or Updater signing keys. catalog.json is informational; clients use exact compiled definitions, HTTPS and SHA-256.\nPP-OCRv4: SWHL/RapidOCR revision1cfba2e90fc938db55889873735088de210cc173, Apache-2.0. Dictionary: PaddleOCRv2.7.3, Apache-2.0.\nConnector: original MIT code; tested Zotero10.0.6. Semantic model: fixed aligned CLIP image MIT and multilingual text Apache-2.0; 8 exact original files with CPU inference self-test. Clipper v1.0.3: owner-authorized MIT grant, original third-party notices and verified unchanged v1.0.2 native Host; external installation only. Formula native adapter remains unavailable, existing Python configuration retained. Historical attachments: HISTORY-INDEX.json maps every original file to the separately verified repository archive. Runtime licenses ship inside NSIS.\n',encoding='utf-8')
    files=[{'name':p.name,'size':p.stat().st_size,'sha256':sha(p)} for p in sorted(output.iterdir()) if p.is_file()]
    (output/'SHA256SUMS').write_text(''.join(f'{p["sha256"]}  {p["name"]}\n' for p in files),encoding='ascii')
    metadata={'status':'MANUAL_STAGED_NOT_PUBLISHED','deliveryMode':MODE,'repository':repo,'version':v,'tag':tag,'sourceCommit':git('rev-parse','HEAD'),'applicationDigest':application_digest(),'installerSha256':sha(output/installer.name),'zoteroXpiSha256':x['sha256'],'hashVerification':'PASS','files':files}
    (output/'release-build.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':metadata['status'],'assets':len(files),'version':v}))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--verified-cache',type=Path);a=p.parse_args()
    try:build(a.output.resolve(),a.verified_cache.resolve() if a.verified_cache else None)
    except (ValueError,OSError,subprocess.CalledProcessError) as e:p.exit(1,str(e)+'\n')
