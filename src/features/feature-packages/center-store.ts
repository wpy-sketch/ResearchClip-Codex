import {create} from 'zustand';
import {packageRequest,updateRequest,type PackageCenterSnapshot,type UpdateSnapshot} from './center-api';
import {errorMessage} from '../../shared/api/errors';
import {useCardStore} from '../notes/store';
interface CenterState {snapshot:PackageCenterSnapshot|null;update:UpdateSnapshot|null;busy:boolean;error:string|null;readError:string|null;read:()=>Promise<void>;action:(request:Parameters<typeof packageRequest>[0])=>Promise<void>;updateAction:(operation:Parameters<typeof updateRequest>[0],confirmed?:boolean)=>Promise<void>}
let reading=false,generation=0;
export const usePackageCenter=create<CenterState>((set,get)=>({snapshot:null,update:null,busy:false,error:null,readError:null,
 read:async()=>{if(reading||get().busy)return;reading=true;const ticket=generation;try{const [snapshot,update]=await Promise.all([packageRequest({operation:'status'}),updateRequest('status')]);if(ticket===generation)set({snapshot,update,readError:null});}catch(e){if(ticket===generation)set({readError:errorMessage(e,'组件状态读取失败。')});}finally{reading=false;}},
 action:async request=>{if(get().busy)return;generation++;set({busy:true,error:null});try{set({snapshot:await packageRequest(request)});}catch(e){set({error:errorMessage(e,'组件操作失败，旧版本保留。')});}finally{set({busy:false});}},
 updateAction:async(operation,confirmed=false)=>{if(get().busy)return;generation++;set({busy:true,error:null});try{if(operation==='install'&&confirmed&&!await useCardStore.getState().flushAll())throw Error('截图笔记尚未保存，未开始更新。请先修复保存错误。');set({update:await updateRequest(operation,confirmed)});}catch(e){set({error:errorMessage(e,'更新操作未完成。')});}finally{set({busy:false});}}
}));
