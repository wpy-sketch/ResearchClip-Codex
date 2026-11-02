import copy
import json
import unittest
from finalize import BASE, SOURCE, check_release
from transfer import manifest


class FinalizeTests(unittest.TestCase):
    def setUp(self):
        self.expected=manifest(json.loads((BASE/'manifest.json').read_text(encoding='utf-8')))
        self.release={'id':431008505,'draft':True,'prerelease':False,'tag_name':'v5.1.1','target_commitish':SOURCE,'assets':[{'id':i,'name':name,'state':'uploaded','size':e['size'],'digest':'sha256:'+e['sha256']} for i,(name,e) in enumerate(self.expected.items())]}

    def test_complete_exact_draft(self):self.assertEqual(len(check_release(self.release,self.expected,True)),34)

    def test_complete_exact_public_state(self):self.assertEqual(len(check_release({**self.release,'draft':False},self.expected,False)),34)

    def test_wrong_identity_or_state_refused(self):
        for patch in [{'id':1},{'tag_name':'v5.1.0'},{'target_commitish':'a'*40},{'draft':False},{'prerelease':True}]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):check_release({**self.release,**patch},self.expected,True)

    def test_missing_extra_duplicate_or_corrupt_assets_refused(self):
        for patch in [self.release['assets'][:-1],self.release['assets']+[self.release['assets'][0]],self.release['assets']+[{'name':'unknown'}]]:
            with self.subTest(patch=patch),self.assertRaises(ValueError):check_release({**self.release,'assets':patch},self.expected,True)
        r=copy.deepcopy(self.release);r['assets'][0]['digest']='sha256:'+'a'*64
        with self.assertRaises(ValueError):check_release(r,self.expected,True)

    def test_different_installer_refused(self):
        expected=copy.deepcopy(self.expected);expected['ResearchClip_5.1.1_x64-setup.exe']['sha256']='a'*64
        with self.assertRaises(ValueError):check_release(self.release,expected,True)


if __name__ == '__main__':unittest.main()
