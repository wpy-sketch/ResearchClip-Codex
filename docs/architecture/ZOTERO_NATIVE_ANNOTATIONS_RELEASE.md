# Zotero六类批注与4.4.0发布边界

日期：2026-10-28。增量沿用Tauri2、React、PDF.js、SQLite21及既有ZoteroAdapter；不创建第二套笔迹ID或Markdown服务。

## 1. 数据与显示

核查已安装Zotero10.0.6及其源码，当前标准类型为highlight、underline、note、text、image、ink。原始身份包含实例、library类型/ID、item key、attachment key、annotation key及版本。ResearchClip稳定资源ID仍由现有统一资源层提供。

`annotationPosition`保存PDF内部坐标。rects/nextPageRects对应普通批注；ink保留paths和width；自由文本保留fontSize与rotation。通过同一个PDF.js viewport矩阵投影到ResearchClip原生Canvas/SVG图层。只读投影不会生成私有HandwritingLayer，也不把Zotero ink转换为可回写的私有笔迹。用户已确认目前只需原生显示与标准同步，因此本轮不开发私有可编辑副本，也不将其列为缺失功能。

每个来源完整缓存最多30,000条，额外探测一条判断截断。单条坐标、路径和文字有预算，拒绝畸形数据并保留诊断。按不可变缓存建立页面索引；列表每页50项，末页可达。普通请求100项/页；单次响应超过2MiB时在同一偏移缩小至20、1项。30000条对应300页与一次溢出探测。密集页Canvas分帧绘制，限制位图面积和DPR。

## 2. 本地写入和云同步

高亮、下划线、便签、自由文本、区域共五类具备严格参数验证及本地API写入路径；ink仅接受远端读取。修改包含预期版本，提交后必须复读核实。失败、冲突和未完成输入继续保留在原待处理队列。新增自由文本/区域的真实宿主CRUD尚未验收，不能把模拟HTTP测试称为Zotero验收。

PDF打开后加载既有缓存并尝试刷新已确认来源。首次确认来源后重新载入该会话，修复预热为空导致来源列表滞留的问题。只有明确授权且记住授权的来源、完整LOCAL_DIRTY条目才可自动提交；一次性授权、冲突、失败条目和编辑中的内容不自动提交。默认每30秒刷新，待提交完整修改可在2秒后处理；用户可关闭实时联动。

ResearchClip只访问本机API。Zotero账号、数据同步与附件同步由Zotero自身设置管理，ResearchClip不读取云端凭据，也不将“本地写入并复读”显示为“已上传云端”。如Zotero云同步关闭，本地成功不会开启云同步。

## 3. UI与权限

按类型/颜色/文字筛选；右侧导航可拖动及键盘调宽，窄面板在下方显示以保留阅读空间。文件名换行完整保留。点击批注区域清除筛选并定位对应列表项。划词工具栏保存原Range，提供五类批注、复制、用户主动外部翻译及可配置的有限快捷键；不自动发送整份PDF。

2026-10-28用户已确认采用真正的Chrome，账号与书签同步由Chrome管理。现有入口请求已安装Chrome打开固定HTTPS首页，保留ResearchClip编辑会话；不将内嵌Chrome或内嵌账号同步列为本轮待实现功能。官方Google OAuth限制嵌入用户代理，普通WebView不能等同Chrome。原CSP保持frame-src none，不向远程Google页面开放Tauri IPC。

XPI按Zotero宿主安装流程使用，ResearchClip提供受验证下载、导出与向导，不将XPI加载为Tauri插件。桌面输入被Windows拒绝时不得绕过宿主安装与授权来宣称验证成功。

## 4. 兼容性与回退

本轮新增位置字段采用serde默认值，继续读取格式1缓存；不新增SQLite迁移，原Step1/2迁移21保留。关闭实时同步后仍可只读缓存和手动处理。解析不支持的批注原文和身份保留；不修改原始PDF，不直接读写Zotero数据库。

## 5. 核查来源

- [Zotero10.0.6批注实现](https://github.com/zotero/zotero/blob/10.0.6/chrome/content/zotero/xpcom/annotations.js)
- [官方插件开发说明](https://www.zotero.org/support/dev/client_coding/plugin_development)
- [Zotero同步设置](https://www.zotero.org/support/sync)
- [Google原生应用OAuth限制](https://developers.google.com/identity/protocols/oauth2/native-app)
- [Chrome书签扩展API](https://developer.chrome.com/docs/extensions/reference/api/bookmarks)
- [Tauri2官方Updater](https://v2.tauri.app/plugin/updater/)
