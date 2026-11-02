# ResearchClip v5.1.1 发布验收

2026-11-02。本轮仅发布已实现的 Zotero ink 图形显示增量，不实施后续业务阶段。发布仓库为 [ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。**v5.1.1 已正式发布且为最新稳定版**；34份附件全部经过完整草稿回读及匿名公开下载SHA-256核验。

## 修改及架构

功能修改文件、边界和前轮实际验证见 [ink 图形验收](ZOTERO-INK-2026-11-02.md)。沿用现有 Zotero 来源、缓存、PDF.js 投影及 Canvas；列表新增受校验数字生成的 SVG 预览。单条错误恢复画布并继续绘制，其余模块保持兼容。

发布配置同步 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`Cargo.lock`、`tauri.conf.json` 为 5.1.1；`package-builtin.json` 更新四项组件主程序兼容范围和两个外部插件附件 URL。模型、插件和第三方依赖字节未升级。README、CHANGELOG、当前发布/安装/扩展说明同步；[5.1.0 原发布报告](FINAL_RELEASE_REPORT-5.1.0.md)保留。

新增依赖：无；SQLite21，无迁移或权限扩张；自动更新继续关闭。Windows x64 NSIS 手动安装版，安装器无 Windows 代码签名。

## 本轮检查

环境：Windows x64 10.0.26300，Node22.22.2/npm10.9.7、Rust1.99.0、Tauri2.12.3。本轮重新执行以下检查，退出码均为0；与前轮记录分开。

- `npm ci` 干净安装327个包；`npm run check`：108个文件、836项测试，类型检查、ESLint、生产前端构建通过。
- `npm run format:rust`、`npm run test:rust`、`npm run check:rust`：370项通过、12项专用用例按原配置忽略；包括数据库迁移、回滚与数据保持。
- 发布门禁、手动发布、公开校验、XPI字节重现及模型传输的23项Python测试通过；`npm run check:docs`通过，文档检查不等同运行验收。
- `tauri build --debug --no-bundle` 和 `build_manual_release.py` 均实际成功；生产优化NSIS安装器22.40 MiB，32份内容文件加2份元数据，全部本地大小/SHA通过。
- 当前版本8套本机原生检查通过，详情见[原生事实摘要](evidence/v511-native-summary.json)：ink6组、六类批注/30,000条fixture8组、Markdown7组、PDF摘录12组、离线OCR5组、真实隔离Zotero10.0.6只读3组、既有已发布剪藏Host7组、安装器6组。模式、原始报告路径和SHA分别保留。
- 实际安装/卸载只使用受管中文/空格路径，验证安装载荷、忙碌拒绝、许可证/运行时、PDF默认程序保持及既有AppData哈希；未向用户真实资料启动主程序。

源码构建提交为 `9c31486df8f99193f95fe309cae8a8b37af07c7a`；安装器 SHA-256 为 `af39eeaf503f34d5be350fadb94ab3bbc07a0bb3ad434cb1d74b6256f6c6a80f`。完整构建与接受范围见[精确发布回执](evidence/v511-release-ready.json)。

失败和修复保留：干净安装首轮因沙盒npm缓存EPERM失败，授权后成功；Rust链接多次因历史生成PDB写入LNK1201失败，保留原调试文件后重新生成，完整测试与Clippy成功。没有移除测试、降低安全断言或删除用户资料。前轮沙盒WebView2页面空列表的失败亦保留。

## 公开版本与完整性

- [正式 Release](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/v5.1.1)
- [Windows x64 安装器](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.1/ResearchClip_5.1.1_x64-setup.exe)：23,490,708字节；SHA见上文。
- [SHA256SUMS](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.1/SHA256SUMS)
- [全部34份附件的大小、SHA与真实下载URL](evidence/v511-publication.json)

v5.1.0的Release身份、标签目标及34份附件的ID/名称/大小/SHA均与发布前快照一致；未覆盖旧版。v5.1.1标签仍指向本机已验收的构建提交9c31486，后续main仅增加QA文档。安装包未被云端重新构建的版本替换。

## 发布链路与CI证据

[标签Windows CI](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41377478128)的validate和NSIS build均成功，[CI事实摘要](evidence/v511-ci.json)区分本机安装器与单独CI产物。自动publish任务按原工作流设计跳过；本版使用本机已验收的原始安装器。

本地上传依次遇到大请求停滞、HTTPS写入超时、curl超时和Git HTTP408；小请求也有可重试HTTP400。失败日志和本地32/34份完整草稿回读记录位于 `artifacts/release-v511-20261102/`，没有把本地缺失的两份大型文件称为下载通过。GitHub连接器无Git blob写权限，使用既有Git凭据的受限发布通道，凭据未写入文件或日志。

两个固定模型先由[既有受限传输](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41378144061)补缺。剩余13份精确附件经101个有界片段传到隔离分支；逐块内容SHA和重组后的完整SHA均验证，已有附件拒绝覆盖。[原样重组补传](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41382078512)成功，隔离分支仅涉及工作流、字节属性和本版传输文件，未合并main、未变更应用源码或版本标签。

最终由[发布及匿名核验任务](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41383227952)重新完整下载全部34份草稿文件并核对大小/SHA，公开后在不携带下载授权头的情况下重新下载全部34份公开文件，全部通过。该真实网络校验在GitHub runner执行，本机32份回读与云端全量结果分别披露；不能将其当作独立Windows应用人工验收。传输/最终发布的12项确定性边界测试通过，实际发布不是模拟器或mock结果。

## 人工验收与已知限制

沿用此前明确接受的本机隔离发布范围；不将本机测试称为独立干净 Windows 验收。真实用户 Zotero 笔迹视觉对照、物理混合DPI/HDR/多屏、真实旧安装器升级和未变模块的完整外部互操作仍需人工验证。原生 fixture、真实宿主及历史测试分别记录，不伪造通过。保留前轮 Vite 大于500kB构建块警告。

不覆盖或删除用户数据、现有 Release、标签和附件；安装测试仅在受管隔离目录执行，181份既有AppData文件哈希未变。未开发 PDF 导出或独立备份恢复。下一步建议按同页SHA核对后手动升级，在真实资料中人工对照笔迹及物理DPI；这些建议不构成继续开发授权。
