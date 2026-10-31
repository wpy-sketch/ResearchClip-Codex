"""Manual delivery contract tests only; fixtures are never real acceptance receipts."""
import json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import release_gate as gate
import build_manual_release as builder
from zotero_connector_release import update_manifest

class ManualReleaseTests(unittest.TestCase):
    def test_local_scope_requires_explicit_confirmation_and_disclosure_and_keeps_core_tests(self):
        receipt={'result':'PASS','applicationDigest':'source','installerSha256':'exe','zoteroXpiSha256':'xpi','mode':'actual-local-windows-and-zotero','isolatedProfile':True,'zoteroVersion':'10.0.6','unresolvedP0':0,'unresolvedP1':0,'checks':{k:True for k in gate.MANUAL_REQUIRED_TESTS},'independentWindows':False,'manualAcceptanceScope':'user-confirmed-local-only','manualOutstanding':['independentWindows']}
        gate.validate_receipt(receipt,'source','exe','xpi',gate.MANUAL_MODE)
        for patch in [{'manualAcceptanceScope':'pending'},{'manualOutstanding':[]},{'isolatedProfile':False},{'unresolvedP1':1},{'checks':{**receipt['checks'],'ocrInference':False}}]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):gate.validate_receipt({**receipt,**patch},'source','exe','xpi',gate.MANUAL_MODE)
    def test_manual_receipt_drops_only_signing_requirement_and_keeps_every_core_gate(self):
        receipt={'result':'PASS','applicationDigest':'source','installerSha256':'exe','zoteroXpiSha256':'xpi','mode':'actual-independent-windows-and-zotero','isolatedProfile':True,'zoteroVersion':'10.0.6','unresolvedP0':0,'unresolvedP1':0,'checks':{k:True for k in gate.MANUAL_REQUIRED_TESTS}}
        gate.validate_receipt(receipt,'source','exe','xpi',gate.MANUAL_MODE)
        for key in gate.MANUAL_REQUIRED_TESTS:
            with self.subTest(key=key),self.assertRaises(ValueError):gate.validate_receipt({**receipt,'checks':{**receipt['checks'],key:False}},'source','exe','xpi',gate.MANUAL_MODE)
        with self.assertRaises(ValueError):gate.validate_receipt({**receipt,'unresolvedP1':1},'source','exe','xpi',gate.MANUAL_MODE)

    def test_compiled_catalog_exact_assets_and_no_fake_updater(self):
        compiled=gate.ROOT/'src-tauri/resources/package-builtin.json';catalog=json.loads(compiled.read_text(encoding='utf-8'))
        repo='wpy-sketch/ResearchClip-Codex';v=gate.version();tag='v'+v
        with tempfile.TemporaryDirectory() as temp:
            out=Path(temp);(out/'catalog.json').write_bytes(compiled.read_bytes());files={}
            for p in catalog['packages']:
                name=p['id']+'.manifest.json';(out/name).write_text(json.dumps(p),encoding='utf-8');files[name]={'name':name,'size':1,'sha256':'unit'}
                for a in p['files']:files[a['path']]={'name':a['path'],'size':a['size'],'sha256':a['sha256']}
            x=next(p for p in catalog['packages'] if p['id']=='plugin.zotero-connector');a=x['files'][0]
            (out/'zotero-updates.json').write_text(json.dumps(update_manifest({'version':x['version'],'pluginId':'researchclip-connector@wpy-sketch.github.io','sha256':a['sha256']},repo,tag)),encoding='utf-8')
            policy={'schemaVersion':1,'mode':gate.MANUAL_MODE,'updater':'DISABLED_MANUAL_DOWNLOAD','catalog':'EXACT_COMPILED_MANIFEST','signingKeysRequired':False}
            (out/'delivery.json').write_text(json.dumps(policy),encoding='utf-8')
            for name in [f'ResearchClip_{v}_x64-setup.exe','catalog.json','delivery.json','compatibility.json','zotero-updates.json','COMPONENT-SOURCES.txt','OCR-LICENSE-APACHE-2.0.txt','SEMANTIC-APACHE-2.0.txt','SEMANTIC-CLIP-MIT.txt','SEMANTIC-NOTICE.txt','CLIPPER-LICENSE-MIT.txt','HISTORY-INDEX.json','USER-GUIDE.zip','INSTALLATION_GUIDE.md','EXTENSION_GUIDE.md','RELEASE_NOTES.md']:files[name]={'name':name,'size':1,'sha256':'unit'}
            metadata={'deliveryMode':gate.MANUAL_MODE,'version':v,'tag':tag,'repository':repo,'files':list(files.values())};gate.validate_asset_links(out,metadata)
            metadata['files'].append({'name':'latest.json','size':1,'sha256':'unit'})
            with self.assertRaises(ValueError):gate.validate_asset_links(out,metadata)
            metadata['files'].pop();(out/'catalog.json').write_bytes(compiled.read_bytes()+b' ')
            with self.assertRaises(ValueError):gate.validate_asset_links(out,metadata)

    def test_existing_staging_is_never_overwritten_and_no_signing_key_is_read(self):
        with tempfile.TemporaryDirectory() as temp,patch.object(builder,'git',return_value='https://github.com/wpy-sketch/ResearchClip-Codex.git'),patch.object(builder.subprocess,'run') as run:
            with self.assertRaises(ValueError):builder.build(Path(temp))
            run.assert_not_called()

if __name__=='__main__':unittest.main()
