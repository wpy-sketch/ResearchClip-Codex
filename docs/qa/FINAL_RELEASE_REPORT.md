# ResearchClip v5.1.1 发布验收

2026-11-02。本轮仅发布已实现的 Zotero ink 图形显示增量，不实施后续业务阶段。发布仓库为 [ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。当前正在构建与核验，尚未宣称发布成功。

## 修改及架构

功能修改文件、边界和前轮实际验证见 [ink 图形验收](ZOTERO-INK-2026-11-02.md)。沿用现有 Zotero 来源、缓存、PDF.js 投影及 Canvas；列表新增受校验数字生成的 SVG 预览。单条错误恢复画布并继续绘制，其余模块保持兼容。

发布配置同步 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`Cargo.lock`、`tauri.conf.json` 为 5.1.1；`package-builtin.json` 更新四项组件主程序兼容范围和两个外部插件附件 URL。模型、插件和第三方依赖字节未升级。README、CHANGELOG、当前发布/安装/扩展说明同步；[5.1.0 原发布报告](FINAL_RELEASE_REPORT-5.1.0.md)保留。

新增依赖：无；SQLite21，无迁移或权限扩张；自动更新继续关闭。Windows x64 NSIS 手动安装版，安装器无 Windows 代码签名。

## 本轮检查

环境：Windows x64 10.0.26300，Node22.22.2/npm10.9.7，项目既有 Rust/Tauri 工具链。具体执行结果在本轮结束后据实补齐；前轮836项测试及原生 ink 验证不冒充本轮结果。

## 人工验收与已知限制

沿用此前明确接受的本机隔离发布范围；不将本机测试称为独立干净 Windows 验收。真实用户 Zotero 笔迹视觉对照、物理混合DPI/HDR/多屏、真实旧安装器升级和未变模块的完整外部互操作仍需人工验证。原生 fixture、真实宿主及历史测试分别记录，不伪造通过。保留前轮 Vite 大于500kB构建块警告。

不覆盖或删除用户数据、现有 Release、标签和附件；安装测试仅在受管隔离目录执行。未开发 PDF 导出或独立备份恢复。下一步是完成当前版本的检查、安装包和公开下载哈希核验后交付 Release 链接。
