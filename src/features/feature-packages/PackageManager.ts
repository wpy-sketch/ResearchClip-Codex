import type {Activation,Availability,Control,FeatureContext,FeatureState,IssueCode,Observation,PackageSnapshot,Presence,ProbeName} from './types';
import {FeatureRegistry} from './FeatureRegistry';
export type Probes=Readonly<Partial<Record<ProbeName,()=>Promise<Observation>>>>;
export type Controls=Readonly<Partial<Record<Control,(enabled:boolean)=>Promise<void>>>>;
export function observed(installed:Presence,enabled:Activation,availability:Availability,code?:IssueCode,message='',installedVersion:string|null=null):Observation{return {installed,enabled,availability,installedVersion,issues:code?[{code,message}]:[]};}
const unknown=()=>observed('UNKNOWN','UNKNOWN','UNVERIFIED','NOT_CHECKED','尚未检查本机状态。');
/** Read-only status aggregation; only two pre-existing native bridge toggles are delegated. */
export class PackageManager {
 private observations=new Map<ProbeName,Observation>();private generation=0;private changing=false;private disposed=false;
 private context:FeatureContext;private current:PackageSnapshot;
 constructor(readonly registry:FeatureRegistry,private probes:Probes,private controls:Controls,context:FeatureContext){this.context=context;this.current=this.resolve(null);}
 snapshot(){return this.current;}
 private resolve(checkedAt:number|null):PackageSnapshot{
  const resolved=new Map<string,FeatureState>();const row=(id:string):FeatureState=>{const old=resolved.get(id);if(old)return old;const definition=this.registry.get(id);let value:Observation;
   if(!definition.compatibleAppVersions.includes(this.context.appVersion))value=observed('UNKNOWN','NO','BLOCKED','APP_VERSION_UNSUPPORTED','此功能清单不匹配当前原生应用，请升级整个主程序。');
   else if(definition.implementation==='NOT_IMPLEMENTED'||definition.implementation==='BROKEN')value=observed('NO','NO','BLOCKED','APP_UPDATE_REQUIRED','当前程序未提供可用实现，不能用资源下载补齐原生代码。');
   else if(!this.context.desktop)value=observed('UNKNOWN','UNKNOWN','UNVERIFIED','DESKTOP_REQUIRED','浏览器不能确认本机安装或原生权限。');
   else if(definition.probe==='native')value=observed('YES','YES','READY',undefined,'',this.context.appVersion);
   else value=this.observations.get(definition.probe)??unknown();
   const issues=[...value.issues];let availability=value.availability;
   for(const dep of definition.dependencies){const state=row(dep);if(state.installed==='NO'||state.enabled==='NO'||state.availability==='BLOCKED'){availability='BLOCKED';issues.push({code:'DEPENDENCY_MISSING',message:'依赖不可用：'+state.definition.name});}else if(state.installed==='UNKNOWN'||state.availability==='UNVERIFIED'){if(availability==='READY')availability='UNVERIFIED';issues.push({code:'DEPENDENCY_UNVERIFIED',message:'依赖尚未核验：'+state.definition.name});}}
   if(definition.implementation==='PARTIAL'){if(availability==='READY')availability='UNVERIFIED';issues.push({code:'HOST_VERIFICATION_REQUIRED',message:definition.verificationNote||'仍有功能或环境验收未完成。'});}
   const result={...value,availability,issues,definition,checkedAt};resolved.set(id,result);return result;
  };return {features:this.registry.list().map(d=>row(d.id)),checkedAt};
 }
 private validate(value:Observation){
  if(!value||!['YES','NO','UNKNOWN'].includes(value.installed)||!['YES','NO','UNKNOWN','NOT_APPLICABLE'].includes(value.enabled)||!['READY','BLOCKED','UNVERIFIED'].includes(value.availability)||!(value.installedVersion===null||typeof value.installedVersion==='string'&&value.installedVersion.length<=160)||!Array.isArray(value.issues)||value.issues.length>20||value.issues.some(i=>!i||typeof i.code!=='string'||typeof i.message!=='string'||i.message.length>1000)||value.availability==='READY'&&(value.installed!=='YES'||value.enabled==='NO'||value.enabled==='UNKNOWN'))throw Error('组件状态响应无效。');return value;
 }
 async refresh(context=this.context){if(this.disposed)return this.current;this.context=context;const generation=++this.generation;
  if(context.desktop){const names=[...new Set(this.registry.list().filter(d=>d.compatibleAppVersions.includes(context.appVersion)).map(d=>d.probe).filter(p=>p!=='native'&&p!=='unavailable'))];const results=await Promise.all(names.map(async name=>{try{const probe=this.probes[name];return [name,probe?this.validate(await probe()):unknown()] as const;}catch{return [name,observed('UNKNOWN','UNKNOWN','UNVERIFIED','PROBE_FAILED','本机组件状态读取失败；原配置未修改，请在原模块查看诊断。')] as const;}}));if(this.disposed||generation!==this.generation)return this.current;this.observations=new Map(results);}
  if(!this.disposed&&generation===this.generation)this.current=this.resolve(Date.now());return this.current;
 }
 async setEnabled(id:string,enabled:boolean,context=this.context){const d=this.registry.get(id);if(typeof enabled!=='boolean'||!context.desktop||!d.compatibleAppVersions.includes(context.appVersion)||!d.control||d.kind!=='BUILTIN_OPTIONAL'||!this.controls[d.control])throw Error('此功能不能在组件清单中启停，请使用原设置或升级主程序。');if(this.disposed||this.changing)throw Error('组件操作仍在处理。');this.changing=true;this.cancel();try{await this.controls[d.control]!(enabled);await this.refresh(context);if(this.current.features.find(r=>r.definition.id===id)?.enabled!==(enabled?'YES':'NO'))throw Error('原生端未确认开关结果，未标记成功。');return this.current;}finally{this.changing=false;}}
 cancel(){this.generation++;}
 dispose(){this.disposed=true;this.cancel();}
}
