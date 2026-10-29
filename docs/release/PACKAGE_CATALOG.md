# v5.0.2 组件清单与手动发布契约

更新日期：2026-10-29。用户明确取消正式Catalog及Tauri Updater的公私钥配置。当前采用`MANUAL_BUILTIN_CATALOG`交付；无需签名Secret、不生成Updater签名或latest.json，不把未签名在线目录当作可信来源。旧签名实现只保留为历史兼容与隔离回归，不是本版安装要求。

## 1. 可信来源

唯一可安装定义来自编译进Rust主程序的[package-builtin.json](../../src-tauri/resources/package-builtin.json)。每项包含固定ID、版本、发布者、许可证及URL、HTTPS文件URL、字节数、SHA-256、应用版本和原生适配器。原生IPC只接收闭合组件ID和确认操作，不能接收任意URL、密钥、DLL或脚本。

本地`catalog.json`不能覆盖编译清单。安装证明沿用现有注册表字段；`BUILTIN_MANIFEST_V1`是模式标记，不是数字签名。启用或加载时要求证明与主程序内置清单逐字一致，包字段完整相同，再核对每个真实文件。仅伪造这个标记、修改URL、版本、哈希或本地清单不能通过。

组件清单更新必须通过新版主程序交付。不会下载任意可变远端目录后跳过签名验证。`published=false`表示没有配置在线签名Catalog，`catalogMode=BUILTIN`表示固定组件可按内置来源下载；这两个状态不会混为线上已发布声明。资源尚未公开或网络故障时报告实际下载错误。

## 2. 真实组件

| ID / 版本 | 实际资源 | 交付及限制 |
| --- | --- | --- |
| model.ocr-line 1.0.1 | PP-OCRv4 ONNX和PaddleOCR字典，Apache-2.0 | CPU中英文单行选区，加载和真实推理成功才READY；整页仍用Windows OCR |
| plugin.zotero-connector 1.0.1 | 本仓库原创MIT XPI；Zotero10.0.6 | 实际独立宿主已安装、启用、配对与右键打开；下载不代表宿主安装 |
| 公式/视觉语义模型 | 完整链或独立权重分发许可未核实 | 不在正式可安装清单 |
| 浏览器扩展历史归档 | 既有源码和安装引导保留，主代码许可证未独立明确 | 不作为本版免密钥可安装包 |
| 模板测试资源 | 仅隔离回归使用 | 不当作正式线上组件 |

| 文件 | 大小（字节） | SHA-256 |
| --- | ---: | --- |
| rec.onnx | 10857958 | 48fc40f24f6d2a207a2b1091d3437eb3cc3eb6b676dc3ef9c37384005483683b |
| characters.txt | 26249 | 28b2362ad4ab2dc38769aa72feb535e3a9ddb3fd2a7585a05920e6393b1dc7f7 |
| onnxruntime.dll | 14186016 | dec964ab1ee36cc9b0ae247d13b376627992fc57dec0454354017ab8fd84f1ea |
| Connector1.0.1.xpi | 10503 | 1f98896eeef54ead00c5405d2460578d3269d82a5a011038779c9a9e2973fdc2 |

OCR来源为[SWHL/RapidOCR固定提交](https://huggingface.co/SWHL/RapidOCR/tree/1cfba2e90fc938db55889873735088de210cc173)，字典固定PaddleOCRv2.7.3。原生ONNX Runtime CPU1.23.2随可信主程序交付，按大小/摘要加载；本机实际核对Microsoft Authenticode及MIT/第三方通知。组件中心不下载或注入原生运行时。没有新增依赖。

## 3. 安装、恢复和卸载

沿用现有DownloadManager、PackageInstaller、PackageRegistry及ModelManager：HTTPS白名单、逐跳重定向检查、字节预算、SHA、磁盘空间、取消/重试、有效强ETag续传、安全ZIP成员白名单、禁止未知文件覆盖。CPU推理自检失败不能READY。

文件在应用设置目录`packages-v1`，与截图/Markdown/数据库分离。版本化注册表及安装意向记录保留原子提交和故障恢复；升级失败保留旧版本。未知未来格式拒绝写入。卸载只删除精确受管文件，遇到额外用户文件停止，不删除科研资料。

5.0.0/5.0.1内置清单已安装的相同文件，可由精确旧清单证明迁移到5.0.2定义；仅允许应用版本与发布URL变化，文件大小、哈希及其他字段必须相同。迁移只更新受管注册表元数据，重新核对文件并通过真实CPU自检后恢复就绪；不会改写模型文件。旧签名模式组件不会自动被本版内置清单信任；不兼容或不受信任的文件保留并停用。受支持资源重新安装新版本时复用既有受管安装流程；未验证来源不会加载。原始截图、Markdown及SQLite21不因清单模式改变而迁移或重置。

## 4. 主程序升级

本版自动更新状态为`MANUAL_UPDATE_REQUIRED`。Tauri自动下载/安装入口不可用，生产配置没有Catalog/Updater公钥、私钥或URL要求。用户从实际origin仓库的Release手动获取NSIS和SHA256SUMS，保存关闭编辑会话后安装。Windows Authenticode与文件哈希不是相同概念；当前候选安装器未代码签名。

## 5. Release资产

`build_manual_release.py`构建NSIS并收集真实权重、字典、XPI、许可、Catalog、逐项Manifest、兼容表、delivery.json、zotero-updates.json及SHA256SUMS。公开catalog.json仅供检查，与编译清单字节相同，客户端不会用它覆盖编译定义。Zotero更新索引精确绑定相同Release的XPI版本、URL及摘要。

`publish_verified_release.py`仍核对实际origin、版本/提交、源摘要、核心原生验收回执、P0/P1、资产全集/哈希、Tag冲突、Draft回读及公开下载。手动模式只移除签名要求，未移除数据保护、Zotero和安装验收。禁止覆盖Tag、clobber已有资产或删除旧Release。

尚未公开的候选URL不是正式可用下载证明。最终真实Release、资源URL及未通过项见[最终报告](../qa/FINAL_RELEASE_REPORT.md)。
