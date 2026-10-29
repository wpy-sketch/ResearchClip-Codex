"""Pure guard unit fixtures, not real Zotero evidence or installation tests."""
import importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('connector_release',Path(__file__).with_name('zotero_connector_release.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
class ReleaseGateTests(unittest.TestCase):
 def test_host_manifest_rejects_missing_update_url_foreign_source_and_untested_host(self):
  import copy,json
  manifest=json.loads((module.ROOT/'zotero-connector/manifest.json').read_text(encoding='utf-8'))
  module.validate_package_manifest(manifest,'wpy-sketch/ResearchClip-Codex')
  for name,value in [('update_url',None),('update_url','https://untrusted.invalid/updates.json'),('strict_max_version','*'),('id','other@example.invalid')]:
   changed=copy.deepcopy(manifest);changed['applications']['zotero'][name]=value
   with self.subTest(name=name,value=value),self.assertRaises(ValueError):module.validate_package_manifest(changed,'wpy-sketch/ResearchClip-Codex')
 def receipt(self):return {'result':'PASS','mode':'actual-independent-zotero','appVersion':module.current_versions()[0],'zoteroVersion':'10.0.6','pluginVersion':module.current_versions()[1],'isolatedProfile':True,'xpiSha256':'a'*64,'checks':{name:True for name in module.REQUIRED_CHECKS}}
 def test_explicit_complete_matching_fixture_is_accepted_by_parser_only(self):module.validate_native_receipt(self.receipt(),{'sha256':'a'*64})
 def test_mock_scope_future_host_wrong_app_hash_or_nonisolated_are_rejected(self):
  for patch in [{'mode':'mock'},{'zoteroVersion':'10.0.7'},{'appVersion':'4.2.7'},{'isolatedProfile':False},{'xpiSha256':'b'*64},{'result':'PASS_LIMITED'}]:
   with self.subTest(patch=patch),self.assertRaises(ValueError):module.validate_native_receipt({**self.receipt(),**patch},{'sha256':'a'*64})
 def test_missing_or_failed_case_is_not_a_pass(self):
  for value in [False,None,'true',1]:
   r=self.receipt();r['checks']['contextMenu']=value
   with self.subTest(value=value),self.assertRaises(ValueError):module.validate_native_receipt(r,{'sha256':'a'*64})
  r=self.receipt();r['checks'].pop('installed')
  with self.assertRaises(ValueError):module.validate_native_receipt(r,{'sha256':'a'*64})
 def test_no_real_receipt_does_not_create_or_fake_one(self):
  import tempfile
  with tempfile.TemporaryDirectory() as folder:
   with self.assertRaises(ValueError):module.checked_xpi(Path(folder))
   self.assertEqual(list(Path(folder).iterdir()),[])
if __name__=='__main__':unittest.main()
