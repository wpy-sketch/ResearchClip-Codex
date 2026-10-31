"""Controlled GitHub-side transfer of the two already-reviewed large model files.
Never publishes, deletes or replaces assets. Local release gate remains authoritative.
"""
import hashlib,json,os,subprocess,tempfile
from pathlib import Path
from release_gate import ROOT,git,repository,version
from build_signed_release import download

ALLOWED={'vision.onnx','text.onnx'}

def reviewed_models(catalog,metadata,repo,tag,commit):
    if metadata.get('repository')!=repo or metadata.get('tag')!=tag or metadata.get('sourceCommit')!=commit:
        raise ValueError('Draft metadata differs from immutable source tag')
    packages=[p for p in catalog['packages'] if p['id']=='model.semantic']
    if len(packages)!=1:raise ValueError('Expected reviewed semantic package')
    files=[f for f in packages[0]['files'] if f['path'] in ALLOWED]
    if len(files)!=2 or {f['path'] for f in files}!=ALLOWED:raise ValueError('Only both reviewed large models may transfer')
    expected={f['name']:f for f in metadata['files']}
    for f in files:
        e=expected[f['path']]
        if e['size']!=f['size'] or e['sha256']!=f['sha256'] or not f['url'].startswith('https://huggingface.co/'):
            raise ValueError('Compiled model differs from exact staged asset')
    return files

def main():
    repo=repository(git('remote','get-url','origin'))
    if repo!='wpy-sketch/ResearchClip-Codex' or os.environ.get('GITHUB_REPOSITORY')!=repo:
        raise ValueError('This controlled transfer is restricted to the verified repository')
    tag='v'+version();commit=git('rev-parse',tag+'^{commit}')
    def gh(*args):
        r=subprocess.run(['gh',*args],capture_output=True,text=True,encoding='utf8')
        if r.returncode:raise ValueError('Official GitHub operation failed; no asset overwritten')
        return r.stdout
    def release():
        identity=json.loads(gh('release','view',tag,'--repo',repo,'--json','databaseId'))
        value=json.loads(gh('api','repos/'+repo+'/releases/'+str(identity['databaseId'])))
        return {'isDraft':value['draft'],'targetCommitish':value['target_commitish'],'assets':value['assets']}
    r=release()
    if not r['isDraft'] or r['targetCommitish']!=commit:raise ValueError('Only the exact existing draft is eligible')
    with tempfile.TemporaryDirectory(prefix='researchclip-reviewed-models-') as temp:
        out=Path(temp);gh('release','download',tag,'--repo',repo,'--pattern','release-build.json','--dir',temp)
        metadata=json.loads((out/'release-build.json').read_text(encoding='utf8'))
        catalog=json.loads(git('show',tag+':src-tauri/resources/package-builtin.json'))
        for f in reviewed_models(catalog,metadata,repo,tag,commit):
            current=[a for a in release()['assets'] if a['name']==f['path']]
            if current:
                a=current[0]
                if a['state']!='uploaded' or a['size']!=f['size'] or a.get('digest')!='sha256:'+f['sha256']:
                    raise ValueError('An existing asset requires inspection; refusing replacement')
                print('EXISTING_EXACT',f['path'],flush=True);continue
            dest=out/f['path'];download(f,dest)
            if dest.stat().st_size!=f['size'] or hashlib.sha256(dest.read_bytes()).hexdigest()!=f['sha256']:
                raise ValueError('Model hash mismatch')
            # Server enforces unique names; a concurrent local upload is never clobbered.
            gh('release','upload',tag,str(dest),'--repo',repo)
            a=next(a for a in release()['assets'] if a['name']==f['path'])
            if a['size']!=f['size'] or a.get('digest')!='sha256:'+f['sha256']:
                raise ValueError('Transferred asset verification failed')
            print('TRANSFER_VERIFIED',f['path'],flush=True)
if __name__=='__main__':main()
