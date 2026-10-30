# ResearchClip仓库整合（进行中）

2026-10-30。用户明确选择：将历史附件去重归档到ResearchClip-Codex，迁移各版说明；完整核验后将原ResearchClip设为只读迁移入口，不删除。

## 已核查事实

- 来源：[ResearchClip](https://github.com/wpy-sketch/ResearchClip)，main为ec867a9；目标：[ResearchClip-Codex](https://github.com/wpy-sketch/ResearchClip-Codex)。两边均为该登录账户可管理的公开仓库。
- 旧仓库47个Release、4,738个附件引用，总计约68.08GB；按GitHub记录的SHA-256去重为543个原始文件、约8.42GB。全部附件有SHA-256，最大单文件约134MB。
- 当前原源码版本历史已包含旧main；迁移工作流还会重新校验祖先关系，原样保留全部源标签对象及旧main归档分支。已有冲突引用拒绝覆盖。
- 用户选择的去重方案采用67个EXE/XPI直接下载，其余文件存入16个独立ZIP卷。各卷无需拼接；原来历史包的分片仍作为原始文件保留。每个原文件的版本、原名、大小、摘要、卷内位置及源URL均保存在映射中。

## 迁移与验证

固定输入为[审计计划](../release/researchclip-consolidation.json)。它目前是PLAN_NOT_YET_COPIED，不是已迁移成功声明。

新增工作流仅手动触发，固定源和目标仓库，使用目标仓库GITHUB_TOKEN；不需要新增Secret。按[GitHub官方说明](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)，该令牌推送历史引用不会重放旧版本工作流。

迁移只读取公开历史附件，校验原大小/SHA，将其作为数据写入归档，不执行、注入或解压到应用资料目录。ZIP卷使用平坦受限文件名及ZIP_STORED，上传前验证每个成员；公开后再次匿名下载所有直接文件、ZIP及索引，并逐一复核543个原文件的SHA。全部验证通过后迁移各版说明与下载映射，保持最新应用Release不变。

不覆盖已发布附件，不移动已有Tag，不删除原Release。只有本次受控草稿中同名、无摘要、未完成的上传记录允许按既有恢复规则清理；其他冲突停止处理。工作流不修改原仓库。原仓库只读归档将在目标数据、版本说明及新应用发布核验完成后执行。

## 当前结果

- 迁移契约单元测试6项通过：确定性ZIP、内容/成员/越界拒绝、远端不覆盖、全映射覆盖、令牌目标约束、公开归档禁止写入。
- 实际复制、匿名下载、标签保留及各版说明迁移结果待工作流完成后补录。
- 原仓库尚未归档；旧下载入口继续保留。GitHub星标、关注及社会关系不属于可合并的Git历史，保留在原仓库，不虚构已转移。
