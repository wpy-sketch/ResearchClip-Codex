# ResearchClip v5.1.0

Windows x64手动安装版，唯一维护仓库为[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。实际发布、SHA与本轮测试见[最终报告](https://github.com/wpy-sketch/ResearchClip-Codex/blob/main/docs/qa/FINAL_RELEASE_REPORT.md)。

## 网页浏览修复

- 修复输入网址后按Tab被后台轮询覆盖，保留地址草稿、失败地址与焦点；Esc取消草稿。
- 默认必应首页，设置可保存首页和必应/Google搜索选择；地址栏仍可直接打开公开网页。
- 独立原生WebView2环境，先创建空白视图再加载；显示真实DNS、TLS、超时与代理错误，不关闭证书检查。
- PDF、Markdown与本地书签保留；网页仍无科研资料IPC权限。Chrome账号/书签同步仍由真正Chrome管理。

## 扩展与模型、设置

- 固定清单提供OCR、原生CPU图文语义模型、Zotero Connector和网页剪藏包，下载校验、取消、启停与受管卸载沿用既有服务。
- 语义模型固定8文件，图像MIT、文本Apache-2.0；无Python依赖，实际推理自检通过才READY。主动建立本地索引后可中英文描述找图。
- 网页剪藏1.0.3补充MIT许可与说明，保留已验证1.0.2 Host及第三方许可；下载不等于浏览器安装，应用不执行注册脚本。
- 本机状态检查区分安装、启用、可用及错误；卡片直接定位实际配置模块。新增首页/搜索设置，补全公式、语义、本地AI、命令设置导航。
- 指南按1–38章顺序保留，287示例、79幅离线图，补充真实Bing截图和安装流程。

## 仓库与发布资源

原ResearchClip的49个Git引用、47版说明及4,738条附件引用已迁入当前仓库。543个去重文件约8.42GB，包含67个直接EXE/XPI和16个独立ZIP卷；匿名下载逐项核验。历史文件映射见[历史归档](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/researchclip-history-20261030)，新版Release另附索引与离线指南。旧仓库在全部发布核验后作为只读入口保留，不删除。

## 兼容与限制

SQLite21无迁移，原图、Markdown、数据库不变。相同OCR旧清单证明可更新兼容元数据，真实自检后恢复可用。直接声明既有锁定webview2-com0.39.1（MIT）用于原生诊断，Tauri固定2.12.3，没有依赖全集升级。

按用户要求，自动更新关闭，手动NSIS升级，不需要Catalog或Updater密钥，不生成虚假.sig/latest.json。安装器未做Windows代码签名。独立干净Windows、物理混合DPI/HDR、多屏、当前原生文件对话框与Zotero写入复测等范围详见报告；不把本机隔离通过称为这些项目已验收。公式原生一键适配及Qwen托管安装仍未交付，旧Python/离线导入设置保留。未开发IDE、Markdown转PDF或独立备份恢复系统。
