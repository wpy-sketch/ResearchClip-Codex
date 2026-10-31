# 应用内科研浏览器决策

2026-10-29，基于已安装Tauri2.12.3/React19/WebView2，应用5.0.1。复用资料工作台，不替换PDF、Markdown和图片会话，无数据库变更、无新依赖。

## 方案与兼容性

采用`WebviewWindowBuilder::parent_raw`创建Windows WS_CHILD窗口，原生网页独立WebView2，React提供导航与地址栏。已核对安装版本源码和[官方接口](https://docs.rs/tauri/2.12.3/tauri/webview/struct.WebviewWindowBuilder.html#method.parent_raw)。不使用iframe，避免网站X-Frame-Options阻止嵌入；不移除网站安全响应头。

没有采用实验性`Window::add_child`：安装版本的`WebviewWindow`命令提取要求主窗口只有同标签视图，多视图会使现有命令失效。独立WS_CHILD保留主窗口既有单视图关系，无需全局改写业务命令。窗口创建在async命令的后台线程中执行，避免Windows WebView2创建死锁。

## 权限与不可信输入

主能力从窗口范围收紧为`webviews:["main"]`，没有remote配置。网页标签`research-browser`没有任何Tauri能力；应用IPC、SQLite、文件、截图、剪贴板读取和Zotero权限均不授予网页。导航只接受有界公开HTTP(S)地址，拒绝凭据、本地文件、应用协议、直接私有IP与localhost；这不是完整的网络防火墙或DNS重绑定防护。外部页面按WebView2网络规则运行。

网页使用独立临时浏览会话。拒绝设备/通知权限和自动文件下载；新窗口链接在同一网页区打开，不启动任意协议。浏览操作只支持固定后退、前进、停止脚本，不提供任意eval接口。Google登录兼容性、Chrome账号与书签同步不承诺，保留真正Chrome入口。

## 生命周期与布局

DOMRect乘实际devicePixelRatio转换为物理像素，后端验证有限数、最小尺寸与主窗口边界。ResizeObserver与有界500ms采样处理分栏、窗口尺寸、CSS缩放、隐藏标签及应用内模态框。隐藏/返回工作台保留浏览历史，所有操作串行，避免迟到show覆盖下一编辑器。关闭主窗口释放所属子窗口；重启不自动打开网页，不持久记录浏览历史或网页正文；用户明确保存的网页书签写入既有本机设置。

## 验收与回退

必须实测Windows创建、导航、主命令持续可用、远端IPC拒绝、分栏/DPI坐标、隐藏恢复和正常退出。自动化与实际网络结果分别记录，不能把加载回调等同远站可用。macOS/Linux暂不提供内嵌窗口，显示明确错误，可使用外部浏览器。失败时返回工作台，原编辑会话保留；不自动放宽权限。

## 5.0.1左侧入口与书签

浏览模式复用UiState.activeView，通过最左侧主导航进入；切换其他导航隐藏原生层，返回按钮恢复进入前的工作台视图。PDF和Markdown会话仍在原Dock/EditorSession中。

书签只包含title/url，URL作为本地条目去重键，不增加研究资源ID系统。既有Preferences增加serde默认的browserBookmarks，沿用SettingsService事务式暂存和原子替换；最多32项、名称80 UTF16单元、地址2048字节，完整设置保留64KiB读写上限。旧设置缺字段时默认空数组，不重写SQLite。非法网址、重复项或失败写入保留原文件。

升级时，仅对编译钉住的5.0.0/5.0.1清单及完全匹配的旧组件元数据更新应用兼容声明；允许JSON行尾LF/CRLF差异，篡改或未知凭据不迁移。模型文件不改写，先标RECHECK_REQUIRED，随后既有ModelManager实际校验与CPU自检成功才恢复READY。该机制无需生产签名密钥，不接受任意本地/远端目录。

## 5.1.0 网络、地址草稿与设置

原生视图先创建about:blank，再注册WebView2 NavigationStarting/NavigationCompleted后加载既有Preferences中的browserHomeUrl。仅内部首次空白导航可用；用户和网页仍不能导航本地文件、内网或应用协议。原生完成事件回报DNS、TLS、超时及代理错误，按原生NavigationId忽略旧请求回调。远程WebView没有主应用能力。

地址栏用独立纯reducer保存dirty/focus/pending状态，500ms轮询只更新原生地址；未提交草稿、离焦和等待导航时不覆盖输入。Esc撤销地址草稿；提交后的重定向在完成后显示。设置沿用SettingsService原子JSON，不增加正文服务、资源ID或SQLite迁移。默认首页https://cn.bing.com/，搜索引擎必应，Google可选。

为调用当前原生完成事件，直接声明已有锁定传递依赖webview2-com=0.39.1（MIT），并精确钉住Tauri2.12.3；无依赖全集升级。架构原则参考[Electron WebContentsView](https://www.electronjs.org/docs/latest/api/web-contents-view)独立主进程管理网页视图，不读取或复制Codex私有实现。实际Windows Bing首页、重定向与搜索、连接失败、设置落盘和原工作台保留分别验收。

远程网页使用单独research-browser-v1 WebView环境目录，与受信任主工作台的EBWebView隔离，继续使用in-private会话；避免共享环境参数或主视图缓存干扰。目录仅由应用本机路径构造，不能由网页指定。原生验收还需不设置WEBVIEW2_USER_DATA_FOLDER的独立应用标识配置，确认正常安装环境的默认路径可用；调试数据覆盖只在debug_assertions构建有效。

导航完成与窗口捕获画面可能不一致；主WebView或窗口捕获不能独自证明独立网页的前台显示。为维护WS_CHILD与主工作台同父窗口的层级，可见状态维护使用Windows SetWindowPos(HWND_TOP)保留同父子窗口层级，SWP_NOACTIVATE不抢地址栏焦点；隐藏、关闭及模态状态不抬升。导航完成不能代替原生前台画面验收。

2026-10-31独立QA标识的优化构建未覆盖WEBVIEW2_USER_DATA_FOLDER，原生导航、自然缓存路径及重启恢复实际通过；用户直接确认专用窗口可见必应首页与搜索框。电脑操作截图在设置页也返回黑屏，记为捕获工具限制，不能称自动视觉通过。
