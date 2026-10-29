import {useState} from 'react';
import {Star,Settings2,ArrowUp,ArrowDown} from 'lucide-react';
import {Modal} from '../../shared/ui/Modal';
import {useSettingsStore} from '../settings/store';
import {normalizeBookmark,parseBookmarks,type BrowserBookmark} from './bookmarks';
export function BrowserBookmarks({url,title,open}:{url:string;title:string;open:(url:string)=>void}){
 const values=useSettingsStore(s=>s.snapshot.preferences.browserBookmarks),phase=useSettingsStore(s=>s.phase);
 const [mode,setMode]=useState<'add'|'manage'|null>(null),[editing,setEditing]=useState<string|null>(null),[name,setName]=useState(''),[address,setAddress]=useState(''),[error,setError]=useState(''),[pending,setPending]=useState(false),[removing,setRemoving]=useState<string|null>(null);
 const busy=pending||phase!=='idle';
 async function persist(next:BrowserBookmark[]){if(busy||useSettingsStore.getState().phase!=='idle')return false;setPending(true);setError('');try{const parsed=parseBookmarks(next);await useSettingsStore.getState().change({browserBookmarks:parsed});const failure=useSettingsStore.getState().error;if(failure)throw Error(failure);return true;}catch(e){setError(e instanceof Error?e.message:'书签保存失败。');return false;}finally{setPending(false);}}
 function add(){setEditing(null);setName((title||'Google').slice(0,80));setAddress(url||'https://www.google.com/');setError('');setMode('add');}
 function edit(value:BrowserBookmark){setEditing(value.url);setName(value.title);setAddress(value.url);setError('');setMode('add');}
 async function save(){try{const value=normalizeBookmark(name,address);const next=editing?values.map(b=>b.url===editing?value:b):[...values,value];if(await persist(next))setMode(null);}catch(e){setError(e instanceof Error?e.message:'书签无效。');}}
 async function move(index:number,offset:number){const next=[...values];[next[index],next[index+offset]]=[next[index+offset],next[index]];await persist(next);}
 return <>
  <div className="browser-bookmarks" aria-label="网页快捷方式"><button className="icon-button" aria-label="保存当前网页为书签" title="保存当前网页为书签" disabled={busy||values.length>=32} onClick={add}><Star size={15}/></button><div className="browser-bookmark-links">{values.map(b=><button className="browser-bookmark-link" key={b.url} title={b.url} onClick={()=>open(b.url)}>{b.title}</button>)}{!values.length&&<span>保存常用网页后，可在此一键打开。</span>}</div><button className="icon-button" aria-label="管理网页快捷方式" title="管理网页快捷方式" onClick={()=>{setError('');setRemoving(null);setMode('manage');}}><Settings2 size={15}/></button></div>
  {mode&&<Modal fitViewport title={mode==='add'?(editing?'编辑网页书签':'保存网页快捷方式'):'管理网页快捷方式'} onClose={()=>{if(!pending)setMode(null);}}><div className="browser-bookmark-dialog">
   <p>仅保存在本机设置中，最多32个。不会保存网页正文、密码或Chrome同步状态。</p>{error&&<p role="alert">{error}</p>}
   {mode==='add'?<form onSubmit={e=>{e.preventDefault();void save();}}><label>名称<input aria-label="书签名称" value={name} maxLength={80} onChange={e=>setName(e.target.value)}/></label><label>网址<input aria-label="书签网址" value={address} maxLength={4096} onChange={e=>setAddress(e.target.value)} spellCheck={false}/></label><div><button type="button" disabled={pending} onClick={()=>setMode(null)}>取消</button><button type="submit" className="primary-button" disabled={busy}>{pending?'正在保存…':'保存书签'}</button></div></form>:<><ol>{values.map((b,i)=><li key={b.url}><div><strong>{b.title}</strong><span>{b.url}</span></div><div><button className="icon-button" aria-label={`上移书签 ${b.title}`} disabled={busy||i===0} onClick={()=>void move(i,-1)}><ArrowUp size={14}/></button><button className="icon-button" aria-label={`下移书签 ${b.title}`} disabled={busy||i===values.length-1} onClick={()=>void move(i,1)}><ArrowDown size={14}/></button><button disabled={busy} aria-label={`编辑书签 ${b.title}`} onClick={()=>edit(b)}>编辑</button><button disabled={busy} aria-label={`移除书签 ${b.title}`} onClick={()=>setRemoving(b.url)}>移除</button></div>{removing===b.url&&<div role="group" aria-label={`确认移除书签 ${b.title}`}><span>仅移除本机快捷方式，是否继续？</span><button disabled={busy} onClick={()=>setRemoving(null)}>取消移除</button><button disabled={busy} onClick={()=>{void persist(values.filter(v=>v.url!==b.url)).then(ok=>{if(ok)setRemoving(null);});}}>确认移除</button></div>}</li>)}</ol>{!values.length&&<p>尚无网页快捷方式。</p>}<button disabled={busy||values.length>=32} onClick={add}>添加网页</button></>}
  </div></Modal>}
 </>;
}
