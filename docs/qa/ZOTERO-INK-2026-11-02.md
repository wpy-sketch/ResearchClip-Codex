# Zotero ink 图形显示增量验收

日期：2026-11-02。范围仅为 ResearchClip 中已有 Zotero ink 的列表预览与 PDF 页面显示；不新增编辑、上传、导出或同步功能。

## 实现与文件

- `src/features/pdf/zotero-annotations/InkPreview.tsx`：以现有 `position.paths/width` 生成只读 SVG，保留颜色、多笔画和单点，按 PDF 纵轴翻转并等比居中；包含线帽余量。仅插入受校验数字，不执行外部 SVG/HTML。
- `ZoteroAnnotationsPanel.tsx`、`annotations.css`：分页列表内展示预览，点击原有定位按钮可回到 PDF 原位置；缺失或畸形笔迹显示明确提示。列表继续使用现有有界分页。
- `AnnotationCoordinateAdapter.ts`：复用 PDF.js 页面投影，补充页面尺寸、点对、有限值与线宽变换校验；保留 CropBox、UserUnit、旋转及原始来源身份。
- `drawing.ts`、`DenseAnnotationCanvas.tsx`：失败时恢复画布状态，逐条处理错误，其余批注继续绘制；无画布上下文时显示错误，重绘时清除过期错误。沿用既有分帧绘制、卸载取消及位图预算。
- `ink.test.tsx`：新增 10 项确定性测试，覆盖多笔画、单点、无效输入、四向旋转、裁切偏移、物理线宽和异常后的画布恢复。
- `scripts/zotero-ink-native-smoke.mjs`、`zotero-ink-native-cases.mjs`：复用 `zotero-alltypes-native-smoke.mjs` 的隔离原生启动与 HTTP fixture；`zotero-annotations-fixture.mjs` 增加仅测试模式使用的多笔画/单点图形。

新增依赖：无。SQLite 仍为 schema21，无迁移、IPC、权限或 Rust 业务代码变更；不修改用户真实资料、原图或 Zotero 库。

## 本轮实际检查

环境：Windows x64（Node `os.release()` 为 `10.0.26300`），Intel Core Ultra 7 356H、32 GiB RAM；Node 22.22.2、npm 10.9.7。系统 CIM 信息查询被环境拒绝，以上来自 Node 的只读系统信息。

| 命令 | 结果 |
| --- | --- |
| `npx vitest run src/features/pdf/zotero-annotations/ink.test.tsx src/features/pdf/zotero-annotations/all-types.test.tsx` | 退出 0；2 个文件、16 项测试通过 |
| `npm run check` | 退出 0；typecheck、lint、108 个文件的 836 项测试及生产构建通过 |
| `node scripts/with-rust.mjs tauri build --debug --no-bundle` | 退出 0；实际生成 `src-tauri/target/debug/researchclip.exe` |
| `node scripts/zotero-ink-native-smoke.mjs ink-20261102-a` | 退出 1；沙盒内 CDP 主页面等待超时，未进入功能检查 |
| `node scripts/zotero-ink-native-smoke.mjs ink-20261102-b` | 退出 1；诊断确认 CDP 页面列表为空，未进入功能检查 |
| `node scripts/zotero-ink-native-smoke.mjs ink-20261102-c` | 经授权沙盒外运行，退出 0；6 组原生检查通过，页面异常 0，源 PDF 摘要保持不变，应用正常退出 |
| `npm run check:docs` | 退出 0；249 份文档、1304 个本地链接，代码围栏/权限/迁移一致性检查通过；属于文档检查 |

Vite 仍提示部分构建块超过 500 kB；未因此扩大本轮架构范围。

原生最终报告见[保留证据](evidence/zotero-ink-20261102.json)。详细本地输出为 `artifacts/zotero-ink-native-ink-20261102-c/report.json`，截图为同目录 `ink-preview-and-page.png`，失败记录 a/b 保留。已查看实际截图，列表和页面均显示同一多笔画图形与单点。原生检查包含：

- 原生读取、SQLite 缓存、列表 SVG 和 PDF Canvas 的多笔画/单点，以及列表回跳。
- 0/90/180/270 度、PDF 放大/缩小后的预期坐标红色像素及空白对照点。
- 图层开关、类型筛选后的移除与重绘，以及页面点击聚焦原 Zotero 批注身份。
- HTTP403 后图形保留、本地缓存重载，以及 480/760/1440 CSS 像素宽度不溢出。

## Windows 人工验收及限制

本轮原生自动化使用真实 ResearchClip exe、WebView2、Rust 读取与 SQLite 缓存，Zotero 服务由本地隔离 HTTP fixture 提供；不能称为真实 Zotero 客户端人工验收。测试图形属于明确标记的虚构协议输入，不代表用户资料效果。

未执行：真实 Zotero 中用户笔迹的视觉对照、物理高 DPI/混合 DPI 多屏、macOS。下一步建议使用本次构建打开已确认来源的 PDF，在“Zotero批注”中主动读取并筛选“Zotero笔迹”，人工核对列表图形、PDF 位置及线宽。旧缓存若缺失 paths，应主动重新读取；不能从包围框猜造笔迹。

Rust 格式、单测及 Clippy 本轮未执行：未修改 Rust、存储或 IPC 业务代码。没有生成安装包或覆盖已安装应用。
