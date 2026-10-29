import {invoke} from '@tauri-apps/api/core';
export type BrowserSnapshot={url:string;title:string;loading:boolean;notice:string;visible:boolean};
export type BrowserBounds={x:number;y:number;width:number;height:number};
export type BrowserRequest={operation:'show';bounds:BrowserBounds}|{operation:'navigate';url:string}|{operation:'zoom';factor:number}|{operation:'hide'|'back'|'forward'|'reload'|'stop'|'status'|'close'};
// Serialize lifecycle with toolbar actions; a late show must not cover the next editor.
let tail:Promise<unknown>=Promise.resolve();
export function browserRequest(request:BrowserRequest):Promise<BrowserSnapshot>{const next=tail.then(()=>invoke<BrowserSnapshot>('research_browser',{request}));tail=next.catch(()=>undefined);return next;}
export function browserAddress(input:string):string{
 const value=input.trim();if(!value||value.length>4096||[...value].some(c=>c.charCodeAt(0)<32||c.charCodeAt(0)===127))throw Error('请输入不超过4096字符的网页地址或搜索词。');
 const explicit=/^[a-z][a-z\d+.-]*:/i.test(value);
 const address=explicit?value:!value.includes(' ')&&value.includes('.')?'https://'+value:'https://www.google.com/search?q='+encodeURIComponent(value);
 const url=new URL(address);if(!['https:','http:'].includes(url.protocol)||url.username||url.password)throw Error('只支持不含账号密码的HTTP或HTTPS网页地址。');return url.href;
}
export function browserBounds(rect:{x:number;y:number;width:number;height:number},dpr:number,viewport?:{width:number;height:number}):BrowserBounds|null{
 if(viewport){const right=Math.min(viewport.width,rect.x+rect.width),bottom=Math.min(viewport.height,rect.y+rect.height);rect={x:Math.max(0,rect.x),y:Math.max(0,rect.y),width:right-Math.max(0,rect.x),height:bottom-Math.max(0,rect.y)};}
 if(![rect.x,rect.y,rect.width,rect.height,dpr].every(Number.isFinite)||dpr<=0||rect.x<0||rect.y<0||rect.width<32||rect.height<32)return null;
 return {x:rect.x*dpr,y:rect.y*dpr,width:rect.width*dpr,height:rect.height*dpr};
}
