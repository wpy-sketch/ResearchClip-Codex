> 2026-10-27范围更新：用户停止IDE开发，将未发布Connector与30,000条批注改进合并到v4.4.0。此文保留前序实现/验收历史，v4.2.8未单独发布；当前状态见[版本验收](../qa/RELEASE-4.4.0.md)。

# Step17 / v4.2.8：ResearchClip Zotero Connector

本轮授权：专用Zotero内部插件、认证本机协议、来源辅助页、可调整布局及配套验收发布。后续v4.3.0仅另做IDE路线评估与独立Windows原型，不替换工作台。v4.2.7源码标签与附件不变。

## 核查与边界

2026-10-27读取本机Zotero app/application.ini：10.0.6 / Build20261007153950 / Gecko140。核对[官方插件开发](https://www.zotero.org/support/dev/client_coding/plugin_development)、[Zotero7插件生命周期与manifest](https://www.zotero.org/support/dev/zotero_7_for_developers)、[Firefox140迁移](https://www.zotero.org/support/dev/zotero_8_for_developers)、[Zotero10变更](https://www.zotero.org/support/dev/zotero_10_for_developers)和[JavaScript API](https://www.zotero.org/support/dev/client_coding/javascript_api)。官方7/8指南不证明10所有接口；另逐项检查本机精确版本MenuManager、Notifier、Reader、HTTP源码，摘要保留于隔离测试产物。未跑的接口只列待验证。

插件使用manifest.json和bootstrap.js，startup/shutdown与窗口钩子清理菜单、监听器、计时器和连接。10.0.6中菜单目标为main/library/item，Notifier使用item/file类型；Reader.open(itemID,location,options)位置含pageIndex或annotationID。重启后所有临时授权须重新配对。严格版本范围以实际平台实验收敛，不广泛承诺7/8/9/所有10.x。

## 进程与协议

Zotero插件仍运行于Zotero；Tauri主应用继续React/Rust模块化单体。插件不作为Tauri插件加载。不复用网页剪藏权限或密钥，复用其现有随机数及受限IPC经验。使用仅127.0.0.1的短请求HTTP；直接复用锁文件已存在的httparse1.10.1解析HTTP头，不引入完整Node服务或自写HTTP语法解析器。已核对精确版本API及MIT OR Apache-2.0元数据，本发行选择MIT并保留完整许可。

连接默认关闭。用户启用并复制有时限的一次配对码，粘贴到Zotero插件后显式连接。配对和会话能力使用现有系统安全随机数；配对码单次且短时有效，会话token仅在两端内存，不能混同Zotero写入API key。Rust执行复制，不向Tauri JS返回token。关闭、取消或失联清除能力与未执行请求，恢复需重新配对。HTTP拒绝Origin、代理/重定向请求、未知方法/路径、重复关键头、超限长度和非法消息；只接收已定义的操作与完整来源身份。固定请求ID和有界回执避免重放与重复打开；同ID不同内容拒绝。

认证证明用户授权的本机配对能力，不声称能远程证明插件二进制签名或抵抗同一用户下已完全入侵的进程。插件仍由Zotero按自身机制检查；不降低签名设置、不强制启用、不绕过安装确认。

## 数据复用

PDF打开复用原件校验、PdfOpenRequests、单实例、受管副本与稳定Document/PdfAttachment ID。插件传递完整library type/ID、父item key、attachment key、可用的Zotero实例身份及最少元数据，不将item key单独当全局ID。Zotero本地库内部libraryID和Local API users/0或groupID明确区分。

路径只在已配对的原生边界内用于用户明确选择的PDF打开，不进入WebView。位置使用PDF页码/批注键，边界与文档哈希检查失败不猜测偏移。元数据及来源经现有Local API复核后才复用原有绑定；API不可用仍可作为普通PDF打开，但身份关联明确待核对，不伪造成功。沿用schema20，无第二套批注/Markdown存储，若实测需要额外持久字段再提出版本化迁移。

反向定位仅操作已关联的完整身份。变化通知是刷新提示，不是权威远端快照，也不能触发任意事件或写入；批注同步仍通过现有适配器执行GET比较与版本条件写入，自动刷新关闭时只提示。插件不转换私有笔迹，不自动写Markdown，不修改PDF原件或直接写Zotero SQLite。

## 来源辅助页与交付

展示用户设置的插件来源、声明版本/许可/兼容范围及核验状态；默认提供本项目来源和官方指南，不擅自挑选第三方插件。可信下载必须用户明确选择来源、核对元数据/摘要并选择目标文件；不自动安装或执行。源站/重定向/大小/后缀及取消全部校验。无法核实的许可证、签名或XPI兼容性明确标为未知，不能把仓库声明当宿主安装验证。

先做隔离插件平台实验，再实现类型协议与原生链路；测试配对拒绝、重放、断线/取消、页码/批注、双向定位、监听清理、同步协同、来源下载完整性与不覆盖文件。真实插件与原生应用均在独立配置运行。通过后发布v4.2.8并完整继承历史附件，再交付授权的v4.3.0 IDE原型。没有PDF导出或独立备份恢复产品。


## 当前验证与发布前置条件（2026-10-27）

本机精确安装版本的源码核查、Mock插件单元测试和真实Tauri端口/IPC测试已执行；真实XPI加载与菜单/配对尚未完成。开发代理文件实验未加载；中文路径解析曾作为假设，但相对路径实验仍未证明加载，不能将其说成已定位或修复的根因。未改变签名、安装安全策略或直接写Zotero SQLite。

Windows UI输入实际返回GetCursorPos / 0x80070005。用户明确选择“等桌面可操作并完成验收后再发布”，因此v4.2.8及后续v4.3.0均保持未发布；完整验收记录见[本轮QA](../qa/RELEASE-4.2.8.md)。

新增代码：原生服务zotero_connector（model/wire/core/runtime）、zotero_plugins来源助手与main-only受控IPC；前端按PDF所有者保存来源上下文，复用既有预览确认、来源绑定、坐标适配器和自动刷新设置；原始Zotero插件独立构建为XPI。布局预设只操作既有FlexLayout Model与WorkspacePersistence，不重建EditorSession。
