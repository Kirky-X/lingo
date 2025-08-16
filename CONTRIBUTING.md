# 贡献指南（CONTRIBUTING）

感谢您对 Lingo 的关注与贡献！本文档说明如何为本项目做出高质量贡献，并帮助您顺利完成从开发到提交 PR 的流程。

## 项目概览

Lingo 是一个以易用性、可扩展性为目标的 Rust 配置管理库，包含：
- 主库：`lingo/`（核心能力与提供器）
- 派生宏：`lingo-derive/`（派生宏、属性宏）
- 示例：`examples/`（多个端到端场景示例）

## 开发环境

- Rust 稳定版工具链（建议使用 `rustup` 安装与管理）
- 推荐 IDE：VS Code / IntelliJ Rust / 其他 Rust 友好 IDE
- Windows、macOS、Linux 任意平台均可

> 提示：CI 会在提交后自动进行格式化、静态检查和测试，但请务必先在本地通过以下检查，以减少往返成本。

## 快速开始

1) 克隆仓库并编译：
```
cargo build
```

2) 运行全部测试（包含工作区所有 crate）：
```
cargo test --workspace --all-features
```

3) 运行常见示例：
```
# 基础示例
cargo run --manifest-path examples/basic/Cargo.toml

# 嵌套配置示例
cargo run --manifest-path examples/nested/Cargo.toml

# Web Server 示例（启动本地服务）
cargo run --manifest-path examples/web_server/Cargo.toml
```

## 代码规范

在提交前请确保：
- 代码已格式化：
```
cargo fmt --all
# 或仅校验：
cargo fmt --all -- --check
```
- 通过 Clippy 严格检查（将警告视为错误）：
```
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
- 测试全部通过：
```
cargo test --workspace --all-features
```

建议：
- 尽量保持函数简短、命名清晰、消除魔法数字与重复代码
- 遵循 SOLID 原则与 Clean Code 实践
- 为新功能与缺陷修复补充或更新测试用例

### 代码风格细节（rustfmt）

- 当前仓库未提供 `rustfmt.toml`，请使用 Rust 稳定版工具链的默认格式化规则。
- CI/本地门禁：`cargo fmt --all -- --check` 与 `cargo clippy -- -D warnings` 会共同确保风格与静态检查一致。
- 如果团队需要在本地固定更一致的风格，可在后续引入 `rustfmt.toml`。以下为推荐模板（仅供参考，是否采用以团队共识为准）：
```
edition = "2021"
use_field_init_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
wrap_comments = true
newline_style = "Unix"
```
> 说明：一旦本仓库引入 `rustfmt.toml`，以该文件为最终准绳；在此之前请遵循默认规则并通过 `cargo fmt` 校验。

## 分支策略

- 主分支：`main`
- 功能分支：`feature/<简述>`（例如：`feature/add-env-provider-option`）
- 修复分支：`fix/<简述>`（例如：`fix/panic-when-empty-config`）
- 文档或构建相关：`docs/<简述>`、`chore/<简述>`、`ci/<简述>`、`build/<简述>` 等

## 提交信息规范（Conventional Commits）

提交信息请遵循 Conventional Commits，以便生成变更日志与自动化处理：
- 类型常用：`feat`、`fix`、`docs`、`refactor`、`test`、`chore`、`ci`、`build`、`perf`
- 可选范围（scope）：如 `core`、`derive`、`examples/web_server` 等
- 简要示例：
```
feat(core): add layered provider merge strategy
fix(derive): handle empty attributes without panic
docs(examples): update README for template generator
chore: bump version to 0.2.0 across examples
```

## Pull Request 指南

在创建 PR 前，请确认：
- [x] 已通过 `cargo fmt`、`cargo clippy -D warnings`、`cargo test` 的本地检查
- [x] 相关示例（若受影响）可以正常运行
- [x] 变更已更新相应文档/注释/示例
- [x] 若涉及用户可见变更，已在 `CHANGELOG.md` 的 Unreleased 小节添加条目
- [x] PR 描述中包含：动机、变更点、影响面、测试要点、回滚策略（如需）

评审标准（Reviewers 会重点关注）：
- 设计合理性（边界条件、错误处理、可维护性）
- 代码质量（简洁、可读、无坏味道、性能合理）
- 兼容性与风险（是否破坏现有 API，是否需要迁移）
- 测试完整性（单元/集成/示例验证）

## 测试与示例

- 单元测试集中在各 crate 的 `src/` 与 `tests/`
- 集成测试与端到端行为可参考 `examples/`，建议对新增特性提供最小可复现示例
- 对涉及配置解析/覆盖优先级的改动，请添加覆盖不同来源（默认值、文件、环境变量、命令行）的测试

## 版本与发布（SemVer）

- 本项目遵循语义化版本（Semantic Versioning）
- 如涉及对外 API 的破坏性变更，请提升主版本（MAJOR）并在文档中明确迁移指南
- 涉及功能新增提升次版本（MINOR），向后兼容修复提升修订号（PATCH）
- 涉及跨目录版本同步时（如 `examples/`、文档示例中的版本号），请确保一并更新，保持一致性

## 变更日志（CHANGELOG）

- 所有用户可见变更应记录在 `CHANGELOG.md` 的 `Unreleased` 段落
- 结构建议：`Added`、`Changed`、`Fixed`、`Removed`、`Security`
- 版本发布时将 `Unreleased` 内容归档到对应版本段落

## 提交 Issue 的建议

- 缺陷（Bug）：请提供复现步骤、期望行为、实际行为、日志（如有）、受影响版本、操作系统与编译器版本
- 新特性（Feature）：请说明使用场景、动机与预期收益，若可，附上最小原型或接口草案
- 性能问题：请尽量提供基准数据或复现场景，便于定位与优化

## 贡献者公约（Code of Conduct）

我们致力于营造开放、友善、包容、无骚扰的社区环境。参与本项目的任何人（包括维护者、贡献者、使用者）应当：
- 相互尊重、善意沟通，不进行人身攻击与骚扰；
- 尊重不同意见与背景，接纳建设性建议；
- 以问题为中心讨论技术分歧，避免偏离话题；
- 在公共空间（Issue/PR/讨论区/会议等）遵守同等标准。

报告与执行：
- 如您遇到违规行为，可在 Issue 中添加 `conduct` 标签并 @ 维护者，避免披露敏感个人信息；
- 维护者可视情况采取：提醒、正式警告、临时或永久限制参与等措施；
- 对执行结果有异议，可在 Issue 中提出申诉并说明理由，维护者将进行复核。

> 说明：本节为项目行为准则的简要版，后续如引入独立的 `CODE_OF_CONDUCT.md`，以该文档为准。

## 贡献者许可协议（CLA）

- 个人贡献：除非仓库显式启用 CLA 全量校验，个人贡献默认以提交 PR 的形式，按照仓库 `LICENSE` 授权进行分发与使用，无需额外签署。
- 组织贡献：若您代表组织（公司/机构）贡献代码，建议在首次提交 PR 之前完成组织 CLA，以明确知识产权与授权范围。

组织 CLA 流程（建议）：
1) 提交 Issue，标题建议为 `[CLA] 组织签署申请`，并提供：
   - 组织法定名称与注册地址；
   - GitHub 组织名（如有）；
   - 法务/授权联系人姓名与邮箱；
   - 适用范围（本仓库/子仓库/特定模块）；
2) 维护者将与您确认并提供电子签署流程（如 GitHub CLA Assistant / DocuSign 等）以及 PR 校验方式（如 CLA bot 状态检查）；
3) 完成签署后，后续 PR 将自动通过 CLA 校验（若仓库启用 CLA 检查）。

权利与义务（摘要）：
- 贡献者声明拥有对提交内容的相应权利，不侵犯第三方权益；
- 授权项目在 `LICENSE` 约束下复制、修改、再许可与分发贡献内容；
- 组织信息变更需及时告知以更新记录；撤销组织 CLA 不影响既有贡献的既有授权。

> 说明：当前仓库未内置 CLA 文档与自动化校验，若您或您的组织需要，请按照以上流程联系维护者，我们将协助推进。

## 安全问题披露

- 如您发现潜在安全问题，请先创建 Issue 并使用 `security` 标签，避免在公开讨论中披露细节。维护者会与您沟通后续处理方式
- 在修复发布前，请勿公开技术细节与复现方式

## 许可协议

- 本项目遵循仓库根目录的 `LICENSE`

---

感谢每一位贡献者的投入！如有任何疑问，欢迎在 Issue 中讨论并与维护者沟通。