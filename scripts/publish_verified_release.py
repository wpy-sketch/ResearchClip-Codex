"""Publish only exact, independently accepted assets under the chosen delivery policy.

No force pushes, tag replacement, --clobber, release deletion or implicit key creation.
"""
import argparse,hashlib,json,os,shutil,subprocess,tempfile,time,urllib.request
from pathlib import Path
from urllib.parse import quote
from release_gate import ROOT,preflight,MANUAL_MODE

def cli():
    value=shutil.which('gh')
    if value:return value
    path=ROOT/'.tools/github-cli/runtime/bin/gh.exe'
    if path.is_file():return str(path)
    raise ValueError('GitHub CLI is unavailable')

def gh(*args,optional=False):
    result=subprocess.run([cli(),*args],cwd=ROOT,capture_output=True,text=True,encoding='utf-8')
    if result.returncode and not optional:raise ValueError('GitHub operation failed: '+args[0]+' (credentials and raw response not logged)')
    return result

def api(path):return json.loads(gh('api',path).stdout)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def public_hash(url,expected):
    for attempt in range(3):
        try:
            h=hashlib.sha256();size=0
            with urllib.request.urlopen(url,timeout=90) as response:
                if not response.geturl().startswith('https://'):raise ValueError('Public asset redirected outside HTTPS')
                for chunk in iter(lambda:response.read(65536),b''):
                    size+=len(chunk)
                    if size>expected['size']:raise ValueError('Published asset exceeds local size')
                    h.update(chunk)
            if size!=expected['size'] or h.hexdigest()!=expected['sha256']:raise ValueError('Published asset digest mismatch')
            return
        except (OSError,ValueError):
            if attempt==2:raise
            time.sleep(2)

def publish(assets,receipt,notes):
    metadata=preflight(assets,receipt);repo=metadata['repository'];tag=metadata['tag'];commit=metadata['sourceCommit']
    manual=metadata.get('deliveryMode')==MANUAL_MODE
    if not manual:subprocess.run(['node',str(ROOT/'scripts/with-rust.mjs'),'cargo','run','--manifest-path','src-tauri/Cargo.toml','--locked','--example','verify_release_signatures','--',str(assets)],cwd=ROOT,check=True)
    access=api('repos/'+repo)
    if access['full_name'].lower()!=repo.lower() or not access.get('permissions',{}).get('push'):raise ValueError('Cannot verify target repository publication permission')
    existing=gh('release','view',tag,'--repo',repo,'--json','id',optional=True)
    if existing.returncode==0:raise ValueError('Release already exists; refusing duplicate or overwrite')
    if 'not found' not in existing.stderr.lower() and 'HTTP 404' not in existing.stderr:raise ValueError('Release absence could not be verified')
    if api('repos/'+repo+'/commits/'+commit)['sha']!=commit:raise ValueError('Verified source commit has not been pushed normally')
    ref=gh('api','repos/'+repo+'/git/ref/tags/'+tag,optional=True)
    if ref.returncode==0:
        if api('repos/'+repo+'/commits/'+tag)['sha']!=commit:raise ValueError('Existing tag points elsewhere; it will not be moved')
    elif 'HTTP 404' in ref.stderr:
        gh('api','--method','POST','repos/'+repo+'/git/refs','-f','ref=refs/tags/'+tag,'-f','sha='+commit)
    else:raise ValueError('Tag absence could not be verified')
    expected={e['name']:e for e in metadata['files']}
    for name in ('release-build.json','SHA256SUMS'):
        p=assets/name;expected[name]={'name':name,'size':p.stat().st_size,'sha256':sha(p)}
    if set(p.name for p in assets.iterdir())!=set(expected):raise ValueError('Unexpected staging files; refusing upload')
    gh('release','create',tag,'--repo',repo,'--verify-tag','--target',commit,'--draft','--title','ResearchClip '+tag,'--notes-file',str(notes))
    created=json.loads(gh('release','view',tag,'--repo',repo,'--json','databaseId,isDraft,tagName,targetCommitish').stdout)
    if created.get('isDraft') is not True or created.get('tagName')!=tag or created.get('targetCommitish')!=commit or type(created.get('databaseId')) is not int:raise ValueError('The just-created draft identity could not be verified; inspect without overwriting')
    release_id=created['databaseId']
    # Only the just-created draft is touched. A retry must inspect the existing draft;
    # it never silently skips, overwrites assets or creates a second release.
    for name in expected:gh('release','upload',tag,str(assets/name),'--repo',repo)
    release=api('repos/'+repo+'/releases/'+str(release_id))
    remote={a['name']:a for a in release['assets']}
    if set(remote)!=set(expected):raise ValueError('Draft asset set differs from verified staging; draft retained')
    with tempfile.TemporaryDirectory(prefix='researchclip-release-readback-') as tmp:
        for name,item in expected.items():
            if remote[name]['size']!=item['size']:raise ValueError('Draft asset size mismatch')
            gh('release','download',tag,'--repo',repo,'--pattern',name,'--dir',tmp)
            if sha(Path(tmp)/name)!=item['sha256']:raise ValueError('Draft readback SHA mismatch; draft retained')
        if not manual:subprocess.run(['node',str(ROOT/'scripts/with-rust.mjs'),'cargo','run','--manifest-path','src-tauri/Cargo.toml','--locked','--example','verify_release_signatures','--',tmp],cwd=ROOT,check=True)
    gh('release','edit',tag,'--repo',repo,'--draft=false','--latest')
    try:
        release=api('repos/'+repo+'/releases/'+str(release_id))
        if release['draft'] or release['prerelease'] or release['tag_name']!=tag:raise ValueError('Release did not enter expected stable state')
        # Draft URLs use an ephemeral untagged-* slug; publication changes them.
        published={a['name']:a for a in release['assets']}
        if set(published)!=set(remote) or any(published[n]['id']!=a['id'] or published[n]['size']!=expected[n]['size'] for n,a in remote.items()):raise ValueError('Published assets differ from the verified draft')
        remote=published
        for name,item in expected.items():public_hash(remote[name]['browser_download_url'],item)
        if not manual:
            trust=json.loads((ROOT/'src-tauri/resources/package-trust.json').read_text(encoding='utf-8'))
            for key,name in [('catalogUrl','catalog.json'),('updaterUrl','latest.json')]:public_hash(trust[key],expected[name])
    except Exception:
        # Revert only this newly-created release to a draft, never delete releases/assets.
        gh('release','edit',tag,'--repo',repo,'--draft=true','--latest=false',optional=True)
        raise ValueError('Public verification failed; attempted to return this new release to draft. Inspect actual GitHub state; publication is NOT verified.') from None
    report={'result':'PUBLISHED_AND_PUBLIC_HASHES_VERIFIED','repository':'https://github.com/'+repo,'version':metadata['version'],'tag':tag,'releaseUrl':release['html_url'],'installerUrl':remote[f'ResearchClip_{metadata["version"]}_x64-setup.exe']['browser_download_url'],'assets':[{**e,'url':remote[n]['browser_download_url']} for n,e in expected.items()],'updater':'DISABLED_MANUAL_DOWNLOAD' if manual else 'SIGNED_VERSION_AND_PUBLIC_METADATA_VERIFIED'}
    (ROOT/'artifacts/final-publication.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,ensure_ascii=False))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--assets',type=Path,required=True);p.add_argument('--receipt',type=Path,required=True);p.add_argument('--notes',type=Path,default=ROOT/'docs/release/RELEASE_NOTES.md');a=p.parse_args()
    try:publish(a.assets.resolve(),a.receipt.resolve(),a.notes.resolve())
    except (ValueError,OSError,subprocess.CalledProcessError) as error:p.exit(1,str(error)+'\n')
