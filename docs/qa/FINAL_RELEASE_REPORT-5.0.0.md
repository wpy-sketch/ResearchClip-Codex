> 5.0.0历史快照；草稿ID427552252，草稿链接可能随编辑变化。不是5.0.1验收回执。

# ResearchClip5.0.0源码与Release草稿交付

本轮目标为5.0.0；之前4.4.0候选报告保留在[历史报告](FINAL_RELEASE_REPORT-4.4.0.md)，不把旧结果视为本轮通过。本轮源码与14个草稿资产已实际上传并经认证下载逐一校验，**没有公开发布稳定版**。

实际origin：[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。已验证GitHub CLI身份具有push/admin权限，正常推送main及新Tag，未强制推送、移动Tag、覆盖资产或删除Release。源码提交和Tag均为`1c1bb6b68b0b413d2b7bad63ef4caef06288206a`；后续验收文档另行提交，不改变安装包源码身份。

当前版本配置已改为5.0.0；SQLite保持21，无新增迁移。OCR组件包装版本1.0.1，权重与字典仍为已验证固定文件；Connector1.0.1。根据用户2026-10-29最新要求，取消Catalog/Updater公私钥配置，使用编译内置精确组件清单与SHA-256，停用自动更新，改为手动安装升级。旧签名测试只保留为兼容性回归，不作为本版发布前置条件。

用户最新确认：没有可用独立Windows，本轮先上传源码和Release草稿，保留干净Windows验收；不公开发布稳定版。原5.0.0未发布，因此本轮继续使用实际配置5.0.0，不覆盖已有正式Tag或Release。

当前实际结果：前端104文件/813项通过；全量Rust362项通过、12项忽略，fmt/Clippy通过。12项需真实模型、WebDAV或显式宿主配置，不计作通过。免密钥原生组件闭环已完成真实上游OCR下载和实际CPU推理；本轮再次以明确SHA验证缓存演练安装、重启代理断网、插入Markdown、启停与卸载，5组检查通过。保留4.4旧组件文件，不兼容时停用提示，不静默删除。

真实独立Zotero10.0.6已安装启用Connector1.0.1；用户完成配对与写入授权。右键打开、来源核对、便签/ink读取与变化通知、反向LOCATED、重启重新配对已验证。五类非ink创建、改字改色和删除共15项通过API独立复读，删除后GET404。修复trim/NFC及有界浮点往返误报后，原待处理高亮按原键复核SYNCED，没有重复创建。真实群组权限、云账号与双机器同步仍未验收，不把本机成功当作云端成功。

PDF216DPI选区截图插入新Markdown，通过真实Windows保存和打开对话框，在中文/空格路径保存、关闭及重开，公式/表格/未知语法逐字保真、相对图片解码和PDF来源高亮、编辑会话保留通过。旋转/坐标/裁剪、六类批注30,000条及有界DOM另有原生HTTP夹具回归；本轮再跑30,000条8组检查通过，不能当作真实30,000条Zotero资料库证明。

本轮新增Windows工作台内嵌网页：独立WS_CHILD WebView2、地址栏/搜索/导航/停止/缩放；保持主窗口原生IPC与Markdown草稿，远程网页资料库IPC被拒绝。example.com/example.org、前进后退和地址同步、75%–200% CSS缩放、150%网页缩放、隐藏恢复和正常退出7组原生检查通过。当前网络未确认Google首页加载成功。当前功能48项：28 IMPLEMENTED、20 PARTIAL，完整范围与修改见[浏览器及功能审查](BROWSER_WORKSPACE_500.md)。未实现的可选模型、未来模块和其他平台没有伪装完成。

## 修改与技术设计

新增浏览器前后端、main视图精确能力、导航和几何/失败测试、Windows原生冒烟；复用现有工作台、文件/资源/历史/权限，不新增数据库或全局状态。图标为原创四色选区文档，指南38章、286示例、76幅离线图，章节和本地图片检查另行记录。新增依赖：无。此前未提交的Step1/2、Connector及批注规模修改一并保留整合，详见[修改文件](FINAL_RELEASE_CHANGED_FILES.md)。

实际日志保留在本地artifacts，包含失败记录；最早多视图实验、过时测试选择器和布局瞬时越界不计作通过。最终修复已改用稳定parent_raw，边界裁剪与紧凑布局避免覆盖相邻面板。不移动或删除用户研究文件。

## 当前交付边界

最终NSIS、SHA-256、确定性XPI、真实OCR权重/字典、Manifest和固定清单已上传Draft，14个文件全部经GitHub CLI认证下载并与本地大小/SHA-256一致。草稿需要仓库授权；公开Connector地址在草稿状态可能404，必须显示未发布/失败，不能误报安装成功。自动更新停用，不生成latest.json/.sig，不要求密钥配置。

干净Windows公开资产安装/启动、真实旧版升级、物理混合DPI/HDR/多屏、部分外部账号与浏览器宿主仍需人工验收。当前机器的隔离安装不能代替独立机器。用户已要求保留草稿，故不降低正式稳定发布门槛。

## 实际GitHub交付

- 版本及实际Tag：`5.0.0` / `v5.0.0`，指向已构建提交`1c1bb6b`。
- [源码提交](https://github.com/wpy-sketch/ResearchClip-Codex/commit/1c1bb6b68b0b413d2b7bad63ef4caef06288206a)。
- [Release草稿真实页面](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/untagged-646300e8bd9199b59035)，GitHub release ID `427552252`，draft=true、prerelease=true，不标记latest稳定版。
- [Windows x64 NSIS资产地址](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/untagged-646300e8bd9199b59035/ResearchClip_5.0.0_x64-setup.exe)：22.39 MiB，SHA-256 `4bc8cbaf2f486f219dfd0a1a0edf3e3d4a61a652fac9f278f6763a6a827f8fbc`。
- [Connector1.0.1资产地址](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/untagged-646300e8bd9199b59035/ResearchClip-Zotero-Connector-v1.0.1.xpi)，SHA-256 `1f98896eeef54ead00c5405d2460578d3269d82a5a011038779c9a9e2973fdc2`。
- 已上传模型：PP-OCRv4 `rec.onnx`和PaddleOCR `characters.txt`，包装版本1.0.1，Apache-2.0。已上传插件：ResearchClip Zotero Connector1.0.1。其余为固定Catalog、2个Manifest、兼容性、来源/许可、交付策略、Zotero更新索引、SHA256SUMS及构建身份，共14个文件；不包含未验证的公式/语义模型和浏览器安装归档。

上述资产地址来自实际GitHub API。Draft下载需要权限；本轮实际回下载走认证端点，**未验证匿名公开下载**。按Tag读取草稿的REST接口返回失败，改用已创建Release ID和CLI核实，未重新创建或覆盖资产。正式稳定发布脚本的草稿读取/续跑仍需在后续正式发布验收中核对。

## 最终安装与数据检查

Windows内核版本10.0.26300.0、Node22.22.2、npm10.9.7、Tauri2.12.3、项目Rust1.99工具链。本轮完整检查命令为`npm run check`、`npm run test:rust`、`npm run format:rust`、`npm run check:rust`、`npm run check:docs`及真实Tauri debug/NSIS构建；记录均退出0。文档检查240份/1276本地链接通过，不称为运行验收。

最终NSIS在中文/空格隔离路径执行6组安装器检查通过：完整载荷、PE版本与图标、Microsoft签名及固定哈希ONNX运行时、运行中拒绝安装/卸载、PDF OpenWith注册且保留原默认关联、卸载保留额外文件及用户资料。没有以生产EXE启动真实AppData；运行中保护使用明确的受控进程夹具，不能当作生产启动验收。另从本轮开始时只读基线核对34份既有资料，changed=0。应用安装器Authenticode为NotSigned，与已停用的Updater签名不是一回事。

## 未完成与下一步前置条件

本轮完成用户授权的源码及草稿交付后停止。保留独立干净Windows从资产安装/启动、真实旧版升级、正式客户端Connector下载、物理断网及硬件矩阵、群组库/云同步、Google网络可达性和未完整验证的可选模型。取得独立环境并完成相应验收后，才评估公开稳定发布；不自动转正，不要求签名密钥配置。
