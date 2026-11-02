# ResearchClip v5.1.1 · Zotero ink 图形显示

2026-11-02；Windows x64 手动安装版。下载同页 `ResearchClip_5.1.1_x64-setup.exe`，使用 `SHA256SUMS` 核对文件。

## 本版改动

- Zotero 批注列表显示 ink 手绘图形，保留多笔画、单点、原颜色和等比形状；点击预览所在按钮可定位到 PDF 原位置。
- PDF 页面的笔迹继续使用原始路径与线宽，支持缩放和四向旋转；补充无效坐标与线宽变换校验。
- 单条批注绘制失败时恢复画布状态，其余批注继续显示；缺失笔迹或画布不可用时显示明确提示。

打开已确认来源的 PDF → “Zotero批注” → “读取Zotero批注（只读）” → 类型选择“Zotero笔迹”。旧缓存缺少笔画数据时请重新读取。

## 安装、组件与兼容性

沿用 v5.1.0 的浏览器、OCR、素材库、Markdown、搜索和四项固定组件；无新增依赖、权限或 SQLite 迁移，仍为 schema21。OCR/语义模型和 Connector/网页剪藏包保留原始已核验字节，仅更新主程序兼容范围及本版下载链接。默认不上传本地资料。

自动更新继续关闭，使用手动 NSIS 安装升级；安装器未做 Windows 代码签名，不生成虚假签名或自动更新元数据。没有新增 PDF 导出、独立备份恢复或其他后续业务功能。

本机隔离原生测试与真实 Zotero/物理硬件验收分开记录；真实用户笔迹视觉对照、独立干净 Windows 和物理混合 DPI/多屏仍属于人工范围。实际命令、结果、安装器哈希及公开附件核验见[发布报告](https://github.com/wpy-sketch/ResearchClip-Codex/blob/main/docs/qa/FINAL_RELEASE_REPORT.md)，功能细节见[ink 验收](https://github.com/wpy-sketch/ResearchClip-Codex/blob/main/docs/qa/ZOTERO-INK-2026-11-02.md)。

此前公开版本和历史附件继续保留：[v5.1.0](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/v5.1.0)、[历史归档](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/researchclip-history-20261030)。
