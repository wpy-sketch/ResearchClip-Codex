import {beforeEach,expect,it,vi} from 'vitest';
const mock=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>({invoke:mock.invoke}));
import {confirmSource} from './zoteroApi';
beforeEach(()=>{mock.invoke.mockReset();});
it('refreshes the PDF owner after a delayed persisted confirmation, independently of the source pane',async()=>{
 let finish!:()=>void;mock.invoke.mockImplementation(()=>new Promise<void>(resolve=>{finish=resolve;}));
 const owner=vi.fn(async()=>{}),task=confirmSource('token','preview',owner);
 expect(owner).not.toHaveBeenCalled();
 // The navigation pane may have unmounted while native storage was pending.
 finish();await task;
 expect(mock.invoke).toHaveBeenCalledWith('confirm_pdf_zotero',{token:'token',previewId:'preview'});
 expect(owner).toHaveBeenCalledOnce();
});
it('does not refresh unconfirmed sources when native validation or storage fails',async()=>{
 mock.invoke.mockRejectedValue(Error('source mismatch'));const owner=vi.fn(async()=>{});
 await expect(confirmSource('token','preview',owner)).rejects.toThrow('source mismatch');expect(owner).not.toHaveBeenCalled();
});
it('reports owner refresh failure after persistence so the caller can offer a cache reload',async()=>{
 mock.invoke.mockResolvedValue(undefined);const owner=vi.fn(async()=>{throw Error('reload failed');});
 await expect(confirmSource('token','preview',owner)).rejects.toThrow('reload failed');expect(owner).toHaveBeenCalledOnce();
});
