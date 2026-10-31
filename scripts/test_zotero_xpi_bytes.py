import hashlib,io,unittest,zipfile
from unittest.mock import patch
import zotero_xpi_bytes as xpi

def pack(sources,level=9):
    stream=io.BytesIO()
    with zipfile.ZipFile(stream,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=level) as archive:
        for name,body in sources.items():archive.writestr(name,body)
    return stream.getvalue()

class StableXpiTests(unittest.TestCase):
    def setUp(self):
        self.sources={'manifest.json':b'{"version":"1.0.1"}','connector.js':b'const original = true;\n'*100}
        self.reviewed=pack(self.sources);self.alternative=pack(self.sources,0)
        self.patches=[patch.object(xpi,'SIZE',len(self.reviewed)),patch.object(xpi,'SHA256',hashlib.sha256(self.reviewed).hexdigest())]
        for p in self.patches:p.start();self.addCleanup(p.stop)
    def test_exact_candidate_avoids_network(self):
        self.assertEqual(xpi.stable_xpi_bytes(self.reviewed,self.sources,xpi.VERSION,lambda _:self.fail('unexpected network')),self.reviewed)
    def test_different_compression_reuses_only_pinned_identical_sources(self):
        urls=[]
        def fetch(url):urls.append(url);return self.reviewed
        self.assertNotEqual(self.alternative,self.reviewed)
        self.assertEqual(xpi.stable_xpi_bytes(self.alternative,self.sources,xpi.VERSION,fetch),self.reviewed)
        self.assertEqual(urls,[xpi.URL])
    def test_corrupt_download_rejected(self):
        with self.assertRaisesRegex(ValueError,'SHA-256'):xpi.stable_xpi_bytes(self.alternative,self.sources,xpi.VERSION,lambda _:self.reviewed+b'x')
    def test_source_change_cannot_reuse_original_container(self):
        modified={**self.sources,'connector.js':b'changed source'}
        with self.assertRaises(ValueError):xpi.stable_xpi_bytes(pack(modified),modified,xpi.VERSION,lambda _:self.reviewed)
    def test_extra_or_duplicate_archive_entries_rejected(self):
        with self.assertRaises(ValueError):xpi.validate_sources(pack({**self.sources,'../evil.js':b'x'}),self.sources)
        stream=io.BytesIO()
        with zipfile.ZipFile(stream,'w') as archive:
            archive.writestr('connector.js',b'a');archive.writestr('connector.js',b'b')
        with self.assertRaises(ValueError):xpi.validate_sources(stream.getvalue(),{'connector.js':b'a'})
    def test_future_version_is_not_silently_downgraded(self):
        self.assertEqual(xpi.stable_xpi_bytes(self.alternative,self.sources,'1.0.2',lambda _:self.fail('unexpected network')),self.alternative)
