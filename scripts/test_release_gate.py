"""Contract fixtures only; never write a real acceptance receipt or publish."""
import copy,json,tempfile,unittest
import build_signed_release
from zotero_connector_release import update_manifest
from pathlib import Path
from unittest.mock import patch
from release_gate import repository,trust_ready,validate_receipt,validate_asset_links,REQUIRED_TESTS
import publish_verified_release

class ReleaseGateTests(unittest.TestCase):
    def test_origin_is_parsed_not_guessed_and_credentials_or_foreign_hosts_are_rejected(self):
        self.assertEqual(repository('https://github.com/owner/repository.git'),'owner/repository')
        self.assertEqual(repository('git@github.com:owner/repository.git'),'owner/repository')
        for url in ['https://github.com.evil.test/owner/repo','https://token@github.com/owner/repo','https://github.com/a/b/c','D:/workspace']:
            with self.assertRaises(ValueError):repository(url)

    def test_unpublished_or_wrong_repository_trust_is_not_a_unsigned_fallback(self):
        trust={'schemaVersion':1,'catalogPublicKey':None,'updaterPublicKey':None,'catalogUrl':None,'updaterUrl':None}
        with self.assertRaises(ValueError):trust_ready(trust,'owner/repository')
        trust.update(catalogPublicKey='public-test-value',updaterPublicKey='public-test-value',catalogUrl='https://github.com/owner/repository/releases/latest/download/catalog.json',updaterUrl='https://github.com/other/repo/releases/latest/download/latest.json')
        with self.assertRaises(ValueError):trust_ready(trust,'owner/repository')

    def test_receipt_requires_exact_bytes_and_every_real_acceptance_gate(self):
        fixture={'result':'PASS','applicationDigest':'source','installerSha256':'installer','zoteroXpiSha256':'xpi','mode':'actual-independent-windows-and-zotero','isolatedProfile':True,'zoteroVersion':'10.0.6','checks':{name:True for name in REQUIRED_TESTS},'unresolvedP0':0,'unresolvedP1':0}
        validate_receipt(fixture,'source','installer','xpi')
        for key,value in [('installerSha256','wrong'),('mode','browser-fixture'),('zoteroVersion','unknown'),('unresolvedP1',1)]:
            with self.assertRaises(ValueError):validate_receipt({**fixture,key:value},'source','installer','xpi')
        for name in REQUIRED_TESTS:
            with self.assertRaises(ValueError):validate_receipt({**fixture,'checks':{**fixture['checks'],name:False}},'source','installer','xpi')

    def test_asset_links_are_closed_and_match_exact_installer_and_model_files(self):
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder);v='4.4.0';base='https://github.com/owner/repository/releases/download/v4.4.0/';exe='ResearchClip_4.4.0_x64-setup.exe'
            files=[{'name':n,'sha256':'unit-hash','size':7} for n in [exe,exe+'.sig','rec.onnx','characters.txt','connector.xpi']]
            metadata={'version':v,'tag':'v'+v,'repository':'owner/repository','files':files}
            packages=[{'id':identity,'version':'1.0.1','appVersion':'='+v,'files':[{'path':n,'url':base+n,'sha256':'unit-hash','size':7} for n in names]} for identity,names in [('model.ocr-line',['rec.onnx','characters.txt']),('plugin.zotero-connector',['connector.xpi'])]]
            (out/(exe+'.sig')).write_text('unit-public-signature',encoding='utf-8')
            latest={'version':v,'platforms':{'windows-x86_64':{'url':base+exe,'signature':'unit-public-signature'}}}
            (out/'latest.json').write_text(json.dumps(latest),encoding='utf-8')
            for package in packages:(out/(package['id']+'.manifest.json')).write_text(json.dumps(package),encoding='utf-8')
            catalog={'packages':packages}
            def save_catalog(): (out/'catalog.json').write_text(json.dumps({'payload':json.dumps(catalog)}),encoding='utf-8')
            save_catalog();index=json.dumps(update_manifest({'version':'1.0.1','pluginId':'researchclip-connector@wpy-sketch.github.io','sha256':'a'*64},'owner/repository','v4.4.0')).replace('a'*64,'unit-hash').replace('ResearchClip-Zotero-Connector-v1.0.1.xpi','connector.xpi')
            (out/'zotero-updates.json').write_text(index,encoding='utf-8');validate_asset_links(out,metadata)
            for old,new in [('unit-hash','different-hash'),('connector.xpi','other.xpi'),('1.0.1','1.0.2'),('10.0.6','10.0.7')]:
                (out/'zotero-updates.json').write_text(index.replace(old,new),encoding='utf-8')
                with self.subTest(field=old),self.assertRaises(ValueError):validate_asset_links(out,metadata)
            (out/'zotero-updates.json').write_text(index,encoding='utf-8')
            altered=copy.deepcopy(metadata);altered['files'].append({'name':'unexpected-native.dll','size':1,'sha256':'unit'})
            with self.assertRaises(ValueError):validate_asset_links(out,altered)
            catalog['packages'][0]['files'][0]['url']='https://untrusted.invalid/model';save_catalog()
            with self.assertRaises(ValueError):validate_asset_links(out,metadata)
            catalog['packages']=packages[1:];save_catalog()
            with self.assertRaises(ValueError):validate_asset_links(out,metadata)

    def test_signed_builder_with_missing_production_trust_creates_no_assets(self):
        with tempfile.TemporaryDirectory() as folder,patch.object(build_signed_release,'trust_ready',side_effect=ValueError('UNPUBLISHED')),patch.object(build_signed_release,'run') as run:
            output=Path(folder)/'must-not-exist'
            with self.assertRaises(ValueError):build_signed_release.build(output)
            self.assertFalse(output.exists());run.assert_not_called()

    def test_failed_gate_never_calls_github_or_creates_a_draft(self):
        with tempfile.TemporaryDirectory() as folder,patch.object(publish_verified_release,'preflight',side_effect=ValueError('NOT_READY')),patch.object(publish_verified_release,'gh') as gh:
            with self.assertRaises(ValueError):publish_verified_release.publish(Path(folder),Path(folder)/'receipt.json',Path(folder)/'notes.md')
            gh.assert_not_called()

if __name__=='__main__':unittest.main()
