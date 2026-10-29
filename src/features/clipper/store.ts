import {create} from 'zustand';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {errorMessage} from '../../shared/api/errors';
export interface SavedClip {documentId:string;title:string;duplicate:boolean}
export interface ClipperStatus {enabled:boolean;extensionId:string;last:SavedClip|null}
interface State {status:ClipperStatus|null;busy:boolean;error:string|null;notice:SavedClip|null;refresh:()=>Promise<void>;connect:(enabled:boolean)=>Promise<void>;receive:(value:unknown)=>void;dismiss:()=>void}
export function parseSavedClip(value:unknown):SavedClip {
 if(!value||typeof value!=='object'||!('documentId'in value)||typeof value.documentId!=='string'||!/^[a-f0-9]{32}$/.test(value.documentId)||!('title'in value)||typeof value.title!=='string'||value.title.length>400||!('duplicate'in value)||typeof value.duplicate!=='boolean')throw Error('剪藏响应无效。');
 return {documentId:value.documentId,title:value.title,duplicate:value.duplicate};
}
export function parseClipperStatus(value:unknown):ClipperStatus {
 if(!value||typeof value!=='object')throw Error('网页剪藏状态无效。');
 const v=value as Record<string,unknown>;
 if(typeof v.enabled!=='boolean'||typeof v.extensionId!=='string'||!/^[a-p]{32}$/.test(v.extensionId)||!(v.last===null||v.last&&typeof v.last==='object'))throw Error('网页剪藏状态无效。');
 return {enabled:v.enabled,extensionId:v.extensionId,last:v.last===null?null:parseSavedClip(v.last)};
}
export async function readClipperStatus(){if(!isTauri())throw Error('网页剪藏需要桌面应用。');return parseClipperStatus(await invoke('clipper_status'));}
export const useClipperStore=create<State>((set,get)=>{let generation=0;return {status:null,busy:false,error:null,notice:null,
 refresh:async()=>{if(!isTauri()||get().busy)return;const current=++generation;try{const status=await readClipperStatus();if(current===generation)set({status,error:null});}catch(e){if(current===generation)set({error:errorMessage(e,'无法读取网页剪藏状态。')});}},
 connect:async(enabled)=>{if(!isTauri()||get().busy)return;const current=++generation;set({busy:true,error:null});try{const status=parseClipperStatus(await invoke('clipper_connection',{enabled}));if(current===generation)set({status,error:null});}catch(e){if(current===generation)set({error:errorMessage(e,'网页剪藏连接失败。')});}finally{if(current===generation)set({busy:false});}},
 receive:value=>{try{set({notice:parseSavedClip(value),error:null});}catch(e){set({error:errorMessage(e,'剪藏通知无效。')});}},dismiss:()=>set({notice:null}),
};});
