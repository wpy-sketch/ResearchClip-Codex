# 应用内科研浏览器决策

2026-10-29，基于已安装Tauri2.12.3/React19/WebView2，应用候选5.0.0。复用资料工作台，不替换PDF、Markdown和图片会话，无数据库变更、无新依赖。

## 方案与兼容性

采用`WebviewWindowBuilder::parent_raw`创建Windows WS_CHILD窗口，原生网页独立WebView2，React提供导航与地址栏。已核对安装版本源码和[官方接口](https://docs.rs/tauri/2.12.3/tauri/webview/struct.WebviewWindowBuilder.html#method.parent_raw)。不使用iframe，避免网站X-Frame-Options阻止嵌入；不移除网站安全响应头。

没有采用实验性`Window::add_child`：安装版本的`WebviewWindow`命令提取要求主窗口只有同标签视图，多视图会使现有命令失效。独立WS_CHILD保留主窗口既有单视图关系，无需全局改写业务命令。窗口创建在async命令的后台线程中执行，避免Windows WebView2创建死锁。

## 权限与不可信输入

主能力从窗口范围收紧为`webviews:["main"]`，没有remote配置。网页标签`research-browser`没有任何Tauri能力；应用IPC、SQLite、文件、截图、剪贴板读取和Zotero权限均不授予网页。导航只接受有界公开HTTP(S)地址，拒绝凭据、本地文件、应用协议、直接私有IP与localhost；这不是完整的网络防火墙或DNS重绑定防护。外部页面按WebView2网络规则运行。

网页使用独立临时浏览会话。拒绝设备/通知权限和自动文件下载；新窗口链接在同一网页区打开，不启动任意协议。浏览操作只支持固定后退、前进、停止脚本，不提供任意eval接口。Google登录兼容性、Chrome账号与书签同步不承诺，保留真正Chrome入口。

## 生命周期与布局

DOMRect乘实际devicePixelRatio转换为物理像素，后端验证有限数、最小尺寸与主窗口边界。ResizeObserver与有界500ms采样处理分栏、窗口尺寸、CSS缩放、隐藏标签及应用内模态框。隐藏/返回工作台保留浏览历史，所有操作串行，避免迟到show覆盖下一编辑器。关闭主窗口释放所属子窗口；重启不自动打开网页，不持久记录网址或网页正文。

## 验收与回退

必须实测Windows创建、导航、主命令持续可用、远端IPC拒绝、分栏/DPI坐标、隐藏恢复和正常退出。自动化与实际网络结果分别记录，不能把加载回调等同远站可用。macOS/Linux暂不提供内嵌窗口，显示明确错误，可使用外部浏览器。失败时返回工作台，原编辑会话保留；不自动放宽权限。
