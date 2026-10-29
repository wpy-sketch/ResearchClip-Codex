# Windows安装、升级与验收指南

当前5.0.0是本地验收候选，**正式Release和安装包下载URL尚不存在**。用户已明确取消Catalog及Updater的密钥配置要求，改用内置固定组件清单和手动升级。请勿把本机安装包或测试签名当作公开稳定版。发布目标从origin核实为[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。

## 1. 正式发布后如何安装

1. 从该仓库实际Release页面取得`ResearchClip_5.0.0_x64-setup.exe`和SHA256SUMS；以正式报告中的实际URL为准。目前没有公开下载入口。
2. 在PowerShell运行`Get-FileHash -LiteralPath '.\ResearchClip_5.0.0_x64-setup.exe' -Algorithm SHA256`，与同一Release校验清单核对。哈希只能核对文件一致性，不能代替发布者身份签名。
3. 运行NSIS安装程序，选择当前用户安装目录。用户资料、截图、Markdown和模型保存在独立数据位置，不随程序安装目录移动。
4. 从开始菜单或安装目录启动。用户不需要Rust、Node.js或Python开发环境；主程序携带可信原生OCR运行时，模型权重按需安装。WebView2按安装程序依赖机制处理。
5. PDF文件关联提供“打开方式”，不擅自替换现有默认阅读器。也可在Zotero设置中选择ResearchClip程序作为外部PDF阅读器。

## 2. 旧版本升级

关闭ResearchClip及相关资料操作再安装。安装器应拒绝运行中直接覆盖，升级前检查版本，不允许降级写入未来数据库。数据库按既有版本化事务迁移及升级前快照处理。升级验收必须检查原截图、Markdown、模型及关联仍存在；不要用重置真实AppData消除报错。

## 3. 自动更新

本版按用户要求停用自动更新，无需配置公钥或私钥，不生成latest.json或更新签名。更新请从实际仓库Release下载新版本NSIS并核对SHA-256，保存并关闭文档后手动安装。组件中心使用随主程序交付的固定清单与真实文件哈希，不能接受任意未签名在线目录。

## 4. 干净Windows人工验收

在独立用户或虚拟机下载实际公开资产，记录安装器SHA、应用版本、Windows版本与显示缩放；完成安装/启动/PDF打开/选区截图/插入Markdown/保存与重开；扩展中心真实下载OCR并离线识别；安装Zotero Connector并配对；验证前一正式版升级和模型保留；卸载后确认研究资料仍在。未实际执行的项目不得签署PASS回执。

2026-10-29桌面已恢复。本机独立Zotero10.0.6已实际安装Connector1.0.1、完成用户主动配对、右键传入测试PDF，并通过真实Local API读取与变化通知检查。完整结果及尚未执行项目以[最终报告](../qa/FINAL_RELEASE_REPORT.md)为准。

## 5. 本地候选验收与正式版的区别

本轮已在独立中文/空格路径执行NSIS安装、运行中拒绝覆盖/卸载、PDF打开方式注册及卸载；34份原有资料文件逐一核对未改写。该验收运行于当前Windows机器，不能代替另一台干净Windows从GitHub下载后验收。截图高DPI的CDP模拟也不能替代物理混合DPI/HDR设备。

安装器当前Windows Authenticode状态为NotSigned；这与Tauri Updater缺少独立更新签名是两个事项。候选内容变化后必须重新构建并验收新字节，不能沿用旧安装器哈希回执。当前Updater已停用，不应期待或伪造自动升级成功。
