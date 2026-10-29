import {mathStatus} from '../math-ocr/api';
import {semanticStatus} from '../semantic/api';
import {aiStatus} from '../similarity/api';
import {ocrStatus} from '../ocr/api';
import {readClipperStatus,useClipperStore} from '../clipper/store';
import {connectorControl} from '../pdf/connector';
import {syncStatus} from '../sync/api';
import {PackageManager,observed,type Probes} from './PackageManager';
import {featureRegistry} from './FeatureRegistry';
import type {FeatureContext,Observation} from './types';
import {packageRequest} from './center-api';
/** Share only in-flight reads. A later check always rereads the existing service. */
function shared<T>(read:()=>Promise<T>){let pending:Promise<T>|null=null;return ()=>{if(!pending)pending=read().finally(()=>pending=null);return pending;};}
export function createNativePackageManager(context:FeatureContext){
 const math=shared(mathStatus),semantic=shared(semanticStatus),qwen=shared(aiStatus),connector=shared(()=>connectorControl('status')),packages=shared(()=>packageRequest({operation:'status'})),ocr=shared(ocrStatus);
 const presence=(found:boolean):Observation=>found?observed('YES','NOT_APPLICABLE','UNVERIFIED','INTEGRITY_ON_USE','文件存在/大小符合清单；SHA与运行能力仍由原服务在使用前核验。'):observed('NO','NOT_APPLICABLE','BLOCKED','NOT_CONFIGURED','缺少所需资源，请使用原模块的受控导入/下载。');
 const probes:Probes={
  packageOcr:async()=>{const s=await packages(),e=s.registry.installed.find(e=>e.package.id==='model.ocr-line');return e?observed('YES',e.enabled?'YES':'NO',e.enabled&&e.readiness==='READY'?'READY':'BLOCKED',e.enabled?undefined:'DISABLED',e.readiness,e.package.version):observed('NO','NO','BLOCKED','NOT_CONFIGURED','请在扩展与模型中心查看可信模型发布情况。');},
  ocr:async()=>{const s=await ocr();return s.languages.some(l=>l.tag!=='researchclip-ocr-line')?observed('YES','NOT_APPLICABLE','READY'):observed('NO','NOT_APPLICABLE','BLOCKED','NOT_CONFIGURED','Windows未报告可用OCR语言包。');},
  ocrFulltext:async()=>{const s=await ocr();return s.languages.length?observed('YES','YES','READY'):observed('YES','YES','UNVERIFIED','NOT_CONFIGURED','已有全文可搜索；生成新OCR索引需安装可用识别提供方。');},
  python:async()=>{const s=await math();return s.configured?observed('UNKNOWN','YES','UNVERIFIED','INSTALLATION_UNKNOWN','已配置运行时路径；尚不能确认解释器和所需Python模块完整。'):observed('NO','NO','BLOCKED','NOT_CONFIGURED','尚未选择本地Python运行时。');},
  latex:async()=>presence((await math()).modelsPresent),
  semantic:async()=>{const s=await packages(),e=s.registry.installed.find(e=>e.package.id==='model.semantic');return e?observed('YES',e.enabled?'YES':'NO',e.enabled&&e.readiness==='READY'?'READY':'BLOCKED',e.enabled?undefined:'DISABLED',e.readiness,e.package.version):presence((await semantic()).modelsPresent);},
  qwen:async()=>presence((await qwen()).available),
  qwenRuntime:async()=>{const s=await qwen();return s.available?observed('UNKNOWN','NOT_APPLICABLE','UNVERIFIED','INTEGRITY_ON_USE','已找到模型入口；完整运行时白名单/摘要在任务开始前核验。'):observed('NO','NOT_APPLICABLE','BLOCKED','NOT_CONFIGURED','尚未配置完整Qwen离线包。');},
  clipper:async()=>{const s=await readClipperStatus();return s.enabled?observed('YES','YES','READY'):observed('YES','NO','BLOCKED','DISABLED','本机剪藏桥已编译，当前连接未启用。');},
  browserHost:async()=>observed('UNKNOWN','NOT_APPLICABLE','UNVERIFIED','INSTALLATION_UNKNOWN','现有接口未报告Host文件与注册状态；需要按扩展包说明在浏览器侧验收。'),
  browser:async()=>observed('UNKNOWN','UNKNOWN','UNVERIFIED','INSTALLATION_UNKNOWN','没有浏览器扩展安装检测接口；本机桥状态不能代替浏览器安装/Host验收。'),
  connector:async()=>{const s=await connector();return s.enabled?observed('YES','YES',s.connected?'READY':'UNVERIFIED',s.connected?undefined:'INSTALLATION_UNKNOWN',s.connected?'':'本机桥已启用，尚未观察到已配对插件。'):observed('YES','NO','BLOCKED','DISABLED','Zotero本机桥默认关闭。');},
  zoteroPlugin:async()=>{const s=await connector();return s.connected?observed('YES','YES','UNVERIFIED','HOST_VERIFICATION_REQUIRED','观察到配对协议端，不等于完整安装/菜单/生命周期验收。',s.pluginVersion):observed('UNKNOWN','UNKNOWN','UNVERIFIED','INSTALLATION_UNKNOWN','未连接不能推断XPI未安装；请在Zotero插件管理器核对。');},
  webdav:async()=>{const s=await syncStatus();if(!s.config)return observed('YES','NO','BLOCKED','NOT_CONFIGURED','WebDAV未配置；此检查不会连接服务器。');if(!s.credentialAvailable||s.credentialError)return observed('YES','YES','BLOCKED','AUTH_REQUIRED','同步已配置但凭据不可用，请到原设置修复。');return observed('YES','YES','UNVERIFIED','HOST_VERIFICATION_REQUIRED','配置与凭据存在；不会在组件检查时发起网络能力测试或同步。');},
 };
 return new PackageManager(featureRegistry,probes,{clipper:async enabled=>{await useClipperStore.getState().connect(enabled);},connector:async enabled=>{await connectorControl(enabled?'enable':'disable');}},context);
}
