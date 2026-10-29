# ResearchClip Step 2 验收报告

日期：2026-10-28。环境：Windows x64、PowerShell7、项目本地Rust MSVC工具链及锁文件、现有Node/npm与WebView2。应用保持4.4.0，SQLite保持schema21。范围是扩展/模型管理及所需OCR、更新联动；未继续开发IDE，未执行Step 3、Git推送、标签或Release发布。

## 1. 结论与完成范围

已在现有模块化单体内实现扩展与模型中心、可信目录验证、受管安装生命周期、中英文ONNX行识别及官方Updater接入，并完成真实Windows应用与真实模型的端到端验证。**Step 2不能标记为所有要求完全上线**：用户确认正式目录与公钥未发布；公式模型、语义视觉权重许可、外部软件真实安装及成功主程序升级仍有明确未完成项。

| 要求 | 本轮状态与证据 |
| --- | --- |
| 七类视图、搜索、组件详情与状态 | IMPLEMENTED；原生UI显示47项定义、7个中心标签，区分实现、安装、启用、宿主安装与错误 |
| 真实HTTPS下载、进度、取消、重试 | IMPLEMENTED；真实ONNX/字典和GitHub历史附件下载；正式在线目录UNPUBLISHED |
| 断点续传 | IMPLEMENTED；真实GitHub附件从81,723字节断点续传并验证完整SHA；模型站点中断用例连接失败，未冒充通过 |
| 可信目录签名、SHA、系统/版本/依赖检查 | IMPLEMENTED；真实Minisign签名、错误密钥、篡改、过期、序号回退、未来注册表与依赖环测试 |
| 安全解压、磁盘检查与专属目录 | IMPLEMENTED；受限平面ZIP白名单、逐成员SHA、超限/路径/原生代码拒绝；不执行脚本 |
| 安装、停用、启用、升级、卸载和重启恢复 | IMPLEMENTED；单元及原生端到端验证；旧版本保留、额外文件不删、提交失败可恢复 |
| 中英文OCR一键安装与真实推理 | PARTIAL整体OCR目标；已交付单行选区模型，真实中英文CPU测试通过；整页继续用原Windows OCR，不声称新模型具备布局检测 |
| OCR写入Markdown、检索索引 | IMPLEMENTED；复用现有正文及FTS，原生单行截图识别、SQLite持久化和Markdown插入已通过；PDF选区经既有图片工具使用，未单独原生回归此分支 |
| 离线OCR | IMPLEMENTED并验证失效进程代理下重启和实际识别；物理拔网线场景未执行 |
| 公式模型一键安装 | NOT_IMPLEMENTED；完整可信原生适配与权重许可未就绪，不能标READY；既有研究流程保留 |
| 原生语义模型 | PARTIAL原型；实际文字和图像CPU推理通过，视觉权重独立分发许可未核实，托管安装禁用 |
| Zotero插件下载/引导/连接检查 | PARTIAL；原有桥和向导复用，XPI实际构建，生产资产未发布；真实独立Zotero安装/右键/配对仍未验收 |
| 浏览器插件下载/引导/连接检查 | PARTIAL；真实历史ZIP已下载校验；目标浏览器安装与通信本轮未执行，绝不从下载成功推断宿主已安装 |
| Manifest、Catalog、兼容性与哈希准备 | IMPLEMENTED本地准备；两份真实候选组件生成测试签名目录，正式目录UNPUBLISHED，详见[目录契约](../release/PACKAGE_CATALOG.md) |
| 官方Updater | PARTIAL；官方SDK与签名策略、受控状态/下载/取消/确认安装已接入；正式源、公钥、签名安装器及真实成功升级未发布/未验证 |

## 2. 架构及数据保护

前端沿用React/Zustand及原FeatureRegistry、PackageManager定义，没有创建第二份业务资源或OCR数据库。新增中心API/store只持有Rust权威快照，原可选内置功能继续使用原设置服务。七个分类是UI视图，六种组件类型沿用Step 1定义。

Rust新增`services/packages`：DownloadManager跟踪单任务与取消，PackageManager串联签名目录和版本检查，PackageInstaller负责安全暂存、提交和卸载，PackageRegistry保存安装状态与签名证明，ModelManager只调用编译进应用的适配器。模型原生运行时随主程序资源交付，不执行下载的安装脚本。命令限主窗口、闭合操作枚举、拒绝未知字段，不开放任意URL/文件路径或通用进程执行。

安装先真实自检后提交；写入意向日志与注册表事务式替换，失败保留旧版。重试验证意向来源，不覆盖未知目录。停用偏好跨升级保留。卸载仅清理Manifest内文件，未知额外文件保留并报告失败；科研截图、Markdown、SQLite与OCR正文不属于卸载目标。

本轮没有SQL迁移或研究数据schema变更。包注册表schema1来自独立应用配置目录`packages-v1`；未知未来版本拒绝写入。默认资料目录在任务开始前只读记录32文件摘要。最后全目录严格比较为FAIL：新增1份schema20升级前快照和1份PDF，原有32文件均未改写/缺失；未删除、重置或覆盖。新增来源已询问用户，保留原始失败审计与差异记录，不能把全目录结果写成不变通过。测试全部使用artifacts隔离资料，不恢复或重置真实AppData，不写Zotero数据库，不改PDF原件。

## 3. 修复与可靠性处理

- 解决组件“已下载”与模型READY/宿主已安装混淆；外部插件始终需要目标软件安装，实际推理失败不能就绪。
- 防止安装任务期间的迟到取消、前端旧轮询覆盖新操作状态及重复任务；下载/安装阶段与错误可见。
- 修复注册表提交失败后的重试：保留签名安装意向，旧注册表仍有效，重试可恢复新目录提交。
- 阻止篡改本地Manifest、测试密钥安装被正式构建信任、异常目录和路径穿越；导出同样重新校验来源与文件。
- 卸载遇到额外文件时保留未知数据，记录UNINSTALL_FAILED，可重试；更新失败保留旧版及停用偏好。
- OCR提供方识别信息只在成功识别后提交到既有正文/FTS，失败不篡改旧识别来源；Windows OCR不可用时已安装ONNX行模型仍可用并显示系统错误。
- 纠正旧语义文档将代码MIT推定为权重许可的说明；视觉模型分发许可未核实，禁用新托管安装，保留实际原型测试结果与既有手动研究路径。
- 官方Updater失败显示FAILED，保留签名验证与512MiB上限；生产配置缺失时不联网、不显示虚假“已是最新版”。

## 4. 修改文件与依赖

以下为本轮相对开始时文件摘要的变更范围，不将已有v4.4.0未提交工作全部归为本轮修改。完整逐文件清单在`artifacts/step2-20261028/changed-files.json`。

| 范围 | 主要文件 |
| --- | --- |
| 模型/包核心 | `src-tauri/src/services/packages/{model,download,installer,models,ocr_model,semantic_model,mod,tests}.rs`及公开签名fixtures |
| 更新与IPC | `src-tauri/src/services/app_update.rs`、`commands/{packages,app_update,ocr,ocr_index,embeddings}.rs`、相关mod/lib/build/main capability |
| OCR/语义复用 | `services/{ocr,ocr_index,embeddings}.rs`、`storage/ocr.rs`、前端OCR/索引/语义面板与API类型测试 |
| 组件中心 | `src/features/feature-packages/PackageCenter.tsx`、`center-api.ts`、`center-store.ts`、样式及测试、原catalog/native/types/registry测试 |
| 主导航 | `src/app/{App,ui-store}.tsx/ts`、`layout/{Sidebar,Toolbar}.tsx` |
| 可信资源 | `src-tauri/resources/{package-trust,ocr-line-model,onnx-runtime}.json`、`native/onnxruntime.dll`及许可证/通知 |
| 构建与测试 | Cargo.toml/lock、tauri.conf、`examples/package_fixture.rs`、debug专用`qa_packages.rs`、`scripts/package-center-native-smoke.mjs`、目录准备脚本、原feature-smoke兼容更新 |
| 文档与指南 | 本报告、PACKAGE_CATALOG、候选JSON、USER_GUIDE第36章、原生截图、指南导航/测试、语义许可更正与依赖通知 |

新增直接Cargo依赖：`tauri-plugin-updater=2.14.1`、`minisign-verify=0.3.0`、`semver`（锁文件固定）、`zip=9.1.3`（仅deflate）、`ort=2.0.0-rc.12`（动态加载、API23、禁默认下载）、`tokenizers=0.22.2`（本地fancy-regex，无远程执行）。未新增npm依赖或改package-lock，没有整体升级依赖。API按安装源码/官方文档核对，替代方案：沿用Windows OCR；ONNX运行时随主程序交付；不采用下载Python/脚本作为一键模型原生依赖。

Microsoft ONNX Runtime1.23.2 DLL实际校验Microsoft Authenticode有效，大小/SHA固定；MIT及第三方通知随安装包交付。直接及新解析依赖的许可元数据与原文见[依赖声明](../licenses/step2-dependencies.json)和[通知集合](../licenses/step2-dependencies-NOTICES.txt)。非Windows平台只解析未构建的依赖不能视为平台验证；本轮未测macOS/Linux/GPU。

## 5. 本轮实际执行检查

命令均在当前Windows工程执行，Cargo通过`node scripts/with-rust.mjs`调用项目工具链，锁文件且离线构建；网络下载测试使用真实HTTPS。日志位于`artifacts/step2-20261028`。退出0才记录通过。

| 检查 | 实际结果 | 日志 |
| --- | --- | --- |
| `npm run check`（类型、ESLint、Vitest、前端build） | 退出0；100文件、790测试通过 | `check-delivery.log` |
| `npm run test:rust` | 退出0；345通过、0失败、12默认忽略；含迁移、持久化、包安装/回退与3万批注回归 | `rust-b.log`、`rust-delivery.log` |
| `npm run check:rust` | 退出0；all-targets、-D warnings | `clippy-b.log`；最后复核`clippy-last.log` |
| `npm run format:rust` | 实际格式化后检查，退出0 | `fmt-last.log` |
| 全新target目录`cargo check --locked -j4` | 退出0；从空目标目录编译新增依赖及主程序 | `clean-cargo.log` |
| 实际ONNX英文+中文CPU测试 | 退出0；1项通过，识别自有真实PNG，包含重开自检 | `real-ocr-final.log` |
| 实际语义CPU原型 | 退出0；1项文字/图像真实推理通过；不代表许可和托管交付 | `real-semantic-final.log` |
| 实际模型HTTPS下载 | 退出0；1项真实权重及字典大小/SHA通过 | `real-https-a.log` |
| 真实下载中断与续传演练 | 模型站点两次连接失败；同下载器真实GitHub附件退出0，1项通过，从81,723字节继续而非重下 | `real-resume.log`、`real-resume-b.log`、`real-resume-c.log` |
| Windows Tauri debug与fixture构建 | 退出0；最新业务源码构建 | `tauri-delivery.log`、`fixture-delivery.log` |
| 原生WebView2端到端 | `step2-c`退出0、9项真实下载流程通过；最新构建`step2-e`退出0、10项通过，明确使用SHA复核的真实上游文件缓存 | `native-c.log`、`native-cached-delivery.log`（`native-delivery.log`为模型站点连接失败，保留） |
| Catalog本地准备与测试签名 | 退出0；2份真实候选Manifest，UNPUBLISHED/development=true；最终元数据含实际NSIS大小/SHA | `catalog-prepare.log`、`catalog-delivery.log` |
| XPI确定性构建 | 退出0；10,461字节；SHA `e9a8fbd71daae4625e8b5cc5c1f4cbb0899f4f961a02ae910af52cdcaedd61e4` | `xpi-build.log` |
| 文档检查 | 首轮退出0；最终结果见docs-final.log；不是运行验收 | `docs-a.log`、`docs-final.log` |

原生网络用例9项覆盖：正式未发布与严格IPC、React确认后真实模型下载/自检、真实浏览器ZIP下载但不冒充宿主安装、480/760/1440宽度与CSS75%–200%布局、重启恢复、失效进程代理下真实OCR及SQLite/FTS/Markdown、未知目标目录失败保留、同权重1.0.1打包版本升级/停用偏好保留、卸载保留科研数据。此处没有用浏览器Mock替代Tauri Rust；CSS缩放不等同物理DPI。

本地NSIS候选包已真实构建：`artifacts/step2-delivery/ResearchClip_4.4.0_x64-step2-candidate-setup.exe`，23,400,212字节，SHA-256 `4edb35552f76f1f7da1b55c8f045d5e1cce6b1f460322f8b5b0aaf057c016fad`。`npm run desktop:bundle`退出0（nsis-candidate.log）。此包未发布、未通过正式Updater签名，本轮未执行安装器，不视为真实安装升级验收。

## 6. 失败记录与未执行项

保留所有失败日志。Clippy初次9条静态规则失败已修复并复查通过，没有关闭规则；自动fix的一次诊断UTF-8失败后核对源文件UTF-8有效，再运行正式Clippy通过。原生a/b用例因测试定位包含工作台额外tab及OCR按钮旧名称失败，按实际DOM修正范围/名称后c通过；没有降低功能断言。中断测试首次连接失败被误显示为缺少暂存文件，已增加“连接未建立”诊断，第二次明确记录模型端点连接失败。随后使用真实GitHub附件验证通用下载器，保持真实部分文件、续传起点及完整SHA断言，退出0；没有给失败的模型站点用例改写成功结果。最新构建原生d也因同模型端点连接失败终止，e使用显式--verified-cache模式复核安装/推理/回退，报告清楚区分缓存与新联网传输。

尚未通过：真实模型大文件中断/续传本轮仍受远端连接影响；真正Zotero安装、右键、配对与浏览器安装/剪藏回环；公式模型实际识别；语义视觉权重许可；正式Updater有效更新/错误签名安装器/更新后资料与组件恢复；物理断网、物理DPI、不同硬件及GPU矩阵。不能用SDK集成或Mock状态测试宣称这些验收完成。

## 7. Windows人工验收及前置条件

1. 在独立测试资料目录运行包含可信运行时的候选应用，打开模型中心，确认正式源未发布与未实现功能标识准确。
2. 配置由发布负责人批准的正式信任根和真实目录后，确认来源/许可证，安装OCR，核对进度/取消/损坏/错误签名/版本不兼容提示；在慢速网络演练模型大文件续传。
3. 用自备单行中文、英文、混合文本及PDF选区测试真实识别，人工校对，再保存正文、全文检索、插入Markdown并重开。整页用Windows OCR，不要求行模型识别复杂版面。
4. 物理断网重启，检查模型状态与识别；在100/125/150/200%系统DPI、窄窗口和双屏下查看卡片与确认对话框。
5. 在独立Zotero10.0.6资料配置主动安装已验证XPI，配对、右键打开/回跳、断连/重启；按Chrome/Edge机制安装真实剪藏包并主动保存网页，验证通信而不是仅查下载记录。
6. 正式Updater密钥/签名安装包/目录就绪后，在隔离安装环境验证成功升级、错误签名拒绝、旧模型兼容与截图/笔记/数据库保留。

下一步仅为上述外部条件与Step 2未验证项的补充验收；本轮不自动发布、不执行Step 3，不加入PDF导出或独立备份恢复系统。
