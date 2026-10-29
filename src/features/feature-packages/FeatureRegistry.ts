import raw from './catalog.json';
import {featureKinds,implementations,probeNames,type FeatureDefinition,type FeatureKind,type Implementation,type ProbeName,type Control} from './types';
/** App capabilities, not resource identities, editor registration or an executable plugin loader. */
export class FeatureRegistry {
 private entries=new Map<string,FeatureDefinition>();
 constructor(definitions:readonly FeatureDefinition[]){
  for(const d of definitions){
   if(!/^[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*$/.test(d.id)||this.entries.has(d.id)||!d.name||!featureKinds.includes(d.kind)||!implementations.includes(d.implementation)||!probeNames.includes(d.probe)||!d.version||!d.compatibleAppVersions.length||!d.sourceFiles.length||new Set(d.dependencies).size!==d.dependencies.length||d.control!==null&&(!['clipper','connector'].includes(d.control)||d.kind!=='BUILTIN_OPTIONAL'))throw Error('功能定义重复或无效：'+d.id);
   this.entries.set(d.id,Object.freeze({...d,dependencies:Object.freeze([...d.dependencies]),sourceFiles:Object.freeze([...d.sourceFiles]),compatibleAppVersions:Object.freeze([...d.compatibleAppVersions])}));
  }
  const complete=new Set<string>(),active=new Set<string>();const visit=(id:string)=>{if(active.has(id))throw Error('功能依赖存在循环。');if(complete.has(id))return;const d=this.get(id);active.add(id);for(const dep of d.dependencies)visit(dep);active.delete(id);complete.add(id);};for(const id of this.entries.keys())visit(id);
 }
 get(id:string){const d=this.entries.get(id);if(!d)throw Error('未知功能ID：'+id);return d;}
 list(){return [...this.entries.values()];}
}
export function createFeatureRegistry(appVersion:string){
 if(!/^\d+\.\d+\.\d+(?:[-+][a-zA-Z0-9.-]+)?$/.test(appVersion))throw Error('应用版本无效。');
 return new FeatureRegistry(raw.map(d=>({...d,version:d.version==='@app'?appVersion:d.version,compatibleAppVersions:[appVersion],kind:d.kind as FeatureKind,implementation:d.implementation as Implementation,probe:d.probe as ProbeName,control:d.control as Control|null})));
}
export const featureRegistry=createFeatureRegistry(__APP_VERSION__);
