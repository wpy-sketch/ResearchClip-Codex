"""Unit-only GitHub simulation: no real releases, credentials or QA receipts."""
import contextlib,hashlib,io,json,tempfile,types,unittest
from pathlib import Path
from unittest.mock import patch
import publish_verified_release as publisher
class PublicationTests(unittest.TestCase):
 def exercise(self,failed=False):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);(root/'artifacts').mkdir();assets=root/'assets';assets.mkdir();repo='example/unit';tag='v5.0.1';commit='a'*40;name='ResearchClip_5.0.1_x64-setup.exe';data=b'unit installer bytes';(assets/name).write_bytes(data)
   for f in ['release-build.json','SHA256SUMS']:(assets/f).write_bytes(b'unit metadata')
   meta={'deliveryMode':publisher.MANUAL_MODE,'repository':repo,'version':'5.0.1','tag':tag,'sourceCommit':commit,'files':[{'name':name,'size':len(data),'sha256':hashlib.sha256(data).hexdigest()}]}
   names=[name,'release-build.json','SHA256SUMS'];state={'created':False,'public':False};calls=[];hashes=[]
   def remote():return [{'id':i+1,'name':f,'size':(assets/f).stat().st_size,'browser_download_url':'https://github.com/example/unit/releases/download/'+('v5.0.1/' if state['public'] else 'untagged-unit/')+f} for i,f in enumerate(names)]
   def api(path):
    if path=='repos/'+repo:return {'full_name':repo,'permissions':{'push':True}}
    if path=='repos/'+repo+'/commits/'+commit:return {'sha':commit}
    if path=='repos/'+repo+'/releases/123':return {'draft':not state['public'],'prerelease':False,'tag_name':tag,'html_url':'https://github.com/example/unit/releases/tag/'+tag,'assets':remote()}
    raise AssertionError('Must read a draft by numeric ID, not published-tag endpoint: '+path)
   def gh(*args,**kw):
    calls.append(args)
    if args[:2]==('release','view'):
     if not state['created']:return types.SimpleNamespace(returncode=1,stderr='release not found',stdout='')
     return types.SimpleNamespace(returncode=0,stderr='',stdout=json.dumps({'databaseId':123,'isDraft':True,'tagName':tag,'targetCommitish':commit}))
    if args[0]=='api' and kw.get('optional'):return types.SimpleNamespace(returncode=1,stderr='HTTP 404',stdout='')
    if args[:2]==('release','create'):
     self.assertEqual(args[args.index('--target')+1],commit);state['created']=True
    if args[:2]==('release','edit'):state['public']='--draft=false' in args
    if args[:2]==('release','download'):
     filename=args[args.index('--pattern')+1];dest=Path(args[args.index('--dir')+1]);(dest/filename).write_bytes((assets/filename).read_bytes())
    return types.SimpleNamespace(returncode=0,stderr='',stdout='')
   def public_hash(url,item):
    hashes.append(url);self.assertIn('/download/v5.0.1/',url)
    if failed:raise ValueError('unit hash mismatch')
   with patch.object(publisher,'ROOT',root),patch.object(publisher,'preflight',return_value=meta),patch.object(publisher,'api',side_effect=api),patch.object(publisher,'gh',side_effect=gh),patch.object(publisher,'public_hash',side_effect=public_hash),contextlib.redirect_stdout(io.StringIO()):
    if failed:
     with self.assertRaisesRegex(ValueError,'NOT verified'):publisher.publish(assets,root/'unit-receipt',root/'unit-notes')
     self.assertFalse(state['public']);self.assertFalse((root/'artifacts/final-publication.json').exists())
    else:
     publisher.publish(assets,root/'unit-receipt',root/'unit-notes');self.assertTrue(state['public']);self.assertEqual(len(hashes),3)
   self.assertFalse(any('--clobber' in c or '--force' in c or c[:2]==('release','delete') for c in calls))
 def test_draft_is_read_by_id_and_rotated_urls_are_refetched_after_publish(self):self.exercise()
 def test_public_hash_failure_returns_only_the_new_release_to_draft(self):self.exercise(True)
