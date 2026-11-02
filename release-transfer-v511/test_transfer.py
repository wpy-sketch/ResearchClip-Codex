import copy
import json
import tempfile
import unittest
from pathlib import Path
import transfer


class TransportTests(unittest.TestCase):
    def setUp(self):
        self.value = json.loads((transfer.BASE / 'manifest.json').read_text(encoding='utf-8'))
        self.expected = transfer.manifest(self.value)
        self.release = {'id':431008505,'draft':True,'prerelease':False,'tag_name':'v5.1.1','target_commitish':transfer.SOURCE,'assets':[]}

    def test_exact_local_payload_hashes(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name in self.value['payload']:
                p=Path(tmp)/name;transfer.assemble(name,self.value,p);transfer.verify_file(p,self.expected[name])

    def test_reject_other_release_or_commit(self):
        for patch in [{'tag':'v5.1.0'},{'sourceCommit':'a'*40},{'repository':'other/repository'},{'releaseId':1}]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):transfer.manifest({**self.value,**patch})

    def test_reject_paths_duplicates_and_unexpected_payload(self):
        for name in ['../private','a/b','..','x\\y']:
            v=copy.deepcopy(self.value);v['expected'][0]['name']=name
            with self.subTest(name=name),self.assertRaises(ValueError):transfer.manifest(v)
        v=copy.deepcopy(self.value);v['expected'].append(v['expected'][0])
        with self.assertRaises(ValueError):transfer.manifest(v)
        with self.assertRaises(ValueError):transfer.manifest({**self.value,'payload':['unknown']})

    def test_refuse_published_or_different_draft(self):
        for patch in [{'draft':False},{'id':1},{'prerelease':True},{'target_commitish':'b'*40}]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):transfer.remote_assets({**self.release,**patch},self.expected)

    def test_existing_asset_must_be_complete_and_exact(self):
        e=next(iter(self.expected.values()));a={'name':e['name'],'size':e['size'],'state':'uploaded','digest':'sha256:'+e['sha256']}
        self.assertIn(e['name'],transfer.remote_assets({**self.release,'assets':[a]},self.expected))
        for patch in [{'state':'starter'},{'size':0},{'digest':'sha256:'+'a'*64},{'name':'unknown'}]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):transfer.remote_assets({**self.release,'assets':[{**a,**patch}]},self.expected)

    def test_corrupt_transport_refuses(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'corrupt';p.write_bytes(b'changed')
            with self.assertRaises(ValueError):transfer.verify_file(p,next(iter(self.expected.values())))

    def test_unsafe_missing_and_over_budget_parts_refuse(self):
        name=self.value['payload'][0]
        for patch in [{'file':'../private'},{'file':'00-9999.bin'},{'size':524289}]:
            v=copy.deepcopy(self.value);v['parts'][name][0].update(patch)
            with tempfile.TemporaryDirectory() as tmp,self.assertRaises(ValueError):transfer.assemble(name,v,Path(tmp)/name)


if __name__ == '__main__':unittest.main()
