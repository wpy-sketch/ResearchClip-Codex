# 扩展、OCR模型与Zotero Connector

当前使用随主程序审核并内置的固定组件清单，无需配置公钥或私钥。组件中心显示定义、安装状态和真实错误，定义存在不代表已安装或可用。测试目录仅用于隔离验收，不是正式下载入口。

## 1. 已发布与未发布资源

| 组件 | 来源与许可 | 当前交付状态 |
| --- | --- | --- |
| 中英文单行OCR | 固定版本PP-OCRv4 ONNX和PaddleOCR字典，Apache-2.0 | 真实HTTPS下载、CPU推理与离线推理已验证；5.0.1资源已公开 |
| Zotero Connector1.0.1 | 本仓库原始MIT代码，目标Zotero10.0.6 | 1.0.1在独立宿主已实际安装、启用与配对；5.0.1 XPI公开下载及原生客户端校验通过 |
| 公式模型 | 完整原生模型链及分发许可未核实 | 不进入正式可安装目录 |
| 视觉语义模型 | 实验推理与权重分发授权分开 | 托管安装禁用，不进入正式可安装目录 |
| Chrome/Edge剪藏扩展 | 保留既有源码和宿主安装指引 | 未声明主代码分发许可证的历史归档不进入正式安装目录 |

OCR包装版本为1.0.1，兼容应用5.0.1，权重未变。详细权重URL、固定SHA及兼容协议见[目录契约](PACKAGE_CATALOG.md)。正式发布后Catalog的资产URL由实际origin、Tag与已上传文件生成，不能使用占位或猜测地址。

## 2. OCR安装与离线使用

打开“扩展与模型中心”→AI模型→OCR，查看来源、许可证、大小、Windows兼容性和依赖，明确勾选确认后安装。下载、精确编译清单/大小/SHA-256验证、受管安装、运行时加载、真实自检全部成功后才显示READY。本版不要求公私钥配置。失败保留旧版本；支持取消、重试、启用、停用及卸载受管文件，不删除科研图片或OCR记录。

安装成功后，在截图素材或PDF摘录进入OCR，选择本地模型，检查实际识别结果，可插入Markdown并更新原OCR全文索引。该模型识别中英文单行选区，不宣称整页复杂版面或数学公式识别。断网后再次启动并识别新图片，验证真实离线工作。

## 3. Zotero插件安装与连接

1. 发布前由维护者运行`python scripts/build-zotero-connector.py`构建确定性XPI，记录版本、SHA及精确目标版本。构建不是安装。
2. 正式发布后，在ResearchClip扩展中心下载并验证XPI，按文件选择导出。不要把XPI加载成Tauri插件。
3. 在Zotero的“工具 → 插件”中点击齿轮 → Install Plugin From File，选择`ResearchClip-Zotero-Connector-v1.0.1.xpi`；完成后检查显示Enabled。不修改签名或安全机制。目标严格限定10.0.6，其他版本需要重新验证，不能放宽版本范围假装兼容。
4. 在ResearchClip设置启用专用本机连接，在Zotero插件设置输入配对信息并明确授权。配对凭据不能公开提交。
5. 在独立练习文献右键“使用ResearchClip打开”，核对library ID、item/attachment key、指定页/批注、已有实例复用和反向定位。
6. 检查批注变化通知、取消配对、重连及待处理修改保留。仅下载XPI不能标记“宿主已安装”。

本轮真实宿主已完成安装、菜单、配对、PDF来源核对和便签变化通知。写入授权必须由用户在独立Zotero主动完成；云端同步沿用Zotero账号和设置，本机成功回读不代表云同步完成。测试不要使用唯一的真实文献或直接编辑Zotero SQLite。

插件Manifest要求非空更新目录URL。正式发布管线提供`zotero-updates.json`，其中插件ID、版本、目标10.0.6、XPI下载URL及SHA均与实际上传文件一致。当前目录尚未发布；本地安装不构成在线插件更新成功。

独立Zotero10.0.6已完成五类非ink创建、改字改色、删除共15项真实复读；真实群组库及云端同步仍需人工验收，条件冲突与错误保留另有自动化夹具回归。ink只读投影，私有手写与Markdown不会自动写回Zotero。云同步是否完成以Zotero自身状态为准。

## 4. 发布与手动升级

按用户要求不配置Catalog或Updater签名密钥。组件只从编译清单确定的HTTPS URL下载，核对精确大小和SHA-256；清单不能从网络或本地文件替换。主程序升级从实际GitHub Release手动下载NSIS并核对校验清单。自动更新已停用，没有latest.json及伪造签名文件。

修改组件权重、URL或版本必须交付新的主程序；不允许下载未知原生代码注入Tauri。发布工作流不访问签名Secret，保留完整测试、实际原生验收、Draft上传、下载哈希和重复发布保护。最终链接及结果见[最终报告](../qa/FINAL_RELEASE_REPORT.md)。

## 5. 从5.0.0升级

已安装的精确5.0.0内置OCR文件保持原位。5.0.1只为相同哈希、大小及资源定义更新本机安装证明，随后重新校验与CPU自检；自检失败仍显示明确错误。更改过的定义、不匹配文件及未知来源不能借此自动启用。

当前真实下载：[正式Release](https://github.com/wpy-sketch/ResearchClip-Codex/releases/tag/v5.0.1)、[Connector XPI](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.0.1/ResearchClip-Zotero-Connector-v1.0.1.xpi)、[OCR权重](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.0.1/rec.onnx)、[OCR字典](https://github.com/wpy-sketch/ResearchClip-Codex/releases/download/v5.0.1/characters.txt)。公开资源均已匿名下载核对SHA；中心成功下载XPI后显示DOWNLOADED_HOST_INSTALL_REQUIRED，仍需按Zotero机制安装并主动配对。
