/** Local address draft only; native polling must not replace unsubmitted input. */
export interface AddressDraft {value:string;nativeUrl:string;dirty:boolean;focused:boolean;pendingBase:string|null}
export type AddressEvent={type:'edit';value:string}|{type:'focus'|'blur'|'cancel'|'failed'}|{type:'submit';url:string}|{type:'native';url:string;loading:boolean;failed:boolean};
export const addressDraft=(home:string):AddressDraft=>({value:home,nativeUrl:'',dirty:false,focused:false,pendingBase:null});
export function reduceAddress(s:AddressDraft,event:AddressEvent):AddressDraft{
 switch(event.type){
  case 'edit':return {...s,value:event.value,dirty:true,pendingBase:null};
  case 'focus':return {...s,focused:true};
  case 'blur':return {...s,focused:false,value:!s.dirty&&s.pendingBase===null&&s.nativeUrl?s.nativeUrl:s.value};
  case 'cancel':return {...s,value:s.nativeUrl||s.value,dirty:false,pendingBase:null};
  case 'failed':return {...s,dirty:true,pendingBase:null};
  case 'submit':return {...s,value:event.url,dirty:false,focused:false,pendingBase:s.nativeUrl};
  case 'native':{
   if(!event.url)return s;
   const next={...s,nativeUrl:event.url};
   if(event.failed)return s.pendingBase===null?next:{...next,dirty:true,pendingBase:null};
   if(s.dirty||s.focused||s.pendingBase!==null&&(event.loading||event.url===s.pendingBase))return next;
   return {...next,value:event.url,pendingBase:null};
  }
 }
}
