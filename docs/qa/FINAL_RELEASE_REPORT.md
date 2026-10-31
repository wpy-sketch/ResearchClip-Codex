# ResearchClip v5.1.0 正式发布验收

环境：Windows10.0.26300、Node22.22.2/npm10.9.7、Rust1.99.0、Tauri2.12.3；本机Python3.14.6（Anaconda，zlib1.3.2）。Windows CI使用windows-2025、同版固定Node/Rust/Python。

本轮目标：修复网页加载与地址输入、完善模型/扩展中心和设置、整合ResearchClip到ResearchClip-Codex、补齐正式发布资源。实际origin为[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)，三个版本配置均5.1.0。既有5.0.1公开版本、5.0.0草稿、5.0.2标签保持不动。**发布成功**。实际Tag为`v5.1.0`，指向已构建及验收的`0bd5429c37fad8a3d99f69f3da562f397cfb5d8b`；34个公开附件均经过匿名下载与SHA-256复核。

## 实现及架构

浏览器保留独立WS_CHILD WebView2与最小权限，地址用纯reducer保护草稿，原生事件按窗口世代及NavigationId处理失败，独立缓存与主工作台隔离；既有Preferences新增兼容默认值的首页与搜索设置，不改SQLite21。卡片复用原FeatureRegistry、PackageManager和本机探针，定位对应设置，不建立第二资源/文件服务。

四项固定可安装组件为OCR、语义模型、Zotero Connector、MIT网页剪藏包。语义模型固定8原始文件，经许可核查、真实CPU文本/图像自检后才READY；网页包保留已发布Host和第三方许可，不能由本中心执行或注入。原生绑定直接声明既有锁定webview2-com0.39.1（MIT），Tauri钉住2.12.3，无依赖全集升级。

仓库迁移实际结果见[完整回执](evidence/consolidation-20261030/complete.json)：49引用、47版本说明、543唯一原文件、4,738引用及约8.42GB内容已核验迁入目标；八组匿名下载逐项SHA通过。旧仓库已重新核验原始引用、47版说明及全部附件名称/大小/可用服务端SHA，确认没有新增问题或PR后归档只读，README增加迁移入口，原内容保留。退休提交`4018898aedbc120f13d79e7634f5160914101b31`另存目标分支`archive/researchclip-retirement-20261031`，见[归档回执](evidence/v510-retirement.json)。GitHub社交数据仍在原仓库，本轮未迁移。

## 本轮实际检查

- npm ci（主工程与扩展）退出0；npm run check退出0，107文件826测试，TypeScript/ESLint/前端构建通过。
- Rust当前批次370通过、12显式忽略；Cargo fmt/Clippy退出0，含迁移和兼容性测试。原生生命周期、许可拦截及窗口层级修正已纳入本轮重跑；清单EOL固定后另跑包兼容检查。
- 真实语义CPU文本/图像自检1项退出0；原生整链7组通过：实际HTTPS下载安装、CPU自检、真实图片索引、离线中英文查询、启停、受管卸载；不声称模型准确率。
- Bing原生5组通过：地址Tab/轮询保留、首页与搜索实际显示、真实网络失败、恢复、设置保存生效；当前版本复测通过；自然缓存路径的优化QA构建3项自动检查通过，用户直接确认必应网页与搜索框可见。
- 指南原生渲染器15组通过：38章/287例/79图、明暗、480–1440宽度、75–200%CSS缩放和独立Markdown撤销；不等同物理DPI。
- 发布/迁移契约19项退出0，属于自动化契约，实际公开迁移另有回执。
- 当前PDF摘录、Markdown内容/保存/重启、Zotero10.0.6只读连接、实际OCR下载/离线推理、真实Host与剪藏服务整链全部通过，分项结果见[事实摘要](evidence/v510-native-summary.json)。
- 主工程与扩展干净npm安装均退出0；默认资料34份文件SHA比较未变。
- 最终生产标识NSIS构建退出0，32份内容文件及2份元数据已收集；实际安装/卸载6组退出0，34份既有资料哈希不变。六类批注/三万条隔离原生用例8组通过（模拟API，不是实际三万条文献）。
- 精确构建提交`0bd5429c37fad8a3d99f69f3da562f397cfb5d8b`；安装器SHA-256 `953b3397acca7a424224143f95fe5252b6add96ae698b5dd91e57f63b444c1c7`。发布前回执见[本机范围回执](evidence/v510-release-ready.json)，公开34项完整资产、大小、SHA及真实下载URL见[发布回执](evidence/v510-publication.json)。页面HTTP200、最新稳定版本、Tag及旧仓库状态见[网页核验](evidence/v510-web-check.json)。

首轮测试失败与修复保留：旧组件数量断言、指南图片白名单漏登记、Clippy嵌套条件、冷启动页面等待、Bing绘制时序、语义安装旧许可拦截。没有移除安全断言或伪造运行结果。旧剪藏用例原先假定切换文档就丢弃旧草稿，已改为验证多标签独立保留及关闭取消/明确放弃；失败残留测试实例先保存后正常退出。原生窗口截图不可靠（设置页也返回黑屏），视觉只计用户明确确认；单一CDP连接不能发现独立WebView环境，不再将该发现机制误当应用失败。最后附加回归曾误用依赖开发服务器的普通cargo build导致主页面超时，改用Tauri内嵌资源构建后同一用例8组通过；原失败报告保留。

## 尚未实现及人工范围

原生公式一键适配与Qwen托管一键安装未交付，不进入固定可安装目录；现有Python公式配置/受控下载导入、Qwen离线导入保留。未默认开启WebDAV/剪藏/Zotero授权，不读写真实Zotero SQLite。没有新增IDE、PDF导出或独立备份恢复。

用户此前明确接受本机隔离范围；独立干净Windows、生产首次启动/真实旧版安装升级、物理混合DPI/HDR、多屏、当前原生Markdown文件对话框、当前Zotero写入CRUD、真实群组库及云同步继续人工验收。当前只读连接和未改模块的历史实际结果与本轮结果分别记录。

自动更新依用户要求停用，只允许精确编译清单+HTTPS+大小/SHA，手动NSIS升级，无Catalog/Updater密钥或虚假更新元数据。安装器代码签名与Updater签名分开，当前安装器未代码签名。出现核心P0/P1或资产核验失败则不发布稳定版。

## 公开版本与下载

- 维护仓库：https://github.com/wpy-sketch/ResearchClip-Codex
- 正式Release：https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/v5.1.0
- Windows x64 NSIS：https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.0/ResearchClip_5.1.0_x64-setup.exe
- Zotero Connector1.0.1：https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.0/ResearchClip-Zotero-Connector-v1.0.1.xpi
- 网页剪藏1.0.3：https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.1.0/ResearchClip-WebClipper-v1.0.3-windows-x64.zip
- AI资源：OCR1.0.1的2文件、图文语义1.0.0的8文件、两项Manifest及Apache/MIT许可。各原始权重也上传Release；客户端按固定上游HTTPS地址与编译哈希下载。
- 另附离线指南、安装/扩展说明、Catalog、兼容矩阵、Zotero更新清单、历史映射及SHA256SUMS。全部34文件见发布回执。

公开后当前原生客户端复测：Zotero XPI真实HTTPS下载2组通过；模型与网页包7组通过，包括8文件真实下载、CPU图像/文本自检、SQLite图片索引、重启及故障代理条件下离线查询、启停、从v5.1.0实际下载网页包和受管卸载。插件仍显示待宿主安装；没有把下载等同于Chrome/Zotero安装或连接。没有代为操作安全授权。

## 发布过程与构建工作流

首轮上传在大权重阶段中断，保留27份草稿资产；恢复时逐项核对、不覆盖既有文件，补缺并使用受限GitHub侧传输完成固定权重（[成功运行](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41187263608)）。传输仅下载固定模型、校验后补缺，不发布、不删除、不替换。完整草稿回读通过后公开，再对34项匿名下载完整SHA复核，最后真实客户端测试通过才归档旧仓库。

原Tag CI的测试通过，构建因XPI容器字节不同被严格校验拦截（[原运行](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41164223818)）。发布采用本机已完整验收的原安装器及同一10503字节插件。隔离分支修复只处理打包重现：固定原包完整SHA与五份源码字节均一致才允许复用；源码差异或损坏拒绝。23项本地发布/重现/传输测试通过；[修复后的完整Windows CI](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41186713225)已成功：107文件826前端测试、370 Rust通过/12忽略、Clippy、前端构建及NSIS构建通过，publish任务按设计跳过。发布工程修复已正常快进合并main，应用源码指纹仍与正式构建一致。云端再次构建的安装器仅作为CI产物保存，未替换本次本机已验收的安装器；见[CI证据](evidence/v510-ci.json)。此后新CI产物不替换已公开安装包，不移动v5.1.0标签。

## 修改文件与后续前置条件

应用改动清单见[修改文件](CHANGED_FILES-5.1.0.md)；本轮仍沿用单体架构和既有资源/文件服务，SQLite21无新迁移。发布工程补充了严格XPI重现及受限模型传输测试和工作流，不引入应用依赖或原生权限。

后续功能开发需另行指定阶段。原生公式模型一键安装/Qwen托管安装需完成权重许可、适配和真实推理验证；独立干净Windows、硬件矩阵、当前文件对话框与Zotero写入复测等保留人工范围见上文。已完成的本轮功能不依赖这些可选后续模块。

最终合并后检查：30项发布/重现/传输/迁移契约退出0，文档248份/1301本地链接检查退出0，Git diff检查退出0。v5.1.0标签仍指向原已验收构建；发布脚本跨压缩实现修复在main，未反向改写该不可变标签。
