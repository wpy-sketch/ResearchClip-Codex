import {describe,it,expect,vi} from 'vitest';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn(async()=>({visible:true}))}));
import {invoke} from '@tauri-apps/api/core';
import {browserAddress,browserBounds,browserRequest} from './browser';
describe('isolated research browser',()=>{
 it('normalizes explicit addresses and encoded searches',()=>{expect(browserAddress(' example.org/paper ')).toBe('https://example.org/paper');expect(browserAddress('矩阵 a+b')).toBe('https://www.google.com/search?q=%E7%9F%A9%E9%98%B5%20a%2Bb');expect(browserAddress('矩阵 a+b','bing')).toBe('https://cn.bing.com/search?q=%E7%9F%A9%E9%98%B5%20a%2Bb');expect(browserAddress('https://example.com/a?q=1','bing')).toBe('https://example.com/a?q=1');});
 it('rejects scripts, credentials, empty and control input',()=>{for(const s of ['javascript:alert(1)','file:///C:/a','data:text/html,x','https://user:pass@example.com','','a\nb'])expect(()=>browserAddress(s)).toThrow();});
 it('maps CSS zoom/DPI and refuses hidden invalid bounds',()=>{expect(browserBounds({x:20,y:100,width:400,height:300},1.5)).toEqual({x:30,y:150,width:600,height:450});for(const r of [{x:-1,y:0,width:40,height:40},{x:0,y:0,width:0,height:0},{x:0,y:NaN,width:40,height:40}])expect(browserBounds(r,1)).toBeNull();});
 it('serializes native requests across failure and cleanup',async()=>{vi.mocked(invoke).mockRejectedValueOnce(Error('offline'));await expect(browserRequest({operation:'status'})).rejects.toThrow('offline');await expect(browserRequest({operation:'hide'})).resolves.toEqual({visible:true});expect(invoke).toHaveBeenLastCalledWith('research_browser',{request:{operation:'hide'}});});
 it('clips overflowing native view at the client edge during large font/layout transitions',()=>{expect(browserBounds({x:900,y:500,width:400,height:400},1.25,{width:1180,height:800})).toEqual({x:1125,y:625,width:350,height:375});expect(browserBounds({x:0,y:900,width:400,height:50},1,{width:1180,height:800})).toBeNull();});
});
