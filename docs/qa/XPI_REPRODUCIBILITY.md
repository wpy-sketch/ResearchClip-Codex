# Connector XPI 构建重现

标签v5.1.0的Windows CI测试成功，构建被XPI严格哈希检查拦截（[原运行](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41164223818)）。本机已验收的Connector1.0.1为10,503字节、SHA-256 `1f98896eeef54ead00c5405d2460578d3269d82a5a011038779c9a9e2973fdc2`，未替换。

隔离工作树中的修复只影响打包：若本地DEFLATE容器不同，从既有v5.0.1公开Release读取固定原包，验证大小、完整SHA、五个条目唯一性、文件集合和每一份源码字节；任何源码差异或下载损坏均拒绝，绝不以解压内容近似或跳过哈希制造通过。不安装/执行下载的XPI，不更换插件或移动标签。

实际18项发布/XPI单元测试退出0；另用不同ZIP容器通过真实HTTPS下载固定原包，核对五份源码和完整SHA退出0。原生zlib1.3.2与Node自带1.3.1生成不同压缩容器已在本地观察；CI原文件对应关系由后续实际运行继续核实，不将压缩实现预判成唯一原因。[修复后完整Windows CI](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41186713225)已成功，最终插件仍为同一10503字节/固定SHA，NSIS构建完成；主程序源码指纹保持不变，修复已合并main。发布资产保持本机已验收的原字节，未移动标签或覆盖资产。另新增受限大模型传输5项失败路径测试，本地合计23项通过；[传输实际运行](https://github.com/wpy-sketch/ResearchClip-Codex/actions/runs/41187263608)成功。
