import {describe,it,expect} from 'vitest';
import {addressDraft,reduceAddress} from './browser-address';
const native=(url:string,loading=false,failed=false)=>({type:'native' as const,url,loading,failed});
describe('native browser address drafts',()=>{
 it('keeps a typed Bing URL after Tab/blur and delayed polling of the previous page',()=>{
  let s=reduceAddress(addressDraft('https://cn.bing.com/'),native('https://www.google.com/'));
  s=reduceAddress(s,{type:'focus'});s=reduceAddress(s,{type:'edit',value:'https://cn.bing.com/'});s=reduceAddress(s,{type:'blur'});
  s=reduceAddress(s,native('https://www.google.com/'));expect(s.value).toBe('https://cn.bing.com/');expect(s.dirty).toBe(true);
 });
 it('keeps a submitted address during old/loading snapshots and displays a completed redirect',()=>{
  let s=reduceAddress(addressDraft('https://www.google.com/'),native('https://www.google.com/'));
  s=reduceAddress(s,{type:'submit',url:'https://cn.bing.com/'});s=reduceAddress(s,native('https://www.google.com/'));
  expect(s.value).toBe('https://cn.bing.com/');s=reduceAddress(s,native('https://www.bing.com/?mkt=zh-CN',true));expect(s.value).toBe('https://cn.bing.com/');
  s=reduceAddress(s,native('https://www.bing.com/?mkt=zh-CN'));expect(s.value).toBe('https://www.bing.com/?mkt=zh-CN');expect(s.pendingBase).toBeNull();
 });
 it('follows native links and history when there is no draft',()=>{let s=reduceAddress(addressDraft('https://cn.bing.com/'),native('https://cn.bing.com/'));s=reduceAddress(s,native('https://www.bing.com/search?q=math'));expect(s.value).toContain('search?q=math');});
 it('retains the failed requested address for retry',()=>{let s=reduceAddress(addressDraft('https://cn.bing.com/'),native('https://cn.bing.com/'));s=reduceAddress(s,{type:'submit',url:'https://offline.example.org/'});s=reduceAddress(s,native('https://cn.bing.com/',false,true));s=reduceAddress(s,native('https://cn.bing.com/'));expect(s.value).toBe('https://offline.example.org/');expect(s.dirty).toBe(true);});
 it('keeps focus stable then reconciles an unedited address on blur, and Escape cancels only the draft',()=>{let s=reduceAddress(addressDraft('https://cn.bing.com/'),native('https://cn.bing.com/'));s=reduceAddress(s,{type:'focus'});s=reduceAddress(s,native('https://www.bing.com/'));expect(s.value).toBe('https://cn.bing.com/');s=reduceAddress(s,{type:'blur'});expect(s.value).toBe('https://www.bing.com/');s=reduceAddress(s,{type:'edit',value:'draft'});s=reduceAddress(s,{type:'cancel'});expect(s.value).toBe('https://www.bing.com/');});
});
