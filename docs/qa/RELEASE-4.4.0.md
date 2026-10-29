# ResearchClip v4.4.0 验收记录

**开发及下列自动检查已完成；正式发布受阻，尚未上传新Release。** 用户明确停止IDE开发，目标版本统一为v4.4.0。前序未发布Connector/布局与本轮30,000条批注一起准备；不发布IDE原型，也不自动实施后续ROADMAP阶段。

## 1. 目标与完成情况

- 当前PDF、一个已确认来源最多30,000条Zotero标准批注；保持实例、library ID/type、父文献、附件和原key身份校验。
- 20条分页，边界额外探测一条；30,000完整与30,001超限分别识别。缓存64MiB、累计传输128MiB、总缓存256MiB和取消/超时明确报错，不假装完整。
- 全量搜索/颜色筛选与页码/类型/修改时间排序；25/50/100卡片分页，末页可达。编辑/同步按需展开并分页，密集页面绘制全部支持区域。
- schema21只扩大ResearchClip缓存约束，升级前快照验证、事务迁移、原JSON保真和失败回滚；Zotero数据库与原PDF不改。
- 缓存与同步比较原子提交，按键比较、批量接受远端、CAS与历史保留。同步身份额度100,000与当前读取30,000分开，满额PDF仍可新增本地草稿。
- 合并专用Connector、认证本机协议、来源下载助手、现有布局预设/比例及原创多色图标；指南35章、277项功能示例、73幅离线图。

## 2. 修改文件与架构

[完整变更清单](CHANGED_FILES-4.4.0.md) · [规模设计与回退](../architecture/ZOTERO_ANNOTATION_SCALE_440.md) · [前序功能逐项核对](FEATURE_AUDIT-4.4.0.md)。

主要修改位于services/zotero_local、zotero_annotations、storage/zotero_sync、commands/zotero_annotations/zotero_sync及前端zotero-annotations目录；新增021迁移、分页组件、密集Canvas和大规模测试。复用既有PDF/Markdown/资源ID/版本/布局服务，没有第二存储服务或IDE框架。

本轮30,000条功能没有新运行时依赖。合并的Connector将已有锁定的httparse1.10.1间接依赖显式列为直接依赖，未升级该版本；安装包与便携包保留完整MIT许可。没有整体升级依赖。

## 3. 实际执行结果

环境：Windows11 10.0.26300，Node22.22.2，Cargo1.99.0/MSVC工具链；CARGO_NET_OFFLINE=true，Rust全量RUST_TEST_THREADS=4。Zotero安装版本10.0.6、Build20261007153950、Gecko140仅证明目标安装与SDK核查，不证明插件运行。

| 命令/检查 | 退出 | 实际结果与范围 |
| --- | --- | --- |
| npm run check（v440-front-check-final） | 0 | 类型、ESLint、96文件/762测试和生产构建通过 |
| npm run test:rust（v440-rust-full-final） | 0 | 327通过、8忽略；含30k/30001边界、额外草稿、迁移/回滚/重启和两客户端冲突 |
| npm run format:rust、npm run check:rust | 各0 | 最终格式、全目标Clippy通过，日志v440-rust-format-last/v440-clippy-last |
| 独立源副本npm ci --offline --no-audit + npm run check | 各0 | 327包干净安装；762测试/构建通过。副本早于末尾标签、状态提示和同步身份额度调整，不能称其逐字等同最终树 |
| Windows custom-protocol构建 | 0 | v440-native-build-last，真实exe内嵌前端；前端完成后顺序构建 |
| zotero-large-native-smoke v440-large-d | 0 | 20项；30,000读取、本地草稿使同步身份30,001、完整缓存与末条搜索/定位/重启均通过 |
| zotero-annotations-native-smoke v440-annotations-a | 0 | 28项；类型、颜色、坐标/旋转/跨页、错误与只读缓存 |
| zotero-writes-native-smoke v440-writes-b | 0 | 25项；受控写入/权限错误/复读/更新/删除确认，使用模拟API |
| zotero-sync-native-smoke v440-sync-b | 0 | 22项；冲突/删除/幂等/持久草稿/自动只读刷新，使用模拟API |
| zotero-connector-native-smoke v440-connector-a | 0 | 9项；真实回环监听、main-only IPC、拒绝浏览器/未认证请求、取消及布局，未配对真实XPI |
| pdf-quality-native-smoke / pdf-layout-native-smoke | 各0 | 各5项；无损像素/分辨率与混合页尺寸布局 |
| guide-reader-smoke v440-a | 0 | 14项；35章/277功能示例/73图、搜索/层级/图片解码/主题/字体/窄窗；是浏览器验证 |
| 六个发布工具测试脚本 | 各0 | 合计27个单元测试；不代表发布完成 |
| 实际Windows NSIS构建与installer-smoke v440-final-b | 各0 | 5组安装/卸载检查，精确payload、图标/版本/5种许可、忙时拒绝覆盖、PDF默认关联和无关文件保护通过 |
| default-data-audit verify v440 | 0 | 本轮基线32文件，变化0；只读哈希审计，排除WebView缓存，不是备份产品 |
| inherit-release-assets 4.4.0（显式回环代理重试c） | 0 | 47个既有发布/4,738个历史附件核对，328个本地继承附件；未上传 |
| publish --prepare负向门槛检查 | 预期拒绝 | 实际缺少真实Zotero回执时拒绝，未建立publish-v440或发起发布；保护测试本身通过 |

原生检查合计114项，公共场景有重复，不能当作114个独立覆盖点。涉及Zotero协议的运行使用虚构、隔离HTTP夹具：真实Tauri/Rust/WebView/SQLite/文件/PDF不是Mock，但**不是Zotero宿主验收**。最终大规模运行读取30,000条耗时13,391毫秒，仅为此夹具单次运行，不是冷启动或真实文献性能承诺。

可机器读取的摘要：[自动验收证据](evidence/v440-automated-tests.json)。完整命令日志与原生report保留在artifacts/dock-evaluation、artifacts/zotero-native-v440-*及相应PDF/guide目录。

## 4. 未通过记录与修复复验

- 原缓存持久化失败：真实SQLite表有8MiB CHECK；新增021而未改已发布迁移，最终30k缓存/升级/回滚/重启通过。
- 历史测试硬编码schema20、未来21及前端status20：更新为当前21/未来22，保留拒绝旧/未来与回滚断言；全量通过。
- v440-front-check-e中现存大图历史测试在并发编译负载时超过5秒：未删除断言或抬高超时，同一完整检查重跑通过。
- native-large-a：原生构建提前读取旧前端资源；改为前端完成后内嵌并重跑，默认折叠和50卡片通过。
- native-large-b：测试在后台同步状态恢复未完成时直接退出，被应用正确拒绝；补充可见状态/取消入口，验收等待完成后正常退出和重启通过，没有移除保护。
- native-writes-a：新增分页select使旧宽泛测试定位不唯一；新增明确ARIA标签并精准定位，原25项写入断言通过。
- 附件继承a/b：TLS EOF；保留错误日志，显式配置既有回环代理的c运行成功。没有覆盖旧发布。

以上失败日志保留，不能将最初失败称为通过；最终结果单独列明。

## 5. 最终包与发布状态

最终安装包：ResearchClip_4.4.0_x64-setup.exe，18187721 bytes，SHA-256 `171fae59792fff0d460db8124f94785d9910d0e9e7002cc3ffb120157fe5fddc`。在隔离中文/空格目录真实安装并卸载；最终报告installationStillPresent=false，原32文件哈希不变。前一轮安装包和报告另存，不冒充最终字节验收。当前包未签名，发布状态仍是未发布。

已通过47个历史发布元数据与前版字节核对，本地companions-v440共328项；当前版本总334附件是按现有流程的准备数量，**不是已上传核验数量**。最终SOURCE/SHA、精确源码冻结与匿名下载验收须待真实宿主门槛通过后执行。未来准备时使用最终安装回执参数 `--installer-run-id v440-final-b` 与target目录v413-distribution，不能复用旧payload哈希。

## 6. 尚未通过、已知限制与人工验收

用户明确要求“等桌面可操作并完成验收后再发布”。独立Zotero源代理实验未加载；本轮可以读取窗口结构，但Windows输入曾拒绝、捕获画面与目标窗口不一致，已停止不可靠输入。**独立真实XPI安装、右键菜单、配对、反向定位、Notifier、取消恢复和真实写入生命周期仍未验证，因此正式发布阻塞。** 没有更改签名/安全设置，没有直接写Zotero数据库，没有创建伪造的真实宿主回执。

人工清单：

1. 在独立Zotero10.0.6配置正常安装本次精确SHA的原创XPI，由Zotero完成兼容与安全检查。
2. 配对后右键文献/附件交给已有ResearchClip；多附件/多库完整身份、页码/批注目标、反向定位、变化通知和取消重连逐项核实。
3. 使用专用文献核对实际批注数量、末条、颜色/评论/页内坐标；不足30,000条时不得把夹具称为真实30,000条文献验收。
4. 经宿主用户授权后验证标准高亮/下划线/便签创建、复读、更新、条件冲突、删除和只读库错误；不以真实科研主库做破坏实验。
5. Windows100/125/150/200%物理DPI、多屏/HDR、真实窗口恢复和中文IME仍需专项验收；CSS缩放与浏览器/原生自动运行不替代它们。macOS/HarmonyOS未执行。

当前PDF页数上限、单批注文字/几何预算继续适用；超过总字节预算会明确失败，不能保证任意长的30,000条记录都能装入。Zotero ink、未知类型或不支持几何保留列表文字/身份和说明，尚不绘制其图形，不转换成ResearchClip私有笔迹。更全面绘制不能伪称已支持。

## 7. 下一步前置条件

需要可靠可操作的独立Zotero桌面，完成真实安装/授权与上述链路，生成匹配精确XPI和应用版本的真实回执，之后才能冻结源码、标签、上传及匿名附件校验。当前没有commit/tag/push/Release；既有v4.2.7未改。IDE已取消，不自动进入新业务阶段；不含Markdown→PDF或独立备份恢复系统。
