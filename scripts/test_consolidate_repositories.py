"""Isolated archive-contract tests; no GitHub mutations or real data."""
import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

import consolidate_repositories as m


class ArchiveTests(unittest.TestCase):
    def fixture(self):
        data = b'historical data, never executable test output'
        item = {'sha256': hashlib.sha256(data).hexdigest(), 'size': len(data), 'member': '1234-original.bin', 'sourceUrl': 'https://github.com/' + m.SOURCE + '/releases/download/v1.0.0/test.bin'}
        return data, {'name': 'part.zip', 'members': [item]}

    def test_fresh_zip_is_deterministic_and_each_original_hash_is_checked(self):
        data, volume = self.fixture()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def download(url, path, size, sha):
                self.assertEqual(size, len(data))
                self.assertEqual(sha, hashlib.sha256(data).hexdigest())
                path.write_bytes(data)
            with patch.object(m, 'public_download', side_effect=download):
                m.make_volume(volume, root / 'one.zip', root)
                m.make_volume(volume, root / 'two.zip', root)
            self.assertEqual((root / 'one.zip').read_bytes(), (root / 'two.zip').read_bytes())
            m.volume_check(root / 'one.zip', volume)

    def test_modified_bytes_extra_members_and_traversal_are_rejected(self):
        data, volume = self.fixture()
        with tempfile.TemporaryDirectory() as temp:
            for index, entries in enumerate([{'1234-original.bin': b'wrong'}, {'1234-original.bin': data, 'extra': b'x'}, {'../escape': data}]):
                path = Path(temp) / str(index)
                with zipfile.ZipFile(path, 'x') as z:
                    for name, body in entries.items():
                        z.writestr(name, body)
                with self.assertRaises(AssertionError):
                    m.volume_check(path, volume)

    def test_unexpected_existing_asset_never_gets_replaced(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'file.zip'
            path.write_bytes(b'correct')
            existing = {'file.zip': {'state': 'uploaded', 'size': 7, 'digest': 'sha256:' + 'a' * 64}}
            with self.assertRaises(AssertionError):
                m.upload(None, {'draft': True}, path, existing)

    def test_current_plan_covers_every_reference_and_has_bounded_volumes(self):
        plan, identity = m.load_plan(m.ROOT / 'docs/release/researchclip-consolidation.json')
        self.assertEqual(len(plan['releases']), 47)
        self.assertEqual(sum(len(r['assets']) for r in plan['releases']), 4738)
        self.assertEqual(len(plan['uniqueAssets']), 543)
        self.assertEqual(len(identity), 64)
        self.assertEqual(sum(1 for i, _ in enumerate(m.tasks(plan)) for shard in range(m.SHARDS) if i % m.SHARDS == shard), len(m.tasks(plan)))

    def test_tokens_cannot_be_redirected_or_used_to_modify_the_old_repository(self):
        with patch.dict(m.os.environ, {'GH_TOKEN': 'unit-only', 'GITHUB_REPOSITORY': m.TARGET}):
            client = m.GitHub()
            with self.assertRaises(AssertionError):
                client.request('https://api.github.com/repos/' + m.SOURCE + '/releases', 'POST', {})
            with self.assertRaises(AssertionError):
                client.request('https://evil.invalid/repos/' + m.TARGET + '/releases')
        with patch.dict(m.os.environ, {'GH_TOKEN': 'unit-only', 'GITHUB_REPOSITORY': m.SOURCE}):
            with self.assertRaises(AssertionError):
                m.GitHub()

    def test_public_archive_is_not_mutated(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'data.bin'
            path.write_bytes(b'plain data')
            with self.assertRaises(AssertionError):
                m.upload(None, {'draft': False}, path, {})

    def test_pinned_github_generated_tag_is_preserved_without_accepting_arbitrary_refs(self):
        self.assertTrue(m.ref_name_ok('refs/tags/untagged-e095ea8690b41054e2e5'))
        self.assertTrue(m.ref_name_ok('refs/tags/v4.2.7'))
        for value in ['refs/heads/other', 'refs/tags/../../main', 'refs/tags/untagged-user-input', 'refs/tags/v4.2.7;command']:
            self.assertFalse(m.ref_name_ok(value))


if __name__ == '__main__':
    unittest.main()
