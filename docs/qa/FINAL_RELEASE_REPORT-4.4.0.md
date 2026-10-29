# ResearchClip 4.4.0 最终发布验收报告

记录日期：2026-10-28。状态：**BLOCKED / NOT_PUBLISHED**。开发和隔离测试继续完成，但不满足公开稳定版门槛；本报告不构成发布成功声明。

## 1. 仓库、权限与版本

| 项目 | 实测结果 |
| --- | --- |
| origin及发布目标 | https://github.com/wpy-sketch/ResearchClip-Codex.git |
| GitHub身份 | wpy-sketch；Git凭据助手提供给当前CLI子进程，不输出或落盘凭据 |
| 仓库权限 | GitHub API返回admin/maintain/push为true |
| 当前分支与HEAD | main；ec867a9d8004625380e3ac48cee0328f8b19f306 |
| 远端main | 53e4bfa994d4c648b5b4d0545cb1b87a164e033f，已确认是当前HEAD祖先 |
| 工作区 | 开始时已有大量Step1/2及旧阶段待提交修改，完整保留；本轮未重置或覆盖用户工作 |
| 应用版本 | package.json、Cargo.toml、tauri.conf.json均4.4.0 |
| SQLite | 21；本轮没有增加迁移，新位置字段兼容旧格式缓存 |
| 既有Tag/Release | 本地最高v4.2.7；目标仓库未发现v4.4.0 Tag及Release |
| 本轮实际Tag | 未创建 |
| Release页面URL | 不存在，未创建Draft或公开Release |
| Windows安装包下载URL | 不存在，未上传 |
| 已上传扩展/模型/XPI | 无 |
| 自动更新 | NOT_READY；正式公钥及地址未配置，没有伪造latest.json |

较早用户消息中的另一个ResearchClip仓库不作为本次发布目标。旧附件搬运脚本包含历史仓库地址，本轮未执行它们。

## 2. 已实现范围与边界

六类Zotero标准批注读取、ink/text原生显示、类型/文字/颜色筛选、最多30000条和50条分页、右导航调宽和完整文件名、点击PDF区域定位列表、划词工具栏和受控快捷键已实现。读取采用100条批次、2MiB响应预算自动缩批、同版本条件刷新、页面索引和分帧Canvas。

非ink五类有严格输入校验、本地API写入、预期版本、复读及持久化待处理状态；授权来源完整LOCAL_DIRTY才自动提交。真实新增类型CRUD、群组权限、宿主重连和云同步仍未完成本轮独立Zotero验收。云端数据及附件同步由Zotero自己设置管理，ResearchClip不会从本地写入推断云端上传完成。

Zotero ink保持标准批注身份及只读路径投影；用户已明确确认“目前的原生显示与标准同步即可”，不开发独立私有可编辑副本。Google最右侧提供真正Chrome的外部入口，用户已于2026-10-28明确选择此方案，账号与书签同步由Chrome管理。内嵌Chrome不再作为本轮缺失功能或发布阻塞项；实际启动和同步配置需在可操作桌面验收。XPI按Zotero机制安装，不能直接加载为Tauri插件。

架构、坐标、权限、缓存和回退见[本轮架构记录](../architecture/ZOTERO_NATIVE_ANNOTATIONS_RELEASE.md)。完整历史功能盘点仍见[Step1功能矩阵](../release/FEATURE_MATRIX.md)，不将历史报告改写成本轮实测。

## 3. 修复的问题

1. 首次来源确认前缓存预热为空，确认后批注来源仍停留为空：确认成功后刷新当前会话来源、缓存和同步状态，并增加回归测试。
2. 最大化PDF后切换其他面板仍隐藏目标：聚焦不同tabset时恢复最大化状态，保留原草稿及EditorSession。
3. 新Google IPC空结构变体曾忽略额外字段：改为空字段结构体变体，拒绝URL、程序、profile覆盖。额外URL从未用于导航；最终原生回归已验证严格拒绝。
4. 发布脚本导入带连字符模块失败：修复显式模块加载，并添加缺少正式信任根时不生成资产的自动测试。
5. 更新元数据、Manifest、Catalog和安装器可能脱节：发布前核对实际URL、版本、SHA、长度及允许组件集合，先验证原生签名再创建Draft，公开后逐文件匿名下载复核。

## 4. 实际检查与证据

环境：Windows11 10.0.26300，Node22.22.2，npm10.9.7，Rust1.99.0，Python3.14.6。命令、退出码及最终测试数量记录在[本轮检查证据](evidence/final-release-checks.json)。日志保留于本机artifacts/release-20261028-r1；测试报告不包含凭据、真实笔记或截图内容。

已执行TypeScript、ESLint、Vitest、Rust测试、fmt、Clippy、前端构建、Tauri开发版构建、NSIS候选构建、数据库迁移/文件一致性测试、真实中英文CPU OCR、隔离组件安装/离线重启、原生PDF渲染/裁切/插入、版本绑定签名和发布拒绝路径测试。每个项目以证据文件最终结果为准；有未通过或未执行项目时不标全部通过。

保留本轮失败记录：早期来源初始化及最大化导航问题；测试使用不存在的“末页”按钮后改用实际页码输入；新IPC未知字段用例复现后修复；Cargo测试执行期间并发链接遇LNK1104，待测试进程结束后串行重跑。一次OCR命令用了不完整的--exact过滤器，实际运行0条；已用正确过滤器执行真实推理，0条运行不计通过。

浏览器/HTTP模拟只验证ResearchClip应用和协议，不是Zotero实际安装或云同步证明。CSS/DPR模拟不能替代物理高DPI多屏。原生摘录render-only覆盖保存摘录及内存Markdown交互，明确不覆盖系统文件选择器保存与重开。

## 5. 安装包与资源

Windows x64 NSIS候选文件实际存在，最终名称为ResearchClip_4.4.0_x64-setup.exe。最终候选SHA、长度和签名状态记录在检查证据中。候选没有正式Updater签名，不是可发布的正式更新资产。Windows Authenticode为NotSigned。

最终候选大小23,414,021字节，SHA-256：`fa4c354c262b6f8ae23c62f95cbb98863d691cc12f688f4808ac19abb16c739e`。本机候选与SHA256SUMS保留在artifacts/release-20261028-r1/candidate，未上传。实际NSIS安装/卸载6项检查通过：中文路径、版本/图标/标识、运行中拒绝覆盖、PDF关联与默认阅读器保留、CPU运行时精确SHA及Microsoft签名、许可文件、卸载保留无关文件。没有将正式主程序启动到用户真实资料目录，此项不代替独立Windows安装后完整业务验收。

本轮默认资料目录34个文件只读哈希核对通过，变化0。Windows Authenticode和Tauri Updater签名分别核查，不能相互代替。

OCR真实权重、字典、可信内置ONNX Runtime及Apache-2.0许可已准备；XPI1.0.0可确定性构建。公式模型未验证完整许可和原生链，视觉权重分发许可未核实，浏览器历史归档主代码许可未声明，均不进入正式可安装目录。测试目录不作为在线正式源。

## 6. 明确阻塞与未执行验收

- src-tauri/resources/package-trust.json四个正式字段仍为null；进程未提供正式签名私钥，目标仓库级Secrets列表为空。进一步核查仓库不存在配置的环境，环境级签名配置同样尚未建立。未擅自建立新生产信任根。
- 已装Zotero10.0.6并启动独立配置，读取UI成功，但点击返回GetCursorPos failed / 拒绝访问0x80070005；未绕过安装、签名或授权。真实XPI安装、右键菜单、配对、五类CRUD、权限和云同步待操作桌面恢复。
- 独立Windows机器使用实际公开资产完成安装/启动/保存重开/升级保留资料/生产模型下载/成功自动更新仍未执行；当前没有公开资产可供此项验收。
- 用户已确认不要求工作台内嵌Chrome，采用真正Chrome及其账号/书签同步；保留实际启动验收。用户同时确认只需原生显示与标准同步，独立私有可编辑笔迹副本不在本轮范围。
- 物理高DPI多显示器、Windows系统文件选择器及实际Chrome登录/同步需人工验收。

用户最新确认：签名暂不配置，先完成开发与其他验收；桌面恢复后再做真实Zotero验收。不会反复请求签名配置，也不会在用户未报告桌面恢复时重试宿主安装/授权。待验收顺序见[桌面验收清单](DESKTOP_ACCEPTANCE_CHECKLIST.md)。

上述条件使正式发布不可执行。本轮没有强制推送、覆盖Tag、删除Release、上传测试私钥、将测试模型标为正式或关闭签名校验。

## 7. 发布恢复路径

1. 配置正式公钥及经安全环境/CI Secrets提供的对应私钥；受保护环境release-signing和production由仓库维护者管理，不提供或提交私钥到聊天或Git。
2. 检查并提交当前源码和已有待提交工作，执行工作流build；使用实际构建出的确切安装器和XPI在独立Windows/Zotero完成所有强制验收。
3. 提交真实验收回执，必须匹配应用源摘要、安装器和XPI哈希；测试和模拟回执不能替代。
4. 发布脚本核实origin/权限/提交/Tag，建立新Tag并创建Draft、无覆盖上传、下载复核、原生验签；门槛全通过才公开，再逐项匿名复核。
5. 公开验证通过后再填写本报告的真实Release/安装器/组件URL。下一步是解除发布阻塞，不自动开发其他功能。

## 8. 修改文件和依赖

本轮文件列表见[修改清单](FINAL_RELEASE_CHANGED_FILES.md)，与任务开始时源文件哈希比较生成，区分既有待提交文件与本轮实际变化。仅新增dev-dependency base64 0.22.1，复用已锁定的同一版本及MIT/Apache-2.0许可；没有批量升级依赖。生产OCR运行时属于Step2既有工作，保留其真实许可和哈希。


## 9. 用户确认后的补充验收（2026-10-28）

用户已确认采用真正Chrome及其自身账号/书签同步；六类批注采用原生显示和标准同步即可，不要求独立私有可编辑副本。两项均不再作为功能缺失或发布阻塞。正式签名暂不配置，先完成开发和其余验收；Zotero真实宿主验收等待用户告知桌面恢复。

补充修改仅包含原生HTTP协议测试和文档，不改变生产业务、数据库、依赖或已构建候选安装包。新增3个回归：

1. 五类标准批注完整prepare→POST→复读→PATCH→复读→条件DELETE流程，检查自由文本字号/旋转、区域与便签不写annotationText，修改时不重写位置；过期删除返回冲突，保留新版本。
2. HTTP200但复读内容不一致时保持未核实，后续复核使用同一key，不重复创建。
3. 自由文本和区域写入403拒绝时不误报保存成功，不生成假远端记录。

命令`cargo test ... zotero_local::write_transport::tests -- --nocapture`实际通过9项，真实宿主测试1项保持忽略。完整`npm run test:rust`实际通过353项，12项忽略，退出码0；`npm run format:rust`、`npm run check:rust`均退出0。首次新增测试因测试服务未剥离URL查询参数而失败，修复测试路由后重跑通过，没有修改产品参数校验来迁就测试。

[补充检查证据](evidence/release-followup-checks.json)将这些结果与上一轮350项Rust测试、804项前端测试、NSIS和原生UI结果分开记录。没有重新运行Zotero宿主安装或授权，没有配置生产密钥、创建Tag/Draft或发布资产。此前安装候选及SHA保持不变，不能将新的协议测试说成真实Zotero验收通过。

本次修改文件：`src-tauri/src/services/zotero_write_tests.rs`、本报告、`docs/qa/DESKTOP_ACCEPTANCE_CHECKLIST.md`、相关架构/发布说明/README、两份检查证据与修改清单；无新依赖，无数据库迁移。下一步依赖可操作桌面及后续正式签名配置。
