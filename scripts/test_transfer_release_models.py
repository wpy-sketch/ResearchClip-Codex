import copy,unittest
from transfer_release_models import reviewed_models
class TransferTests(unittest.TestCase):
 def setUp(self):
  self.files=[{'path':n,'size':100,'sha256':'a'*64,'url':'https://huggingface.co/owner/model/resolve/'+'b'*40+'/'+n} for n in ['vision.onnx','text.onnx']]
  self.catalog={'packages':[{'id':'model.semantic','files':self.files}]}
  self.metadata={'repository':'owner/repo','tag':'v5.1.0','sourceCommit':'c'*40,'files':[{'name':f['path'],'size':f['size'],'sha256':f['sha256']} for f in self.files]}
 def run_case(self):return reviewed_models(self.catalog,self.metadata,'owner/repo','v5.1.0','c'*40)
 def test_exact_two_models(self):self.assertEqual(self.run_case(),self.files)
 def test_wrong_repo_or_commit_rejected(self):
  for key in ['repository','sourceCommit']:
   original=self.metadata[key];self.metadata[key]='wrong'
   with self.assertRaises(ValueError):self.run_case()
   self.metadata[key]=original
 def test_changed_hash_rejected(self):
  self.metadata['files'][0]['sha256']='d'*64
  with self.assertRaises(ValueError):self.run_case()
 def test_arbitrary_source_rejected(self):
  self.files[0]['url']='https://untrusted.invalid/file'
  with self.assertRaises(ValueError):self.run_case()
 def test_missing_or_duplicate_model_rejected(self):
  self.catalog['packages'][0]['files']=[self.files[0],self.files[0]]
  with self.assertRaises(ValueError):self.run_case()
