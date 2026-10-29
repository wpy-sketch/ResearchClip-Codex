export const featureKinds=['CORE','BUILTIN_OPTIONAL','RESOURCE_PACKAGE','MODEL_PACKAGE','EXTERNAL_PLUGIN','APP_UPDATE_REQUIRED'] as const;
export const implementations=['IMPLEMENTED','PARTIAL','NOT_IMPLEMENTED','BROKEN'] as const;
export type FeatureKind=typeof featureKinds[number];
export type Implementation=typeof implementations[number];
export const probeNames=['native','clipper','browserHost','browser','webdav','connector','zoteroPlugin','ocr','python','latex','semantic','qwenRuntime','qwen','packageOcr','ocrFulltext','unavailable'] as const;
export type ProbeName=typeof probeNames[number];
export type Presence='YES'|'NO'|'UNKNOWN';
export type Activation=Presence|'NOT_APPLICABLE';
export type Availability='READY'|'BLOCKED'|'UNVERIFIED';
export type Control='clipper'|'connector';
export type IssueCode='NOT_CHECKED'|'DESKTOP_REQUIRED'|'PROBE_FAILED'|'DEPENDENCY_MISSING'|'DEPENDENCY_UNVERIFIED'|'DISABLED'|'NOT_CONFIGURED'|'AUTH_REQUIRED'|'INTEGRITY_ON_USE'|'INSTALLATION_UNKNOWN'|'HOST_VERIFICATION_REQUIRED'|'APP_UPDATE_REQUIRED'|'APP_VERSION_UNSUPPORTED'|'LAST_JOB_FAILED';
export interface FeatureIssue {code:IssueCode;message:string}
export interface FeatureDefinition {readonly id:string;readonly name:string;readonly kind:FeatureKind;readonly implementation:Implementation;readonly version:string;readonly compatibleAppVersions:readonly string[];readonly dependencies:readonly string[];readonly sourceFiles:readonly string[];readonly probe:ProbeName;readonly control:Control|null;readonly verificationNote:string}
export interface Observation {installed:Presence;enabled:Activation;installedVersion:string|null;availability:Availability;issues:readonly FeatureIssue[]}
export interface FeatureState extends Observation {definition:FeatureDefinition;checkedAt:number|null}
export interface PackageSnapshot {features:readonly FeatureState[];checkedAt:number|null}
export interface FeatureContext {desktop:boolean;appVersion:string}
export const kindLabels:Record<FeatureKind,string>={CORE:'安装包核心',BUILTIN_OPTIONAL:'内置可选',RESOURCE_PACKAGE:'资源包',MODEL_PACKAGE:'AI模型',EXTERNAL_PLUGIN:'外部插件',APP_UPDATE_REQUIRED:'需升级主程序'};
export const implementationLabels:Record<Implementation,string>={IMPLEMENTED:'已实现',PARTIAL:'部分实现',NOT_IMPLEMENTED:'未实现',BROKEN:'已损坏'};
export const presenceLabels:Record<Presence,string>={YES:'是',NO:'否',UNKNOWN:'未知'};
export const activationLabels:Record<Activation,string>={...presenceLabels,NOT_APPLICABLE:'不适用'};
export const availabilityLabels:Record<Availability,string>={READY:'可用',BLOCKED:'不可用',UNVERIFIED:'待核验'};
