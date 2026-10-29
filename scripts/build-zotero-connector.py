"""Deterministic original Zotero XPI; only ZIP metadata/code packaging, never installation."""
from pathlib import Path
import hashlib,json,zipfile,io,re
from release_gate import git,repository
from zotero_connector_release import validate_package_manifest
root=Path(__file__).resolve().parent.parent;source=root/'zotero-connector'
manifest=json.loads((source/'manifest.json').read_text(encoding='utf-8'));version=manifest['version'];assert re.fullmatch(r'\d+\.\d+\.\d+',version)
validate_package_manifest(manifest,repository(git('remote','get-url','origin')))
files=['manifest.json','bootstrap.js','connector.js','LICENSE','README.md'];stream=io.BytesIO()
with zipfile.ZipFile(stream,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
 for name in files:
  info=zipfile.ZipInfo(name,date_time=(2026,10,27,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16;z.writestr(info,(source/name).read_bytes())
body=stream.getvalue();digest=hashlib.sha256(body).hexdigest();out=root/'artifacts/zotero-connector-build'/digest;out.mkdir(parents=True,exist_ok=True);path=out/f'ResearchClip-Zotero-Connector-v{version}.xpi'
if path.exists():assert path.read_bytes()==body
else:
 with path.open('xb') as f:f.write(body)
record={'version':version,'pluginId':manifest['applications']['zotero']['id'],'file':str(path.relative_to(root)),'size':len(body),'sha256':digest,'license':'MIT','compatibility':manifest['applications']['zotero'],'installed':False}
index=root/'artifacts/zotero-connector-build/latest.json';index.write_text(json.dumps(record,indent=2),encoding='utf-8');print(json.dumps(record,indent=2))
