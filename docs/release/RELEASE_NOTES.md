# ResearchClip5.0.2

Windows x64手动安装版本；实际检查及验收范围见[最终报告](https://github.com/wpy-sketch/ResearchClip-Codex/blob/main/docs/qa/FINAL_RELEASE_REPORT.md)。实际仓库为[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。旧5.0.0草稿和Tag保持原样。

## 新增与修复

- 修正内置指南导言及插件发布状态的旧文案；继承5.0.1的左侧浏览器、书签和阅读路线。
- 兼容5.0.0/5.0.1精确组件安装证明，保留模型和设置，重新校验后启用。
- 固定插件打包源文件字节，避免Git换行转换破坏XPI可复现性。

- Google浏览器移到最左侧主导航，直接在资料工作台内浏览公开网页；保留PDF与Markdown编辑会话。
- 本地网页书签：保存当前网页、编辑名称与网址、上移/下移、确认移除、重启后保留，点击一键打开。
- 窄分栏及低高度布局优化；书签与导航共用紧凑栏，切换其他主导航自动隐藏网页。
- 指南按1–38章保留帮助链接，默认章节概览，六条常用路线，287示例与77幅离线图。
- 旧精确OCR清单升级兼容，保留模型字节，经真实CPU自检后启用；非法设置或超出64KiB保留原文件。
- 发布链路按草稿ID读取，转公开后刷新真实资产URL；失败不覆盖已有Tag或资产。

## 兼容与边界

应用5.0.2、SQLite21，新增设置字段有兼容默认值，无新增依赖。Connector仍1.0.1（Zotero10.0.6），OCR包装1.0.1，原始权重/字典不变。仅已验证OCR与Connector进入固定组件目录；不包含未验证的公式/语义模型。

内置网页为独立WebView2，不是完整Chrome；账号与Chrome书签同步由外部Chrome管理。本地书签不跨设备同步；自动更新继续停用，手动安装NSIS并核对SHA-256，无需密钥配置。Google可达性取决于网络。未新增Markdown→PDF或独立备份恢复功能。

本机已通过819项前端测试、当前检查结果见最终报告；上一版366项Rust测试（12忽略）、原生书签/网页、PDF摘录、真实Zotero读取、真实下载与离线CPU OCR。未在另一台干净Windows或物理混合DPI设备验收；当前版未重新运行原生Markdown保存对话框或Zotero写入CRUD，相同服务/插件的上一版真实结果保留在报告。安装包未做Windows代码签名。
