export interface BrowserBookmark {title:string;url:string}
const controlled=(s:string)=>[...s].some(c=>c.charCodeAt(0)<32||(c.charCodeAt(0)>=127&&c.charCodeAt(0)<=159));
export function normalizeBookmark(title:string,address:string):BrowserBookmark{
 const name=title.trim(),raw=address.trim();
 if(!name||name.length>80||controlled(name)||!raw||controlled(raw))throw Error('书签名称需为1–80字符，地址不能包含控制字符。');
 let url:URL;try{url=new URL(/^[a-z][a-z\d+.-]*:/i.test(raw)?raw:'https://'+raw);}catch{throw Error('请输入完整HTTP(S)网址，例如https://example.com/。');}
 const host=url.hostname.toLowerCase().replace(/\.$/,'');
 const ip=/^\d+\.\d+\.\d+\.\d+$/.test(host)?host.split('.').map(Number):null;
 if(!['https:','http:'].includes(url.protocol)||url.username||url.password||!host.includes('.')||host.endsWith('.localhost')||host.endsWith('.local')||host.endsWith('.internal')||host.includes(':')||(ip&&(ip[0]===0||ip[0]===10||ip[0]===127||ip[0]>=224||(ip[0]===169&&ip[1]===254)||(ip[0]===172&&ip[1]>=16&&ip[1]<=31)||(ip[0]===192&&ip[1]===168))))throw Error('书签只支持不含凭据的公开HTTP(S)网址。');
 if(new TextEncoder().encode(url.href).length>2048)throw Error('书签地址不得超过2048字节。');
 return {title:name,url:url.href};
}
export function parseBookmarks(value:unknown):BrowserBookmark[]{
 if(!Array.isArray(value)||value.length>32)throw Error('网页快捷方式最多32个。');
 const seen=new Set<string>();return value.map(item=>{if(!item||typeof item!=='object'||typeof item.title!=='string'||typeof item.url!=='string'||Object.keys(item).some(k=>!['title','url'].includes(k)))throw Error('书签数据无效。');const b=normalizeBookmark(item.title,item.url);if(seen.has(b.url))throw Error('这个网页已在快捷方式中。');seen.add(b.url);return b;});
}
