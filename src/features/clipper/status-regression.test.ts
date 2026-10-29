import {beforeEach,describe,it,expect,vi} from 'vitest';
import {useClipperStore} from './store';
const native=vi.hoisted(()=>({invoke:vi.fn(),isTauri:vi.fn(()=>true)}));
vi.mock('@tauri-apps/api/core',()=>native);
const status=(enabled:boolean)=>({enabled,extensionId:'a'.repeat(32),last:null});
beforeEach(()=>{native.invoke.mockReset();native.isTauri.mockReturnValue(true);useClipperStore.setState({status:null,busy:false,error:null,notice:null});});
describe('clipper connection status authority and recovery',()=>{
 it('does not let an older status response undo a later explicit native toggle',async()=>{let finish!:(v:unknown)=>void;native.invoke.mockImplementationOnce(()=>new Promise(resolve=>finish=resolve));const old=useClipperStore.getState().refresh();native.invoke.mockResolvedValueOnce(status(true));await useClipperStore.getState().connect(true);finish(status(false));await old;expect(useClipperStore.getState().status?.enabled).toBe(true);expect(useClipperStore.getState().busy).toBe(false);});
 it('clears a prior refresh failure after the next valid status',async()=>{native.invoke.mockRejectedValueOnce(Error('owned transient failure'));await useClipperStore.getState().refresh();expect(useClipperStore.getState().error).toBeTruthy();native.invoke.mockResolvedValueOnce(status(false));await useClipperStore.getState().refresh();expect(useClipperStore.getState().error).toBeNull();});
 it('rejects malformed native flags and preserves the previous valid observation',async()=>{native.invoke.mockResolvedValueOnce(status(false));await useClipperStore.getState().refresh();native.invoke.mockResolvedValueOnce({...status(false),enabled:'yes'});await useClipperStore.getState().refresh();expect(useClipperStore.getState().status).toEqual(status(false));expect(useClipperStore.getState().error).toBeTruthy();});
});
