# ResearchClip Step 1：功能集成、修复与发布准备报告

本轮范围：审查当前项目、修复可复现缺陷、整合功能与可选组件清单，执行现有检查与隔离 Windows 开发版验证。**没有进入 Step 2，没有发布 Release、修改版本号、生成新正式安装包或安装第三方插件。**

## 1. 核查对象与结论

环境：Windows 11 build 26300、Node 22.22.2、npm 10.9.7、Rust/Cargo 1.99.0、现有 MSVC 工具链与 WebView2。应用源码版本为 4.4.0，SQLite schema 21。Git 基点为 `ec867a9`；开始时已有大量 v4.4.0 未提交修改，本轮全部保留。不能将整份 `git diff HEAD` 当成本轮新增功能。

已核对 AGENTS.md、README、PRD、ARCHITECTURE、既有架构/验收记录与 Git 历史；检查前端入口、模块状态与服务接口、Rust 命令/存储实现、001–021 SQL 迁移及迁移执行器。采用当前实现和实际检查结果，而非照搬历史计划。

21 类用户要求的逐项判定、模块源码证据、依赖、版本和能力限制见 [FEATURE_MATRIX.md](../release/FEATURE_MATRIX.md)。组件清单共 **46 项：27 IMPLEMENTED、18 PARTIAL、1 NOT_IMPLEMENTED**。此数量包含模型/资源/外部插件条目，不能当作 46 项全部实机验收通过。支持 BROKEN 分类；本轮已复现的剪藏缺陷修复后不再列作未解决 BROKEN。

核心截图、编辑/历史、素材库、Markdown、PDF、摘录关联、传统相似检索、引用和链接已有实现。Zotero、浏览器剪藏、WebDAV、离线模型相关能力保留明确的范围和外部验收限制。未发现本轮检查仍可复现的核心数据丢失或越权 P0/P1；此结论只覆盖所执行的检查，不代替所有硬件及第三方环境验收。

## 2. 本轮修复

| 缺陷 | 修复与证据 |
| --- | --- |
| 剪藏状态刷新先发后到，可能覆盖用户稍后的连接操作，显示相反状态 | generation 校验丢弃旧刷新；连接期间跳过额外刷新。回归用例先复现再修复。 |
| 剪藏状态成功恢复后，旧错误仍残留 | 有效状态读取清除旧错误；失败保留此前有效状态。 |
| 原生剪藏状态缺少运行时结构校验 | 校验 enabled、扩展 ID 与 last 数据；异常响应显示错误，不把无效 flag 当作已启用。 |
| 未读取剪藏状态就显示“未启用” | 初始显示“状态未读取”，区分未知与关闭。 |

前三项新增回归用例在修复前实际 **3 失败 / exit 1**；修复后连同原有剪藏测试 **5 通过 / exit 0**。前者是保留的故障复现证据，不是删去或跳过断言。日志位于 `artifacts/step1-20261027/clipper-regressions-before.log` 与 `clipper-regressions-after.log`。

初次类型检查发现新测试引用 Node 内建类型未声明（TS2591）；添加测试文件的现有 Node 类型引用后，最终完整类型检查通过，没有升级依赖。

## 3. 模块与组件设计

新增轻量 `FeatureRegistry` 和 `PackageManager`，复用现有模块化单体及服务，不引入全局状态框架、DI 容器、微服务或第二套文件/Markdown/资源 ID 系统。

- `catalog.json` 是静态能力定义的单一来源，记录功能 ID、分类、实现状态、清单版本、依赖、原生探测种类、证据与限制。注册阶段拒绝重复 ID、缺失依赖、循环依赖和伪造可启停分类。
- `PackageManager` 分别聚合安装、启用、实际观察版本、兼容应用版本、可用性及错误码。浏览器预览、探测失败、模型仅存在或外部插件未连接时，不伪装成安装/运行成功。PARTIAL 不能自动成为 READY。
- `native.ts` 只读取现有 OCR、数学 OCR、语义模型、Qwen、剪藏、Connector、WebDAV 状态接口。同一次检查合并重复的在途读取；取消或销毁忽略迟到响应。状态检查不下载、执行模型或连接 WebDAV。
- 仅两种既有本机桥提供真实启停：剪藏桥、Connector 桥。操作后重新读取原服务确认；不开放任意命令、路径、SQL、DLL、EXE 或脚本安装接口。
- 设置页新增“功能与组件”，清单默认折叠、手动检查，支持类型筛选、ID/名称查找、依赖和错误展开。未查询外部安装时保留 UNKNOWN。不同应用版本禁止操作，不假设未来兼容。

资源和模型沿用原模块的固定清单、受控导入或显式下载流程。新 PackageManager 是状态与受控操作门面，**不是已实现通用包下载/安装器**。缺少浏览器扩展/Host 的只读安装探测接口，因此清单不提供伪造检测。没有注册 Tauri updater，应用内自动更新器明确 NOT_IMPLEMENTED / APP_UPDATE_REQUIRED。

## 4. 本轮修改文件

以下按开始时的 1,241 文件 SHA-256 清单比较，不混入此前未提交开发：

| 文件 | 用途 |
| --- | --- |
| [FeatureRegistry.ts](../../src/features/feature-packages/FeatureRegistry.ts)、[catalog.json](../../src/features/feature-packages/catalog.json)、[types.ts](../../src/features/feature-packages/types.ts) | 定义、校验、六类交付与四类实现状态 |
| [PackageManager.ts](../../src/features/feature-packages/PackageManager.ts)、[native.ts](../../src/features/feature-packages/native.ts) | 本机状态聚合、依赖、取消与受控桥启停 |
| [FeaturePackages.tsx](../../src/features/feature-packages/FeaturePackages.tsx)、[packages.css](../../src/features/feature-packages/packages.css) | 设置内组件清单与窄窗口布局 |
| [registry.test.ts](../../src/features/feature-packages/registry.test.ts) | 14 项清单、依赖、失败、取消、版本与权限测试 |
| [clipper/store.ts](../../src/features/clipper/store.ts)、[Clipper.tsx](../../src/features/clipper/Clipper.tsx)、[status-regression.test.ts](../../src/features/clipper/status-regression.test.ts) | 修复与 3 项回归用例 |
| [SettingsView.tsx](../../src/features/settings/SettingsView.tsx) | 接入设置分类与面板 |
| [feature-packages-native-smoke.mjs](../../scripts/feature-packages-native-smoke.mjs) | 独立进程、隔离数据的 Windows WebView2/Rust 验证 |
| [FEATURE_MATRIX.md](../release/FEATURE_MATRIX.md)、本报告、[step1-checks.json](evidence/step1-checks.json)、[step1-native.json](evidence/step1-native.json) | 可复核的审查与检查记录 |

新增依赖：**无**。本轮没有改动 package.json、锁文件、Rust 源码、原生权限、数据库迁移或应用图标；其中已有的 v4.4.0 改动保留原样。本轮没有移动、删除或覆盖既有用户资料。

## 5. 实际运行的检查

本轮日志：`artifacts/step1-20261027/`。汇总见 [step1-checks.json](evidence/step1-checks.json)。

| 命令 | 本轮结果 | 证据 |
| --- | --- | --- |
| `npm run check` | exit 0 | `check-final.log`；顺序运行类型检查、ESLint、Vitest、前端构建 |
| `npm run typecheck`（由 check 执行） | exit 0 | TypeScript 项目检查 |
| `npm run lint`（由 check 执行；新增脚本后再运行一次） | exit 0 | `lint-final.log`，零警告 |
| `npm test`（由 check 执行） | exit 0；98 文件、779 测试通过 | 新增 17 项测试，包含故障复现修复与组件管理核心逻辑 |
| `npm run build`（check 及 Tauri beforeBuild 执行） | exit 0 | Vite 前端构建；没有为消除构建提示而升级全部依赖 |
| `npm run format:rust` | exit 0 | `rust-fmt.log`，检查而非自动重排已有源码 |
| `npm run test:rust` | exit 0；327 通过、8 ignored、0 失败 | `rust-tests.log`；未把 ignored 算作通过 |
| `npm run check:rust` | exit 0 | `rust-clippy.log`，all-targets、`-D warnings` |
| `node scripts/with-rust.mjs cargo test --manifest-path src-tauri/Cargo.toml --locked migration` | exit 0；22 通过 | `migration-tests.log`；新库/旧库升级、失败回滚、未来版本拒绝及资料保持 |
| `npm run tauri -- build --debug --no-bundle` | exit 0 | `tauri-debug-build.log`；实际 Windows 开发版/custom-protocol，可执行文件已生成 |
| `node scripts/feature-packages-native-smoke.mjs step1-a` | exit 0；8 项检查通过、0 pageerror | 实际隔离 Tauri + WebView2 + Rust IPC；[原生结果](evidence/step1-native.json) |
| `npm run check:docs` | exit 0；228 份文档、1,236 个本地链接 | 本地链接、围栏与既有文档/源码契约检查；不是运行验收 |
| 默认资料只读 SHA 审计 | exit 0；32 个文件，0 改变 | `scripts/default-data-audit.ps1`，排除 WebView 缓存；不恢复或改写资料 |

原生测试使用仓库 artifacts 下新建的专门设置与 SQLite 目录，通过 `RESEARCHCLIP_TEST_SETTINGS_DIR` 隔离；没有选择真实 PDF、修改真实 Zotero 配置、重置 AppData 或触发系统全屏截图。测试通过自己的 WebView2 调试接口操作本次启动的测试程序，不是浏览器模拟代替原生。资料审计首次在 Windows PowerShell 5.1 中因缺少 `Path.GetRelativePath` 失败；保留失败日志，随后用 PowerShell 7.6.5 运行原脚本通过，没有修改或恢复用户文件。

8 项原生检查覆盖：schema21 初始化；46 条及六类定义；外部安装未知与更新器未实现；剪藏桥真实启停；Connector 桥真实启停；类型/ID 筛选；480/760/1440 宽度及 75–200% CSS 缩放；Markdown 草稿、关闭确认及无空白分栏。CSS 缩放结果不能代替物理 Windows DPI、HDR 或多屏验证。

## 6. 未执行、未完成与风险

Rust 的 **8 ignored**：7 项需要独立 HTTP/TLS WebDAV 服务，1 项要求显式标记的独立真实 Zotero 配置。本轮未启动这些外部验收环境。所用 Python 检查显示未安装 `wsgidav`；未为本轮擅自安装依赖。模拟服务、单元测试和先前报告均不替代真实第三方兼容性验收。

尚未完成或不能确认：

1. 实际 Zotero XPI 安装、右键菜单、配对、外部阅读器启动、批注读写权限和双向同步生命周期。此前用户要求桌面可操作并验收后再发布，仍保留此前置条件。本轮不操作真实 Zotero。
2. 当前 PDF 30,000 条批注实现与测试是此前已有工作，源码和本轮相关全套测试保留；不能宣称任意真实 Zotero 文献均已验收。未知/私有 ink 保留诊断，不转成标准批注。
3. 浏览器扩展与 Native Messaging Host 的真实安装检测接口缺失；当前清单诚实显示 UNKNOWN，需实际 Chrome/Edge 验收。
4. WebDAV 真实服务端 ETag/原子条件写入、TLS/凭据/离线恢复；本机配置存在不证明远端兼容。
5. 公式 OCR、语义模型、Qwen 推理依赖用户授权运行时与固定模型包，数学准确率需人工校对。状态读取不启动模型。
6. `annotation.create` 统一命令仍返回 MODULE_UNAVAILABLE；专用 Zotero 批注 UI 的实现不能替代此入口。代码/Agent 命令也是未提供模块；IDE 已取消，不在本轮开发。
7. 滚动截图仅浏览器 MVP；物理 DPI、HDR、保护窗口、多屏、托盘/全局快捷键组合及安装升级仍需人工矩阵。没有增加 Markdown→PDF 或独立备份/恢复系统。
8. 应用内自动更新器未实现。此前生成的 v4.4.0 review 安装包不包含本轮新增清单和修复，不能拿旧包当成本轮交付安装包；本轮只验证开发版。

## 7. Windows 人工验收清单与后续前置条件

1. 在独立资料目录运行本轮构建，检查设置 → 功能与组件，未知/缺失资源/不匹配版本和错误状态是否明确；确认原有模块设置入口可用。
2. 用真实设备验证全屏、区域、窗口、浏览器滚动截图；100/125/150/200% 物理 DPI、多屏/HDR、Esc 取消及快捷键冲突。
3. 用专门测试 PDF 检查旋转、文本复制、搜索、高清选区截图、相对 assets、Markdown 重开和内容哈希不一致提示。
4. 桌面可操作后，在独立 Zotero 测试配置安装 XPI，执行菜单/配对/取消、用户库/群组库、30,000 条分页、创建/读取/更新/删除、冲突与重启恢复；不得使用真实资料库做破坏性测试。
5. 使用独立 WebDAV 服务和明确授权的模型包验证外部能力；在真实浏览器安装可信扩展与 Host，验证重连和剪藏。
6. 正式发布前，重新构建安装包、执行隔离安装/升级验收、核验所有附件与摘要。需新的发布指令及满足既有 Zotero 验收条件；Step 1 完成后停止，不自动进入 Step 2。
