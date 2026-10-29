import {useEffect,useRef,useState,type FormEvent} from 'react';
import {ArrowLeft,ArrowRight,RotateCw,House,ExternalLink,X,Globe,Search} from 'lucide-react';
import {browserAddress,browserBounds,browserRequest,type BrowserSnapshot} from './browser';
const initial:BrowserSnapshot={url:'',title:'Google',loading:false,notice:'',visible:false};
export function ResearchBrowser({openExternal,back}:{openExternal:()=>Promise<void>;back:()=>void}){
 const host=useRef<HTMLDivElement>(null),[snapshot,setSnapshot]=useState(initial),[address,setAddress]=useState('https://www.google.com/'),[error,setError]=useState(''),[busy,setBusy]=useState(false),[zoom,setZoom]=useState(1),[reloadKey,setReloadKey]=useState(0);
 const editing=useRef(false);
 useEffect(()=>{let stopped=false,pending=false,last='',shown=false;const node=host.current;if(!node)return;
  const update=async()=>{if(stopped||pending)return;pending=true;try{
   const modal=!!document.querySelector('[role="dialog"],dialog[open]');
   const panel=node.closest('.research-panel')?.getBoundingClientRect();
   const bounds=!modal&&document.visibilityState==='visible'?browserBounds(node.getBoundingClientRect(),window.devicePixelRatio,{width:Math.min(innerWidth,panel?.right??innerWidth),height:Math.min(innerHeight,panel?.bottom??innerHeight)}):null;
   const key=bounds?JSON.stringify(bounds):'';
   if(!bounds){if(shown){await browserRequest({operation:'hide'});shown=false;last='';}return;}
   if(key!==last||!shown){await browserRequest({operation:'show',bounds});shown=true;last=key;if(!stopped)setError('');}
   const state=await browserRequest({operation:'status'});if(!stopped){setSnapshot(state);if(!editing.current&&state.url)setAddress(state.url);}
  }catch(e){if(!stopped)setError(e instanceof Error?e.message:typeof e==='object'&&e&&'message'in e?String(e.message):'网页区域不可用，请重试。');}finally{pending=false;}};
  const observer=new ResizeObserver(()=>void update());observer.observe(node);const timer=window.setInterval(()=>void update(),500);void update();
  return()=>{stopped=true;observer.disconnect();clearInterval(timer);void browserRequest({operation:'hide'}).catch(e=>console.error('Browser hide failed',e instanceof Error?e.message:'native lifecycle error'));};
 },[reloadKey]);
 async function action(request:Parameters<typeof browserRequest>[0]){setBusy(true);setError('');try{const next=await browserRequest(request);setSnapshot(next);}catch(e){setError(e instanceof Error?e.message:typeof e==='object'&&e&&'message'in e?String(e.message):'网页操作失败。');}finally{setBusy(false);}}
 function navigate(e:FormEvent){e.preventDefault();try{const url=browserAddress(address);editing.current=false;setAddress(url);void action({operation:'navigate',url});}catch(e){setError(e instanceof Error?e.message:'地址无效。');}}
 return <section className="research-browser" aria-label="应用内Google浏览器">
  <header className="research-browser-title"><Globe size={16}/><strong title={snapshot.title}>{snapshot.title||'Google · 科研浏览器'}</strong><button className="icon-button" title="返回资料工作台，保留网页" aria-label="返回资料工作台" onClick={back}><X size={16}/></button></header>
  <form className="research-browser-toolbar" aria-label="网页导航" onSubmit={navigate}>
   <div className="research-browser-navigation"><button type="button" className="icon-button" aria-label="网页后退" disabled={busy} onClick={()=>void action({operation:'back'})}><ArrowLeft size={16}/></button><button type="button" className="icon-button" aria-label="网页前进" disabled={busy} onClick={()=>void action({operation:'forward'})}><ArrowRight size={16}/></button><button type="button" className="icon-button" aria-label={snapshot.loading?'停止加载网页':'刷新网页'} disabled={busy} onClick={()=>void action({operation:snapshot.loading?'stop':'reload'})}>{snapshot.loading?<X size={16}/>:<RotateCw size={16}/>}</button><button type="button" className="icon-button" aria-label="Google首页" disabled={busy} onClick={()=>void action({operation:'navigate',url:'https://www.google.com/'})}><House size={16}/></button></div>
   <label className="research-browser-address"><Search size={16}/><input aria-label="网页地址或Google搜索" value={address} onFocus={()=>{editing.current=true;}} onBlur={()=>{editing.current=false;}} onChange={e=>setAddress(e.target.value)} placeholder="输入网址或搜索内容" autoComplete="off" spellCheck={false}/></label><button type="submit" className="secondary-button" disabled={busy}>前往</button>
   <label className="research-browser-zoom"><span>网页</span><select aria-label="网页缩放" value={zoom} onChange={e=>{const value=Number(e.target.value);setZoom(value);void action({operation:'zoom',factor:value});}}>{[.5,.75,1,1.25,1.5,2].map(n=><option key={n} value={n}>{n*100}%</option>)}</select></label>
   <button type="button" className="icon-button" title="使用真正Chrome打开Google（账号与书签由Chrome管理）" aria-label="使用Chrome打开Google" disabled={busy} onClick={()=>{setBusy(true);setError('');void openExternal().catch(e=>setError(e instanceof Error?e.message:typeof e==='object'&&e&&'message'in e?String(e.message):'无法启动Chrome。')).finally(()=>setBusy(false));}}><ExternalLink size={16}/></button>
  </form>
  {(error||snapshot.notice)&&<p role="alert" className="research-browser-message">{error||snapshot.notice}<button onClick={()=>{setError('');setReloadKey(n=>n+1);}}>重试区域</button></p>}
  <div className="research-browser-status" role="status">{snapshot.loading?'正在加载网页，可停止或更换地址。':'独立WebView2浏览区域 · 返回工作台保留PDF与Markdown。'}</div>
  <div ref={host} className="research-browser-viewport" aria-label="原生网页显示区域"><p>正在准备应用内网页区域…</p></div>
  <footer>网页内容来自外部网站。此处不提供Chrome账号同步；登录限制或文件下载请使用外部Chrome。</footer>
 </section>;
}
