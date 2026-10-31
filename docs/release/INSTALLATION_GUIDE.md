# Windows安装、升级与验收指南

v5.1.0的公开下载与SHA核对结果以[最终报告](../qa/FINAL_RELEASE_REPORT.md)为准；请从对应Release核对SHA，不把构建快照当作公开下载证明。5.0.1旧版保持可用。使用内置固定组件清单与手动升级，无需公私钥配置；完整验收范围见[最终报告](../qa/FINAL_RELEASE_REPORT.md)。

## 1. 下载安装

1. 下载[ResearchClip_5.1.0_x64-setup.exe](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.0/ResearchClip_5.1.0_x64-setup.exe)和同页SHA256SUMS。以[报告中的实际URL及SHA](../qa/FINAL_RELEASE_REPORT.md)核对，不把Draft视作匿名下载入口。
2. 在PowerShell运行`Get-FileHash -LiteralPath '.\ResearchClip_5.1.0_x64-setup.exe' -Algorithm SHA256`，与同一Release校验清单核对。哈希只能核对文件一致性，不能代替发布者身份签名。
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

5.0.1本机隔离验收6组通过：中文/空格路径NSIS安装、运行中拒绝覆盖/卸载、PDF打开方式注册及卸载；34份原有资料文件SHA未变。该验收运行于当前Windows机器，不能代替另一台干净Windows从GitHub下载后验收。截图高DPI的CDP模拟也不能替代物理混合DPI/HDR设备。

最终安装器Windows Authenticode为NotSigned；这与Tauri Updater签名是两个事项。最终NSIS哈希和6组本机安装器验收已记录，但没有用生产程序启动真实资料，不替代另一台干净Windows安装/运行。候选内容变化后需重新构建并验收新字节，不能沿用旧回执。Updater已停用，不应期待或伪造自动升级成功。
