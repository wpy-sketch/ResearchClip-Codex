# Step 1 功能与组件核查矩阵

## 5.0.0本轮补充（2026-10-29）

下文Step1为历史快照。当前注册表48项，28项IMPLEMENTED、20项PARTIAL；新增Windows工作台内嵌网页，保留真正Chrome入口。真实Zotero10.0.6五类15项CRUD、Markdown原生保存重开和当前OCR推理已另行验证，不把历史“待验收”当作本轮结果。完整补充、仍缺能力及测试边界见[当前审查](../qa/BROWSER_WORKSPACE_500.md)。尚未验证的模型/扩展不进入当前可安装目录；IDE、HarmonyOS、未来Agent能力未因此自动实施。用户要求本轮先上传源码和Release草稿，独立干净Windows验收保留。

核查对象：当前未提交工作区，应用版本 **4.4.0**，SQLite schema **21**；Git 基点 `ec867a9`。这不是已发布版本或安装包验收声明。本轮没有更新依赖、数据库结构或发布 GitHub Release。既有 v4.4.0 改动保留；本轮变更见 [Step 1 报告](../qa/STEP1_REPORT.md)。

## 状态含义

- IMPLEMENTED：当前源码有完整的该范围实现；不表示所有硬件、外部宿主、安装配置均已验收。
- PARTIAL：已有实现，但仍有明确的范围、能力或外部验收缺口。
- NOT_IMPLEMENTED：当前没有该功能实现。
- BROKEN：已复现且尚未修复的问题。本轮剪藏状态缺陷已修复，不把历史故障永久算作 BROKEN。

当前清单把实现、安装、启用、依赖与实际可用性分开。静态矩阵不能判断用户机器上扩展或模型是否已安装。设置 → 功能与组件 → 展开清单 → **检查本机组件状态**，才执行此清单的整合检查；不会自动下载、安装或同步。原有设置模块仍按各自逻辑读取自身状态。

## 用户要求逐项对应

| 项目 | 判定 | 主要功能 ID / 边界 |
| --- | --- | --- |
| 1 全屏、区域、智能窗口截图 | IMPLEMENTED | capture.fullscreen / capture.region / capture.window；受保护窗口和物理 DPI 待设备验收 |
| 2 编辑、标注、撤销重做、版本历史 | IMPLEMENTED | image.editor / image.revisions；原图不可变，恢复追加修订 |
| 3 滚动长截图 | PARTIAL | capture.scrolling；Chrome/Edge 普通网页 MVP，非任意窗口保证 |
| 4 素材库、标签、搜索、SQLite | IMPLEMENTED | library.sqlite；事务、FTS、文件意图恢复 |
| 5 Markdown 与工具栏 | IMPLEMENTED | document.markdown；源码/投影/预览，非完整富文本编辑器 |
| 6 动态分栏 | IMPLEMENTED | workspace.dynamic；未打开 Markdown 不显示空白 Markdown 面板 |
| 7 图文块排序 | IMPLEMENTED | document.image-notes；图片和注释整体操作 |
| 8 Markdown 创建、保存、打开 | IMPLEMENTED | document.files；受限文件授权、相对 assets |
| 9 LaTeX 与数学定理 | IMPLEMENTED | document.mathematics；保留源码，KaTeX 渲染与稳定标签 |
| 10 PDF 阅读、翻页、搜索、缩放 | IMPLEMENTED | pdf.reader；PDF.js 会话与缓存释放 |
| 11 PDF 内部截图 | IMPLEMENTED | pdf.excerpts；PDF 内部坐标和内容哈希 |
| 12 插入笔记、原文回跳 | IMPLEMENTED | pdf.markdown-links；替换文件不猜测跳转 |
| 13 Zotero 外部打开 | PARTIAL | zotero.external-open；单实例路径处理已有，真实宿主待验收 |
| 14 Zotero 标准批注读取、编辑、同步 | PARTIAL | zotero.annotations-*；当前 PDF 最多 30,000 条，真实 Zotero 测试库仍待验收 |
| 15 OCR 全文检索 | IMPLEMENTED | ocr.windows / ocr.fulltext；Windows OCR 语言资源是前置条件 |
| 16 公式图片转 LaTeX | PARTIAL | math.image-latex；离线适配已实现，模型/运行时与识别准确率有边界 |
| 17 相似图片 | IMPLEMENTED / PARTIAL | image.similarity 为传统检索；image.semantic / image.qwen-assistance 是可选实验适配 |
| 18 网页剪藏 | PARTIAL | clipper.bridge 已实现；浏览器扩展和 Native Messaging Host 手动安装与实浏览器验收独立 |
| 19 DOI、BibTeX、双向链接 | IMPLEMENTED | references.citations / document.backlinks；不把手动输入元数据标为已核实 |
| 20 WebDAV | PARTIAL | sync.webdav；冲突及条件写入实现存在，真实服务端能力待验收 |
| 21 其他实际功能 | 见下表 | 图片导入、双图对比、模板/历史、桌面工具、Connector、统一命令与组件状态页 |

## 六类交付边界

| 分类 | 本轮契约 |
| --- | --- |
| CORE | 随主程序编译。代码存在不代表依赖资源存在或业务已实机验收。 |
| BUILTIN_OPTIONAL | 已编译且有真实启停能力。本轮只允许切换现有剪藏桥和 Connector 桥。 |
| RESOURCE_PACKAGE | Windows OCR 语言、用户指定运行时、受限离线包或外部 Host；不新增通用可执行文件安装器。 |
| MODEL_PACKAGE | 复用现有固定模型清单、导入及显式下载流程。资源存在/长度符合不等于 SHA/推理通过。 |
| EXTERNAL_PLUGIN | 运行在 Zotero 或浏览器内；本页不安装插件，不推断未连接就未安装。 |
| APP_UPDATE_REQUIRED | 本机原生能力必须升级主程序，不能靠资源包补齐。目前自动更新器尚未实现。 |

## 功能定义、版本、依赖与源码证据

应用内定义唯一来源是 [catalog.json](../../src/features/feature-packages/catalog.json)，不是第二套资源 ID。下表应用代码版本取本轮 4.4.0；外部模型/插件取现有清单版本。**所有条目仅声明兼容当前 4.4.0 主程序**，不推定未来版本兼容。源码路径也由单元测试检查。

| 功能 ID / 名称 | 分类 | 实现 | 清单版本 | 依赖 ID | 源码证据与限制 |
| --- | --- | --- | --- | --- | --- |
| `capture.fullscreen` 全屏截图 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/capture/store.ts](../../src/features/capture/store.ts)、[src-tauri/src/services/screen_capture.rs](../../src-tauri/src/services/screen_capture.rs)；单显示器完整画面；混合DPI/HDR需设备验收。 |
| `capture.region` 区域截图 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/capture/geometry.ts](../../src/features/capture/geometry.ts)、[src-tauri/src/services/region_capture.rs](../../src-tauri/src/services/region_capture.rs)；受限于所选显示器；Esc取消和原始像素坐标。 |
| `capture.window` 智能窗口截图 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/capture/CaptureControls.tsx](../../src/features/capture/CaptureControls.tsx)、[src-tauri/src/services/window_capture/mod.rs](../../src-tauri/src/services/window_capture/mod.rs)；Windows Graphics Capture；受保护窗口不承诺可捕获。 |
| `capture.scrolling` 滚动长截图 | CORE | PARTIAL | 4.4.0 | 无 | [src/features/capture/ScrollCapture.tsx](../../src/features/capture/ScrollCapture.tsx)、[src-tauri/src/services/stitching.rs](../../src-tauri/src/services/stitching.rs)；已有拼接与受控滚动流程；具体第三方窗口、动态内容和物理DPI矩阵未完整验收。 |
| `image.editor` 图片编辑、标注与撤销重做 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/editor/store.ts](../../src/features/editor/store.ts)、[src/features/editor/serialization.ts](../../src/features/editor/serialization.ts)；原图不可变，编辑与合成另存。 |
| `image.revisions` 截图版本历史 | CORE | IMPLEMENTED | 4.4.0 | `image.editor`、`library.sqlite` | [src-tauri/src/storage/revisions.rs](../../src-tauri/src/storage/revisions.rs)、[src/features/editor/RevisionHistory.tsx](../../src/features/editor/RevisionHistory.tsx)；恢复追加新修订，不销毁当前版本。 |
| `library.sqlite` 素材库、标签与SQLite搜索 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/library/store.ts](../../src/features/library/store.ts)、[src-tauri/src/storage/library.rs](../../src-tauri/src/storage/library.rs)、[src-tauri/src/storage/search.rs](../../src-tauri/src/storage/search.rs)；按本机服务状态和本轮测试证据判断。 |
| `document.markdown` Markdown源码、投影与工具栏 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/document/MarkdownPanel.tsx](../../src/features/document/MarkdownPanel.tsx)、[src/features/document/formatting.ts](../../src/features/document/formatting.ts)；接近所见即所得的原文投影，不承诺完整富文本往返编辑器。 |
| `workspace.dynamic` 动态分栏与工作区布局 | CORE | IMPLEMENTED | 4.4.0 | `document.markdown` | [src/features/workspace/manager.ts](../../src/features/workspace/manager.ts)、[src/features/document/WorkspaceSplit.tsx](../../src/features/document/WorkspaceSplit.tsx)；没有打开Markdown时不创建空白分栏。 |
| `document.image-notes` 按顺序排列的图片与注释 | CORE | IMPLEMENTED | 4.4.0 | `document.markdown`、`image.editor` | [src/features/document/model.ts](../../src/features/document/model.ts)、[src/features/document/content-model.ts](../../src/features/document/content-model.ts)；按本机服务状态和本轮测试证据判断。 |
| `document.files` Markdown创建、保存与重新打开 | CORE | IMPLEMENTED | 4.4.0 | `document.markdown` | [src/features/document/store.ts](../../src/features/document/store.ts)、[src-tauri/src/services/markdown_document.rs](../../src-tauri/src/services/markdown_document.rs)；单一文件服务，相对assets及外部变更检查。 |
| `document.mathematics` LaTeX、数学定理与编号 | CORE | IMPLEMENTED | 4.4.0 | `document.markdown` | [src/features/document/math-render.ts](../../src/features/document/math-render.ts)、[src/features/document/paper-model.ts](../../src/features/document/paper-model.ts)；扩展语法有源码说明；识别和数学正确性不自动核实。 |
| `pdf.reader` PDF阅读、搜索与缩放 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/pdf/PdfViewer.tsx](../../src/features/pdf/PdfViewer.tsx)、[src/features/pdf/PdfViewerSession.ts](../../src/features/pdf/PdfViewerSession.ts)、[src-tauri/src/services/pdf.rs](../../src-tauri/src/services/pdf.rs)；只读PDF.js；按需渲染、损坏/密码/权限诊断。 |
| `pdf.excerpts` PDF内部选区截图 | CORE | IMPLEMENTED | 4.4.0 | `pdf.reader`、`library.sqlite` | [src/features/pdf/excerpt-geometry.ts](../../src/features/pdf/excerpt-geometry.ts)、[src-tauri/src/commands/pdf_excerpts.rs](../../src-tauri/src/commands/pdf_excerpts.rs)；按本机服务状态和本轮测试证据判断。 |
| `pdf.markdown-links` 截图插入Markdown与来源回跳 | CORE | IMPLEMENTED | 4.4.0 | `pdf.excerpts`、`document.files` | [src/features/pdf/navigateExcerpt.ts](../../src/features/pdf/navigateExcerpt.ts)、[src-tauri/src/storage/pdf_notes.rs](../../src-tauri/src/storage/pdf_notes.rs)；稳定ID与内容哈希；替换PDF时拒绝猜测位置。 |
| `zotero.external-open` Zotero外部阅读器打开PDF | CORE | PARTIAL | 4.4.0 | `pdf.reader` | [src-tauri/src/services/pdf_open.rs](../../src-tauri/src/services/pdf_open.rs)、[src-tauri/src/services/zotero_local.rs](../../src-tauri/src/services/zotero_local.rs)；已实现单实例argv和来源核对，真实Zotero外部阅读器链路仍需宿主验收。 |
| `zotero.annotations-read` Zotero标准批注读取 | CORE | PARTIAL | 4.4.0 | `pdf.reader` | [src-tauri/src/services/zotero_annotations.rs](../../src-tauri/src/services/zotero_annotations.rs)、[src/features/pdf/zotero-annotations/Repository.ts](../../src/features/pdf/zotero-annotations/Repository.ts)；当前PDF最多30,000条；五类几何，未知/ink保留诊断，真实宿主矩阵待验。 |
| `zotero.annotations-write` Zotero标准批注编辑 | CORE | PARTIAL | 4.4.0 | `zotero.annotations-read` | [src-tauri/src/services/zotero_write_transport.rs](../../src-tauri/src/services/zotero_write_transport.rs)、[src/features/pdf/zotero-annotations/Writer.ts](../../src/features/pdf/zotero-annotations/Writer.ts)；仅已检测且授权的类型；创建后复读，真实宿主授权/写入仍待验。 |
| `zotero.annotations-sync` Zotero标准批注同步 | CORE | PARTIAL | 4.4.0 | `zotero.annotations-write` | [src-tauri/src/storage/zotero_sync.rs](../../src-tauri/src/storage/zotero_sync.rs)、[src/features/pdf/zotero-annotations/ZoteroSyncPanel.tsx](../../src/features/pdf/zotero-annotations/ZoteroSyncPanel.tsx)；六态、持久修改、条件写入和冲突历史；不混入私有笔迹。 |
| `ocr.windows` Windows OCR文字识别 | CORE | IMPLEMENTED | 4.4.0 | `resource.ocr-languages` | [src/features/ocr/api.ts](../../src/features/ocr/api.ts)、[src-tauri/src/services/ocr.rs](../../src-tauri/src/services/ocr.rs)；Windows系统语言包决定可用语言。 |
| `ocr.fulltext` OCR全文索引与检索 | CORE | IMPLEMENTED | 4.4.0 | `ocr.windows`、`library.sqlite` | [src/features/ocr-index/api.ts](../../src/features/ocr-index/api.ts)、[src-tauri/src/storage/ocr.rs](../../src-tauri/src/storage/ocr.rs)；按本机服务状态和本轮测试证据判断。 |
| `math.image-latex` 公式图片转LaTeX适配器 | CORE | PARTIAL | 4.4.0 | `resource.python-runtime`、`model.latex` | [src/features/math-ocr/api.ts](../../src/features/math-ocr/api.ts)、[src-tauri/src/services/math_ocr.rs](../../src-tauri/src/services/math_ocr.rs)；已有离线推理/校对流程；样本存在误识别，不保证数学正确。 |
| `image.similarity` 相似图片与完全重复检索 | CORE | IMPLEMENTED | 4.4.0 | `library.sqlite` | [src/features/similarity/api.ts](../../src/features/similarity/api.ts)、[src-tauri/src/services/similarity.rs](../../src-tauri/src/services/similarity.rs)；传统哈希检索无需AI模型；不自动删除。 |
| `image.semantic` 本地语义检索适配器 | CORE | PARTIAL | 4.4.0 | `resource.python-runtime`、`model.semantic` | [src/features/semantic/api.ts](../../src/features/semantic/api.ts)、[src-tauri/src/services/embeddings.rs](../../src-tauri/src/services/embeddings.rs)；依赖已授权本地运行时和模型；结果需人工判断。 |
| `image.qwen-assistance` Qwen图片辅助比较适配器 | CORE | PARTIAL | 4.4.0 | `resource.qwen-runtime`、`model.qwen` | [src/features/similarity/LocalAi.tsx](../../src/features/similarity/LocalAi.tsx)、[src-tauri/src/services/similarity_ai.rs](../../src-tauri/src/services/similarity_ai.rs)；实验性双图辅助，不是删除或数学判断依据。 |
| `clipper.bridge` 网页剪藏本机桥 | BUILTIN_OPTIONAL | IMPLEMENTED | 4.4.0 | 无 | [src/features/clipper/store.ts](../../src/features/clipper/store.ts)、[src-tauri/src/services/clipper.rs](../../src-tauri/src/services/clipper.rs)；启用本机桥不代表浏览器扩展已安装。 |
| `plugin.browser-clipper` 浏览器网页剪藏扩展 | EXTERNAL_PLUGIN | PARTIAL | 1.0.2 | `clipper.bridge`、`resource.clipper-host` | [browser-extension/manifest.json](../../browser-extension/manifest.json)、[browser-extension/src/popup.ts](../../browser-extension/src/popup.ts)；需要手动安装和Native Messaging配置；本机桥不能检测浏览器安装状态。 |
| `references.citations` DOI、BibTeX与本地引用 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/references/citation-model.ts](../../src/features/references/citation-model.ts)、[src-tauri/src/services/references.rs](../../src-tauri/src/services/references.rs)；本地录入/导入不等于已核实DOI元数据。 |
| `document.backlinks` 双向链接、图谱与文档组织 | CORE | IMPLEMENTED | 4.4.0 | `document.files` | [src/features/links/indexing.ts](../../src/features/links/indexing.ts)、[src-tauri/src/storage/links.rs](../../src-tauri/src/storage/links.rs)；按本机服务状态和本轮测试证据判断。 |
| `sync.webdav` WebDAV资源同步 | CORE | PARTIAL | 4.4.0 | `library.sqlite` | [src/features/sync/api.ts](../../src/features/sync/api.ts)、[src-tauri/src/services/sync/runtime.rs](../../src-tauri/src/services/sync/runtime.rs)；配置和凭据分别核对；真实坚果云强条件写入仍待验证，不自动测试或上传。 没有独立持久总开关，不能把停止任务伪称卸载或停用模块。 |
| `zotero.connector-bridge` Zotero Connector本机桥 | BUILTIN_OPTIONAL | PARTIAL | 4.4.0 | 无 | [src/features/pdf/connector.ts](../../src/features/pdf/connector.ts)、[src-tauri/src/services/zotero_connector/runtime.rs](../../src-tauri/src/services/zotero_connector/runtime.rs)；默认关闭，配对短时能力；真实XPI安装/配对仍待验。 |
| `plugin.zotero-connector` ResearchClip Zotero Connector XPI | EXTERNAL_PLUGIN | PARTIAL | 1.0.0 | `zotero.connector-bridge` | [zotero-connector/manifest.json](../../zotero-connector/manifest.json)、[zotero-connector/connector.js](../../zotero-connector/connector.js)；未连接不能推断未安装；配对版本仅表示观察到运行中的协议端。 |
| `resource.ocr-languages` Windows OCR语言资源 | RESOURCE_PACKAGE | IMPLEMENTED | Windows-managed | 无 | [src/features/ocr/model.ts](../../src/features/ocr/model.ts)、[src-tauri/src/services/ocr.rs](../../src-tauri/src/services/ocr.rs)；系统资源由Windows管理；本应用不下载系统DLL。 |
| `resource.python-runtime` 用户指定的本地Python运行时 | RESOURCE_PACKAGE | PARTIAL | user-managed | 无 | [math-ocr/README.md](../../math-ocr/README.md)、[src-tauri/src/services/math_ocr.rs](../../src-tauri/src/services/math_ocr.rs)；已配置路径不证明Python模块安装完整；只复用已有受限选择/执行流程。 |
| `model.latex` RapidLaTeXOCR模型 | MODEL_PACKAGE | IMPLEMENTED | rapid-latex-ocr-onnx-v0 | 无 | [math-ocr/models.json](../../math-ocr/models.json)；现有导入/显式下载按固定清单校验，状态只证明大小/存在性，任务执行再核验SHA。 |
| `model.semantic` 多语言CLIP语义模型 | MODEL_PACKAGE | IMPLEMENTED | clip-b32-multilingual-int8-v1 | 无 | [semantic-search/models.json](../../semantic-search/models.json)；固定模型清单，SHA在任务时校验。 |
| `resource.qwen-runtime` Qwen离线捆绑运行时 | RESOURCE_PACKAGE | PARTIAL | b11748 | 无 | [src-tauri/resources/similarity-ai-manifest.json](../../src-tauri/resources/similarity-ai-manifest.json)；目录检查不等于全部DLL完整；执行前由原服务验证完整白名单和SHA，不开放任意程序。 |
| `model.qwen` Qwen3.5 4B GGUF与视觉投影 | MODEL_PACKAGE | PARTIAL | e87f176479d0855a907a41277aca2f8ee7a09523 | 无 | [src-tauri/resources/similarity-ai-manifest.json](../../src-tauri/resources/similarity-ai-manifest.json)；实验性；仅复用完整离线包选择，无新增任意下载/执行器。 |
| `commands.framework` 统一命令、事件、权限和取消任务 | CORE | PARTIAL | 4.4.0 | 无 | [src/features/commands/platform/registry.ts](../../src/features/commands/platform/registry.ts)、[src/features/commands/platform/runtime.ts](../../src/features/commands/platform/runtime.ts)；部分示例已接入；annotation.create/代码/Agent仍返回MODULE_UNAVAILABLE。 |
| `app.auto-updater` 应用内自动更新器 | APP_UPDATE_REQUIRED | NOT_IMPLEMENTED | not-implemented | 无 | [src-tauri/Cargo.toml](../../src-tauri/Cargo.toml)；未注册Tauri updater能力；需要后续受控主程序升级，不能下载EXE绕过权限。 |
| `image.import` 本地图片、剪贴板和拖放导入 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/import/api.ts](../../src/features/import/api.ts)、[src-tauri/src/services/image_import.rs](../../src-tauri/src/services/image_import.rs)；按本机服务状态和本轮测试证据判断。 |
| `image.compare` 双图对比 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/compare/CompareView.tsx](../../src/features/compare/CompareView.tsx)、[src/features/compare/model.ts](../../src/features/compare/model.ts)；按本机服务状态和本轮测试证据判断。 |
| `document.organization` 模板、目录、收藏与文档历史 | CORE | IMPLEMENTED | 4.4.0 | `document.files` | [src/features/document/DocumentHistory.tsx](../../src/features/document/DocumentHistory.tsx)、[src/features/document/templates.ts](../../src/features/document/templates.ts)、[src-tauri/src/storage/document_organization.rs](../../src-tauri/src/storage/document_organization.rs)；按本机服务状态和本轮测试证据判断。 |
| `desktop.tools` 贴图、快捷键、托盘与显示设置 | CORE | PARTIAL | 4.4.0 | 无 | [src/features/pins/PinView.tsx](../../src/features/pins/PinView.tsx)、[src-tauri/src/services/settings.rs](../../src-tauri/src/services/settings.rs)、[src-tauri/src/services/desktop.rs](../../src-tauri/src/services/desktop.rs)；已有原生实现；托盘/快捷键按各自运行时状态显示，混合DPI和具体桌面环境需人工验收。 |
| `app.feature-catalog` 功能与可选组件清单 | CORE | IMPLEMENTED | 4.4.0 | 无 | [src/features/feature-packages/FeatureRegistry.ts](../../src/features/feature-packages/FeatureRegistry.ts)、[src/features/feature-packages/PackageManager.ts](../../src/features/feature-packages/PackageManager.ts)；只复用本机状态服务和既有桥开关，不是动态代码安装器。 |
| `resource.clipper-host` 浏览器Native Messaging Host | RESOURCE_PACKAGE | PARTIAL | 1.0.2 | 无 | [browser-extension/native-host/Cargo.toml](../../browser-extension/native-host/Cargo.toml)、[browser-extension/README.md](../../browser-extension/README.md)；独立扩展包的Host需手动注册；当前没有只读安装检测接口，不能由本机监听器推断已安装。 |

## 运行时状态与具体错误

| 字段 | 含义 |
| --- | --- |
| installed | YES / NO / UNKNOWN；未检测保留 UNKNOWN，不据连接失败假报未安装。 |
| enabled | YES / NO / UNKNOWN / NOT_APPLICABLE；资源模型不是应用开关。 |
| installedVersion | 实际服务报告的版本；没有报告则 null。清单版本不能伪装成本地版本。 |
| compatibleAppVersions | 本轮只支持主程序 4.4.0；不匹配则拒绝状态操作和开关。 |
| availability | READY / BLOCKED / UNVERIFIED；PARTIAL 绝不自动成为 READY。READY 也不是全硬件验收声明。 |
| checkedAt | 本机检查时间；取消后迟到响应不替换有效结果。 |
| issues | 可查询错误码和可读原因；失败保留 UNKNOWN 和诊断，不制造未安装状态。 |

错误码：`NOT_CHECKED`、`DESKTOP_REQUIRED`、`PROBE_FAILED`、`DEPENDENCY_MISSING`、`DEPENDENCY_UNVERIFIED`、`DISABLED`、`NOT_CONFIGURED`、`AUTH_REQUIRED`、`INTEGRITY_ON_USE`、`INSTALLATION_UNKNOWN`、`HOST_VERIFICATION_REQUIRED`、`APP_UPDATE_REQUIRED`、`APP_VERSION_UNSUPPORTED`、`LAST_JOB_FAILED`（预留，当前聚合器不推断任务结果）。

现有接口只能证明 Python 路径已配置，不能证明模块完整；Qwen 入口存在不能证明完整 DLL 清单有效；模型文件在任务前由原生服务核验 SHA。浏览器扩展/Host 安装状态缺少只读接口，保持 UNKNOWN。Connector 的配对版本仅证明观察到协议端，不代替真实 XPI 菜单和生命周期验收。WebDAV 检查只读本地配置/凭据，不连接或上传。

## 发布工具与未实现范围

仓库已有 GitHub Releases 发布、历史附件继承、摘要校验和安装验证脚本（如 [publish-desktop-release.py](../../scripts/publish-desktop-release.py)）；这是仓库发布工具，不是假装安装在用户应用内的自动更新功能。本轮不生成新正式安装包，不推送标签或发布。

统一命令框架中 `annotation.create` 仍为 `MODULE_UNAVAILABLE`；这与专用 Zotero 批注 UI 中的受控读写能力是两个不同入口。`codeWorkspace.open`、`agent.requestEdit` 也为未提供模块；IDE 已取消，不在本轮补齐。私有手写数据类型不等于 HarmonyOS 真机能力已经实现。自动更新器、通用动态原生插件安装器均未实现。

不包含 Markdown→PDF 导出或独立备份/恢复产品。数据库迁移前的安全快照是既有升级保护，不是增加独立备份系统。
