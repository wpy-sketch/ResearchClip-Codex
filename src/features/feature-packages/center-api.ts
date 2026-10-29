import {invoke,isTauri} from '@tauri-apps/api/core';
export interface PackageAsset {path:string;url:string;size:number;sha256:string;entries:{path:string;size:number;sha256:string}[]}
export interface InstallablePackage {id:string;name:string;description:string;version:string;publisher:string;license:string;licenseUrl:string;source:string;kind:string;appVersion:string;platforms:string[];dependencies:string[];adapter:string;files:PackageAsset[]}
export interface InstalledPackage {package:InstallablePackage;directory:string;enabled:boolean;readiness:string;installedAt:number}
export interface PackageCenterSnapshot {catalogMode?:'BUILTIN'|'SIGNED_REMOTE'|'UNPUBLISHED';published:boolean;catalog:{schemaVersion:number;sequence:number;expiresAt:number;packages:InstallablePackage[]}|null;registry:{schemaVersion:number;highestSequence:number;installed:InstalledPackage[]};job:{generation?:number;phase:string;packageId:string|null;downloaded:number;total:number;error:string|null};error:string|null}
export interface UpdateSnapshot {sha256?:string|null;configured:boolean;phase:string;version:string|null;notes:string|null;downloaded:number;total:number|null;error:string|null}
type PackageRequest={operation:'status'|'refresh'|'cancel'}|{operation:'install'|'uninstall';id:string;confirmed:boolean}|{operation:'enable'|'disable'|'export'|'source';id:string};
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
export function parseCenter(v:unknown):PackageCenterSnapshot {
 if(object(v)&&v.catalogMode!==undefined&&!['BUILTIN','SIGNED_REMOTE','UNPUBLISHED'].includes(String(v.catalogMode)))throw Error('组件清单来源模式无效。');
 if(!object(v)||typeof v.published!=='boolean'||!object(v.registry)||v.registry.schemaVersion!==1||!Array.isArray(v.registry.installed)||v.registry.installed.length>100||!(v.catalog===null||object(v.catalog)&&v.catalog.schemaVersion===1&&Array.isArray(v.catalog.packages)&&v.catalog.packages.length<=100)||!object(v.job)||typeof v.job.phase!=='string'||!Number.isSafeInteger(v.job.downloaded)||Number(v.job.downloaded)<0||!Number.isSafeInteger(v.job.total)||Number(v.job.total)<0||!(v.error===null||typeof v.error==='string'))throw Error('组件中心响应无效。');
 for(const entry of v.registry.installed){if(!object(entry)||!validPackage(entry.package)||typeof entry.enabled!=='boolean'||typeof entry.readiness!=='string')throw Error('已安装组件响应无效。');}
 if(v.catalog!==null)for(const p of (v.catalog as {packages:unknown[]}).packages)if(!validPackage(p))throw Error('目录组件响应无效。');
 return v as unknown as PackageCenterSnapshot;
}
function validPackage(v:unknown):v is InstallablePackage {return object(v)&&['id','name','description','version','publisher','license','licenseUrl','source','kind','appVersion','adapter'].every(k=>typeof v[k]==='string')&&Array.isArray(v.dependencies)&&v.dependencies.every(d=>typeof d==='string')&&Array.isArray(v.platforms)&&v.platforms.every(d=>typeof d==='string')&&Array.isArray(v.files)&&v.files.every(f=>object(f)&&typeof f.path==='string'&&typeof f.sha256==='string'&&Number.isSafeInteger(f.size)&&Number(f.size)>0);}
export async function packageRequest(request:PackageRequest){if(!isTauri())throw Error('扩展安装需要Windows桌面应用。');return parseCenter(await invoke('package_center',{request}));}
export function parseUpdate(v:unknown):UpdateSnapshot {if(!object(v)||typeof v.configured!=='boolean'||typeof v.phase!=='string'||!Number.isSafeInteger(v.downloaded)||Number(v.downloaded)<0||![v.version,v.notes,v.error].every(x=>x===null||typeof x==='string'))throw Error('更新器响应无效。');return v as unknown as UpdateSnapshot;}
export async function updateRequest(operation:'status'|'check'|'download'|'cancel'|'install',confirmed=false){return parseUpdate(await invoke('app_update',{request:operation==='install'?{operation,confirmed}:{operation}}));}
export function newer(candidate:string,current:string){
 const parse=(v:string)=>{const m=v.match(/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*))?(?:\+[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?$/);if(!m)return null;const pre=m[4]?.split('.')??[];if(pre.some(p=>/^\d+$/.test(p)&&p.length>1&&p.startsWith('0')))return null;return {core:m.slice(1,4).map(n=>BigInt(n)),pre};};
 const a=parse(candidate),b=parse(current);if(!a||!b)return false;for(let i=0;i<3;i++){if(a.core[i]!==b.core[i])return a.core[i]>b.core[i];}if(!a.pre.length||!b.pre.length)return !a.pre.length&&!!b.pre.length;
 for(let i=0;i<Math.max(a.pre.length,b.pre.length);i++){if(a.pre[i]===undefined)return false;if(b.pre[i]===undefined)return true;const x=a.pre[i],y=b.pre[i];if(x===y)continue;const xn=/^\d+$/.test(x),yn=/^\d+$/.test(y);if(xn&&yn)return BigInt(x)>BigInt(y);if(xn!==yn)return !xn;return x>y;}return false;
}
