"""Reuse reviewed XPI bytes only after hash and exact archived-source validation.

DEFLATE output can differ between native zlib builds. This is not a code update:
immutable Connector1.0.1 keeps its previously accepted original container.
"""
import hashlib, io, urllib.request, zipfile
VERSION = '1.0.1'
URL = 'https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.0.1/ResearchClip-Zotero-Connector-v1.0.1.xpi'
SIZE = 10503
SHA256 = '1f98896eeef54ead00c5405d2460578d3269d82a5a011038779c9a9e2973fdc2'

def validate_sources(body, sources):
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        names=archive.namelist()
        if len(names)!=len(set(names)) or set(names)!=set(sources):
            raise ValueError('XPI must contain only the exact reviewed source files')
        for entry in archive.infolist():
            if entry.is_dir() or entry.flag_bits & 1 or entry.file_size!=len(sources[entry.filename]):
                raise ValueError('Unexpected XPI entry metadata')
            if archive.read(entry)!=sources[entry.filename]:
                raise ValueError('XPI source differs from the current verified checkout')

def download_reviewed(url):
    with urllib.request.urlopen(url,timeout=90) as response:
        if not response.geturl().startswith('https://'):
            raise ValueError('Reviewed XPI redirected outside HTTPS')
        body=response.read(SIZE+1)
    return body

def stable_xpi_bytes(candidate, sources, version, fetch=download_reviewed):
    validate_sources(candidate,sources)
    if version!=VERSION:
        return candidate
    if len(candidate)==SIZE and hashlib.sha256(candidate).hexdigest()==SHA256:
        return candidate
    reviewed=fetch(URL)
    if len(reviewed)!=SIZE or hashlib.sha256(reviewed).hexdigest()!=SHA256:
        raise ValueError('Reviewed original XPI size/SHA-256 mismatch')
    validate_sources(reviewed,sources)
    return reviewed
