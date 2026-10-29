"""Release-only guard: real independent Zotero acceptance must match the exact original XPI.

Unit fixtures validate the guard only; they never create the real acceptance evidence.
"""
import hashlib,json,zipfile
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
def current_versions():
    return (json.loads((ROOT/'package.json').read_text(encoding='utf-8'))['version'],json.loads((ROOT/'zotero-connector/manifest.json').read_text(encoding='utf-8'))['version'])

REQUIRED_CHECKS={'installed','contextMenu','scopedIdentity','existingInstance','pageAndAnnotation','reverseFocus','notifications','cancelReconnect','pdfHashUnchanged','pendingEditsPreserved'}

def validate_package_manifest(manifest,repository):
    """Fail packaging before Zotero rejects an incomplete host manifest."""
    import re
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository):
        raise ValueError('Invalid verified origin repository')
    host=manifest.get('applications',{}).get('zotero',{})
    expected={'id':'researchclip-connector@wpy-sketch.github.io',
              'update_url':f'https://github.com/{repository}/releases/latest/download/zotero-updates.json',
              'strict_min_version':'10.0.6','strict_max_version':'10.0.6'}
    if any(host.get(name)!=value for name,value in expected.items()):
        raise ValueError('Zotero manifest requires verified identity, update URL and tested host bounds')
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+',str(manifest.get('version',''))):
        raise ValueError('Invalid connector version')

def validate_native_receipt(receipt,metadata):
    app_version,plugin_version=current_versions()
    if not isinstance(receipt,dict) or receipt.get('result')!='PASS' or receipt.get('mode')!='actual-independent-zotero' or receipt.get('appVersion')!=app_version or receipt.get('zoteroVersion')!='10.0.6' or receipt.get('pluginVersion')!=plugin_version or receipt.get('isolatedProfile') is not True or receipt.get('xpiSha256')!=metadata['sha256']:
        raise ValueError('Actual independent Zotero10.0.6 acceptance must match the current application and exact XPI before publication')
    checks=receipt.get('checks')
    if not isinstance(checks,dict) or not REQUIRED_CHECKS.issubset(checks) or any(checks[name] is not True for name in REQUIRED_CHECKS):
        raise ValueError('Real Zotero installation/menu/pairing/location/notification/cancellation acceptance is incomplete')

def checked_xpi(root):
    root=Path(root).resolve()
    app_version,plugin_version=current_versions()
    marker=root/'docs/qa/evidence'/('v'+app_version.replace('.','')+'-zotero-connector-native.json')
    if not marker.is_file():
        raise ValueError('User requires desktop unlock and real independent Zotero acceptance first; this version is not publishable')
    metadata=json.loads((root/'artifacts/zotero-connector-build/latest.json').read_text(encoding='utf-8'))
    path=(root/metadata['file']).resolve()
    if not path.is_relative_to((root/'artifacts/zotero-connector-build').resolve()) or path.name!=f'ResearchClip-Zotero-Connector-v{plugin_version}.xpi' or metadata.get('version')!=plugin_version or metadata.get('pluginId')!='researchclip-connector@wpy-sketch.github.io' or path.stat().st_size!=metadata['size'] or hashlib.sha256(path.read_bytes()).hexdigest()!=metadata['sha256']:
        raise ValueError('XPI source, size or digest changed; rebuild and repeat actual acceptance')
    validate_native_receipt(json.loads(marker.read_text(encoding='utf-8')),metadata)
    members={'manifest.json','bootstrap.js','connector.js','LICENSE','README.md'}
    with zipfile.ZipFile(path) as archive:
        if set(archive.namelist())!=members or archive.testzip() is not None:
            raise ValueError('XPI members/integrity differ from original connector package')
        for name in members:
            if archive.read(name)!=(root/'zotero-connector'/name).read_bytes():
                raise ValueError('Current connector source differs from the accepted XPI')
    return path


def update_manifest(metadata,repository,tag):
    """Prepare a genuine candidate asset; not evidence that its URL is published."""
    import re
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository) or not re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+',tag):
        raise ValueError('Unverified repository or release tag')
    version=metadata['version'];name=f'ResearchClip-Zotero-Connector-v{version}.xpi'
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+',version) or not re.fullmatch(r'[a-f0-9]{64}',metadata['sha256']):raise ValueError('Invalid connector metadata')
    return {'addons':{metadata['pluginId']:{'updates':[{'version':version,'update_link':f'https://github.com/{repository}/releases/download/{tag}/{name}','update_hash':'sha256:'+metadata['sha256'],'applications':{'zotero':{'strict_min_version':'10.0.6','strict_max_version':'10.0.6'}}}]}}}
