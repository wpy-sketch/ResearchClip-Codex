"""Repackage the unchanged, hash-pinned released clipper with owner's MIT grant.

No binary is executed, registered or injected. The external browser retains its
own installation/permission flow. Deterministic ZIP bytes allow CI verification.
"""
import argparse
import hashlib
import json
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parent.parent
BASE = 'https://github.com/wpy-sketch/ResearchClip/releases/download/v3.1.3/'
SOURCE = 'ResearchClip-WebClipper-v1.0.2-windows-x64.zip'
SOURCE_SHA = '1e456e8cc198f1b366afc47614cc5ad3dbf14410ad5a52aad21cf61d551cd4ce'
HOST_SHA = '1a30f11a1cf39f31b262bdffba2ce800d7d664ceeab5504b2d174aaaeee69a23'
NAME = 'ResearchClip-WebClipper-v1.0.3-windows-x64.zip'


def build(output, source=None):
    import io
    if output.exists():
        raise ValueError('Existing component is retained; use a new output path')
    if source:
        data = source.read_bytes()
    else:
        with urllib.request.urlopen(BASE + SOURCE, timeout=90) as response:
            if not response.geturl().startswith('https://'):
                raise ValueError('HTTPS required')
            data = response.read(541480)
    if len(data) != 541479 or hashlib.sha256(data).hexdigest() != SOURCE_SHA:
        raise ValueError('Previously released source archive mismatch')
    files = {}
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        for member in archive.infolist():
            name = member.filename
            if (name in files or member.is_dir() or '\\' in name or ':' in name
                    or PurePosixPath(name).is_absolute() or '..' in PurePosixPath(name).parts
                    or member.file_size > 2_000_000
                    or member.external_attr >> 16 & 0o170000 == 0o120000):
                raise ValueError('Unsafe historical archive member')
            files[name] = archive.read(member)
    if hashlib.sha256(files['bin/researchclip-native-host.exe']).hexdigest() != HOST_SHA:
        raise ValueError('Trusted host differs')
    # Preserve runtime JS, third-party notices and the original fixed native host.
    files['dist/manifest.json'] = (ROOT / 'browser-extension/manifest.json').read_bytes()
    for name in ['LICENSE', 'README.md', 'PROTOCOL.md', 'register-host.ps1']:
        files[name] = (ROOT / 'browser-extension' / name).read_text(encoding='utf-8').replace('\r\n', '\n').encode()
    files['BINARY-PROVENANCE.json'] = (json.dumps({
        'packageVersion': '1.0.3', 'hostVersion': '1.0.2', 'protocolVersion': 1,
        'sourceArchive': BASE + SOURCE, 'sourceSha256': SOURCE_SHA,
        'hostSha256': HOST_SHA,
        'changes': 'Owner-authorized MIT license, guide and package metadata; runtime code retained.'
    }, indent=2) + '\n').encode()
    # Normalize generated JSON independently of Windows checkout line endings.
    files['dist/manifest.json'] = (json.dumps(json.loads(files['dist/manifest.json']), ensure_ascii=False, indent=2) + '\n').encode()
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, 'x', compression=zipfile.ZIP_STORED) as archive:
        for name, data in sorted(files.items()):
            entry = zipfile.ZipInfo(name, (2026, 10, 30, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            entry.compress_type = zipfile.ZIP_STORED
            archive.writestr(entry, data)
    return {'path': output.name, 'size': output.stat().st_size,
            'sha256': hashlib.sha256(output.read_bytes()).hexdigest(), 'hostSha256': HOST_SHA}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    print(json.dumps(build(args.output, args.source)))
