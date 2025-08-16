**【软件设计文档 - Quantum Config (超详细版 for LLM Code Generation - v4 完整无省略版)】**

**版本：4.0**
**日期：2025年6月10日 **

**目录**
1.  项目概述与详细描述
2.  最终确认的架构方案描述
3.  项目结构或模块划分
    3.1. 顶层目录结构
    3.2. 项目中所有模块/组件的列表及其主要职责
    3.3. 建议的开发顺序
4.  核心模块/类/函数接口定义 (API)
    4.1. `macros` Crate (宏定义)
    4.2. `core` Crate (主库 API)
5.  关键功能流程或算法描述
    5.1. 配置加载流程 (`MyConfig::load()`)
    5.2. 异步配置加载流程 (`MyConfig::load_async()`)
    5.3. 配置文件模板生成流程 (`MyConfig::generate_template()`)
    5.4. 环境变量名构造逻辑
    5.5. 配置文件键名构造逻辑
6.  所需软件栈与依赖 (含版本)
    6.1. 工作区 `quantum_config/Cargo.toml`
    6.2. `core` Crate `quantum_config/core/Cargo.toml`
    6.3. `macros` Crate `quantum_config/macros/Cargo.toml`
7.  关键设计决策解释
8.  数据模型 (用户定义与内部元数据)
    8.1. 用户定义的配置结构体 (示例与约束)
    8.2. 内部元数据结构 (`QuantumConfigAppMeta`, `StructMeta`, `FieldMeta`)
9.  部署考虑
10. 非功能性需求实现策略
    10.1. 类型安全
    10.2. 易用性 (开发者体验 DX)
    10.3. 可定制性
    10.4. 性能
    10.5. 安全性与安全硬化
    10.6. 可维护性
    10.7. 可测试性
    10.8. 前向兼容性与宏的版本控制
11. 详细的 `#[config(...)]` 属性行为
    11.1. `default = <value_expr_string>`
        11.1.1. `default` 表达式的上下文与可见性
    11.2. `description = "<string>"`
    11.3. `name_config = "<string>"`
    11.4. `name_env = "<string>"`
    11.5. `name_clap_long = "<string>"`
    11.6. `name_clap_short = '<char_literal>'`
    11.7. `flatten`
    11.8. `skip`
    11.9. `clap(...)`
12. 嵌套结构体处理详解
13. `Option<T>` 字段处理详解
14. 错误处理 (`QuantumConfigError`) 详细定义
15. 测试策略
    15.1. 单元测试 (`core` Crate)
    15.2. 宏测试 (`macros` Crate)
    15.3. 集成测试 (项目根 `tests/` 目录)
    15.4. 文档测试

---

**1. 项目概述与详细描述**

*   **项目名称：** Quantum Config
*   **目标：** `Quantum Config` 是一个强大且灵活的 Rust 配置库，旨在简化应用程序配置的管理。它允许开发者通过简单的派生宏和字段属性，将配置从多种来源（包括 TOML/JSON/INI 文件、环境变量和命令行参数）无缝加载到自定义的 Rust 结构体中。
*   **核心价值：** 提供类型安全、易于使用、高度可定制、开发者友好、安全且具有前向兼容性的配置解决方案。
*   **哲学定位 (新增)：**
    > `Quantum Config` 并非旨在取代 `figment` 或 `clap`，而是作为一个"固执己见" (opinionated) 的上层框架。它通过封装这两个库 80% 的常用功能，为绝大多数应用场景提供了一个"开箱即用"、"约定优于配置"的解决方案。其核心价值在于**极致简化**，而非**极致灵活**。当用户需要 `clap` 的完整子命令系统或 `figment` 的全部高级特性时，直接使用这两个库可能是更合适的选择。
*   **主要功能概述：**
    *   **多源配置加载：** 支持从 TOML、JSON 和 INI 格式的配置文件、环境变量以及命令行参数加载配置。
    *   **优先级覆盖：** 严格按照预定义优先级（系统文件 < 用户文件 < 指定文件 < 环境变量 < 命令行参数）合并配置，高优先级源覆盖低优先级源。
    *   **过程宏驱动：** 通过 `#[derive(Config)]` 和字段级 `#[config(...)]` 属性，以及结构体级 `#[config(...)]` 属性（用于版本控制等元数据），简化配置结构体的定义和元数据指定。
    *   **与 `clap` 深度集成：** 自动处理命令行参数的解析，包括标准参数（如 `--version`, `--help`，由 `clap` 自动生成）和用户自定义参数，均通过 `#[config(...)]` 属性驱动。
    *   **自动命名与映射：** 为配置文件键和环境变量名提供灵活的默认生成规则（基于结构体和字段名，支持全局前缀），并允许显式覆盖。
    *   **嵌套结构体支持：** 完全支持任意深度的嵌套配置结构体，并能正确处理其命名和加载。
    *   **默认值与描述：** 允许为配置项指定默认值（作为 Rust 表达式）和描述信息，用于自动填充 `serde` 默认值、生成 `clap` 帮助信息和配置文件模板中的注释。
    *   **可选字段支持：** 通过 `Option<T>` 类型自然支持可选配置项。
    *   **配置文件模板生成：** 能够根据配置结构体定义，自动生成指定格式（TOML、JSON、INI）的配置文件模板，包含注释和默认值。
    *   **同步与异步 API：** 提供 `load()` 和 `load_async()` 两种加载方式，满足不同应用场景的需求。异步 API 通过 Cargo 特性 `async` 启用。
    *   **跨平台兼容：** 设计为在主流操作系统（Linux, macOS, Windows）上均可运行，特别是在配置文件路径解析方面。
    *   **基于 `figment` 构建：** 利用成熟的 `figment` 库作为核心配置加载和合并引擎，确保健壮性和功能丰富性。
    *   **详细的错误报告：** 提供清晰、携带上下文的错误信息 (`QuantumConfigError`)，指出配置错误的来源和原因。
    *   **日志与追踪：** 支持通过 `log` 门面进行日志记录，并可通过特性 `tracing-support` 启用更高级的 `tracing` 集成。
    *   **前向兼容性：** 通过在宏中引入可选的版本属性，确保库在未来版本升级时，用户可以平滑过渡，避免隐性的重大行为变更。
    *   **安全硬化：** 包含针对配置解析深度限制和敏感默认值编译时警告等安全增强措施。

---

**2. 最终确认的架构方案描述**

*   **架构模式：** 模块化与运行时元数据驱动。
*   **设计原理：**
    *   **`macros` Crate (过程宏库)：** 此 crate 负责在编译时处理用户定义的配置结构体。它通过 `#[derive(Config)]` 宏、结构体级 `#[config(...)]` 属性（如 `env_prefix`）和字段级 `#[quantum_config_opt(...)]` 属性，解析结构体定义和字段元数据。其核心职责包括：
        1.  **解析宏参数：** 从 `#[config(...)]` 中提取 `env_prefix`。
        2.  **解析字段属性：** 详细解析每个字段上的 `#[config(...)]` 属性，提取如默认值表达式、描述、各种名称覆盖、`flatten` 标记、`skip` 标记以及 `clap` 特定属性等信息。
        3.  **行为版本控制：** 宏内部的生成逻辑保持一致，以确保稳定的行为。
        4.  **生成 `ClapArgs` 结构体：**
            *   动态创建一个内部的、临时的结构体定义（例如 `__{UserStructName}ClapArgs`），并为其派生 `clap::Parser`。
            *   根据 `#[config(...)]` 中的 `description`, `default`, `name_clap_long/short`, `clap(...)` 等属性，为 `ClapArgs` 的字段生成相应的 `clap` 属性。
            *   特殊处理 `--config` 参数，用于指定配置文件路径。
        5.  **生成默认值函数：** 对于每个带有 `#[config(default = <value_expr>)]` 的字段，生成一个私有的静态函数，该函数返回指定的默认值。例如，`fn __default_field_name() -> FieldType { <value_expr> }`。
            *   **安全增强：** 在此阶段，如果字段名暗示敏感信息（如 `password`, `secret` 等）且 `default` 属性被使用，则触发编译时警告。
        6.  **修改用户结构体定义 (用于 `serde`)：**
            *   为用户结构体的每个相关字段添加 `#[serde(default = "path::to::__default_field_name")]` 属性（如果该字段有 `config(default)`）。
            *   为需要 `flatten` 的字段添加 `#[serde(flatten)]`。
        7.  **生成元数据初始化代码：** 生成代码，用于在运行时创建一个静态的或在首次加载时初始化的 `StructMeta` 实例（详见 `core` 库的 `meta` 模块）。此元数据将包含结构体名、字段名、类型信息（字符串表示）、`name_config`、`name_env`、默认值函数的引用（或路径）、`description` 等。
        8.  **生成 `Config` trait 实现：**
            *   `load()`: 生成调用 `core::runtime::load_config::<Self>(app_meta, struct_meta_accessor_fn, clap_args, clap_matches)` 的代码。
            *   `load_async()`: 生成调用 `core::runtime::load_config_async::<Self>(...)` 的代码。
            *   `generate_template()`: 生成调用 `core::template::generate_template::<Self>(struct_meta_accessor_fn, format)` 的代码。
    *   **`core` Crate (运行时库)：** 此 crate 包含 `Lingo` 的所有运行时逻辑。它定义了元数据结构体，提供了用于路径解析、Provider 创建、配置加载和模板生成的模块。其核心职责包括：
        1.  **定义元数据结构：** 定义 `QuantumConfigAppMeta` (存储应用名、环境前缀、行为版本、解析深度)、`StructMeta` (存储结构体级元数据) 和 `FieldMeta` (存储字段级元数据) 等结构体。这些结构体用于在运行时表示和使用从编译时收集的信息。
        2.  **路径解析模块 (`paths`)：** 实现确定各级配置文件（系统级、用户级）路径的逻辑。这包括处理不同操作系统的标准目录、我们约定的文件名模式 (`config.{ext}`, `{app_name}.{ext}`) 以及查找顺序。
        3.  **自定义 `figment` Provider 实现 (`providers` 模块)：**
            *   `QuantumConfigFileProvider`: 封装 `figment` 的文件加载，处理文件可选性（必需 vs. 可选），支持同步和异步文件读取，并应用可配置的解析深度限制。
            *   `QuantumConfigEnvProvider`: 实现从环境变量加载配置的逻辑。它会根据 `StructMeta` 和 `QuantumConfigAppMeta`（包括全局前缀、嵌套结构、`flatten` 规则、`name_env` 覆盖）动态构造预期的环境变量名，并将其值映射到 `figment` 理解的键路径。
            *   `QuantumConfigClapProvider`: 实现从 `clap` 解析结果加载配置的逻辑。它会特别处理布尔参数的来源（确保仅当用户显式提供时才覆盖低优先级源），并将其余命令行参数值提供给 `figment`。
        4.  **核心加载与编排逻辑 (`runtime` 模块)：** 包含 `load_config` (同步) 和 `load_config_async` (异步, 条件编译) 函数。这些函数是配置加载过程的总指挥，负责：实例化 `figment::Figment`；按正确的优先级顺序创建并添加所有 `QuantumConfig*Provider` 实例；调用 `figment.extract()` 或 `figment.extract_async()` 将配置值反序列化到用户结构体中；以及处理和转换错误。
        5.  **配置文件模板生成逻辑 (`template` 模块)：** 实现 `generate_template` 函数，该函数根据 `StructMeta` 和用户请求的格式 (`TemplateFormat` 枚举：TOML, JSON, INI) 生成包含注释、默认值和必需字段提示的配置文件模板字符串。
        6.  **错误类型定义 (`error` 模块)：** 定义 `QuantumConfigError` 枚举，这是一个全面的错误类型，用于报告配置加载过程中可能发生的各种问题，并携带详细的上下文信息。
        7.  **公共 API 定义 (`prelude` 模块与 `lib.rs`)：** 导出用户需要直接使用的类型和 trait，如 `Config` trait、`QuantumConfigError` 枚举、`TemplateFormat` 枚举以及 `Config` 派生宏本身。
*   **关键组件交互：** 编译时，`macros` crate 处理用户代码，生成辅助代码和元数据结构。运行时，用户调用 `load()` 等方法，这些方法（由宏生成）进而调用 `core` crate 中的函数，`core` crate 利用编译时生成的元数据来驱动配置加载过程。
*   **选择理由：** 此架构将复杂的编译时代码生成任务（宏）与清晰的运行时逻辑分离。这种分离提高了整体的可维护性、可测试性和代码清晰度。宏的职责更加集中于解析和代码转换，而运行时库则专注于配置加载的步骤和策略。这种设计也使得未来扩展新配置源或修改加载行为时，可能只需要修改运行时库，而无需触及复杂的宏代码。此架构还考虑了前向兼容性（通过宏版本控制）和安全性（通过解析深度限制和敏感默认值警告），旨在提供一个更健壮和用户友好的长期解决方案。

---

**3. 项目结构或模块划分**

**3.1. 顶层目录结构**

项目根目录为 `quantum_config/`，采用 Cargo 工作区 (workspace) 结构。

```
quantum_config/
├── Cargo.toml              # 工作区清单文件 (定义成员和共享依赖)
├── core/                   # 运行时库 crate
│   ├── Cargo.toml          # core crate 的清单文件
│   └── src/                # core crate 的源代码
│       ├── lib.rs          # core crate 的入口点，声明模块和导出 API
│       ├── prelude.rs      # 导出最常用的公共条目
│       ├── error.rs        # 定义 LingoError 枚举
│       ├── meta.rs         # 定义运行时元数据结构
│       ├── paths.rs        # 配置文件路径解析逻辑
│       ├── providers/      # figment Provider 实现的子模块
│       │   ├── mod.rs      # 声明 providers 子模块
│       │   ├── file_provider.rs  # LingoFileProvider 实现
│       │   ├── env_provider.rs   # LingoEnvProvider 实现
│       │   └── clap_provider.rs  # LingoClapProvider 实现
│       ├── runtime.rs      # 核心配置加载和编排逻辑
│       └── template.rs     # 配置文件模板生成逻辑
├── macros/                 # 过程宏 crate
│   ├── Cargo.toml          # macros crate 的清单文件
│   └── src/                # macros crate 的源代码
│       └── lib.rs          # macros crate 的入口点，定义派生宏
├── README.md               # 项目说明文件
├── LICENSE                 # 项目许可证文件 (例如 Apache-2.0)
├── examples/               # 示例用法目录 (可选，但强烈建议)
│   └── basic_usage.rs      # 一个基本的 Quantum Config 使用示例
└── tests/                  # 集成测试和宏测试的根目录
    ├── integration/        # 集成测试
    │   └── priority_test.rs
    │   # ... 其他集成测试文件 ...
    └── macros/             # 宏测试 (使用 trybuild)
        ├── pass/           # 应该成功编译的宏用法示例
        │   └── simple_struct.rs
        │   # ... 其他成功案例 ...
        └── fail/           # 应该导致编译失败的宏用法示例
            └── invalid_attribute.rs
            # ... 其他失败案例 ...
```

**3.2. 项目中所有模块/组件的列表及其主要职责**

*   **`macros` Crate (位于 `quantum_config/macros/`)**
    *   **`macros/src/lib.rs`**:
        *   **主要组件**: `#[proc_macro_derive(Config, attributes(config))]` 宏实现。
        *   **职责**:
            1.  **解析 `#[config(...)]` 结构体级属性**:
*   `env_prefix: String` (可选, 例如 `#[config(env_prefix = "MY_APP_")]`): 全局环境变量前缀。
            2.  **解析 `#[config(...)]` 字段级属性**: 对每个字段，详细解析其上的 `#[config(...)]` 属性，提取所有配置项（如 `default`, `description`, `name_config`, `name_env`, `name_clap_long`, `name_clap_short`, `flatten`, `skip`, `clap(...)`）。
            3.  **敏感默认值编译时警告**: 如果 `#[config(default = "...")]` 被使用，且字段名（`rust_name`）包含如 "password", "secret", "token", "api_key", "passwd", "credentials" 等常见敏感词列表中的子串（不区分大小写），则使用 `proc_macro::Diagnostic` API (如果 Rust 版本支持稳定化，或通过 `proc_macro_error` crate) 发出一个编译时警告。
            4.  **生成 `__{UserStructName}ClapArgs` 结构体**:
                *   动态创建一个新的 Rust 结构体定义，其名称基于用户结构体名（例如，前缀 `__LingoClapArgs_`）。
                *   为此新结构体派生 `#[derive(clap::Parser, Debug)]`。
                *   根据从 `#[config(...)]` 收集到的信息，为 `__{UserStructName}ClapArgs` 的相应字段生成 `clap` 属性（如 `#[clap(long = "...", short = '...', help = "...", default_value_t = ..., action = ...)]` 等）。
                *   自动为 `__{UserStructName}ClapArgs` 添加一个名为 `config` 的字段，类型为 `Option<std::path::PathBuf>`，并带有 `#[clap(long = "config", short = 'c', help = "Path to an additional configuration file")]` 属性，用于通过命令行指定额外的配置文件。
            5.  **生成默认值函数**:
                *   对于每个在 `#[config(default = "<expr_str>")]` 中提供了有效 Rust 表达式字符串 `<expr_str>` 的字段：
                    *   生成一个唯一的、私有的 (`#[doc(hidden)]`)、静态的函数，例如 `fn __quantum_config_default_{struct_name}_{field_name}() -> FieldType { <expr_str> }`。
                    *   `<expr_str>` 被直接用作此函数体内的代码。
                    *   `FieldType` 是该字段的实际 Rust 类型。
            6.  **修改用户结构体定义 (用于 `serde`)**:
                *   确保用户结构体派生了 `#[derive(serde::Deserialize)]`。如果用户已自行派生，则不重复添加。
                *   对于每个生成了默认值函数的字段，向该字段添加 `#[serde(default = "crate::path::to::__quantum_config_default_{struct_name}_{field_name}")]` 属性。路径必须是可从用户结构体定义处访问的。
                *   对于在 `#[quantum_config_opt(flatten)]` 中标记的字段，向该字段添加 `#[serde(flatten)]` 属性。
            7.  **生成元数据构建代码**:
                *   生成一个静态函数，例如 `fn __quantum_config_get_meta_{struct_name}() -> &'static crate::meta::StructMeta`。
                *   此函数负责在首次调用时（或作为静态初始化的一部分）构建并返回一个包含该结构体所有元数据（`StructMeta` 及其包含的 `FieldMeta` 列表）的静态引用。
                *   `FieldMeta` 的构建需要使用从 `#[quantum_config_opt(...)]` 解析出来的所有信息。
            8.  **实现 `Config` Trait**:
*   为用户结构体生成 `impl crate::Config for UserStructName { ... }`。
                *   **`load()` 方法实现**:
                    *   调用内部生成的 `__{UserStructName}ClapArgs::parse()` 获取 `clap_args_struct` 和 `clap_matches`。
                    *   构造 `crate::meta::QuantumConfigAppMeta` 实例。
                    *   调用 `__quantum_config_get_meta_{struct_name}()` 获取元数据。
                    *   调用 `crate::runtime::load_config::<Self>(app_meta, struct_meta_ref, clap_args_struct, clap_matches)`。
                *   **`load_async()` 方法实现 (被 `#[cfg(feature = "async")]` 包裹)**:
                    *   与 `load()` 类似，但调用 `crate::runtime::load_config_async::<Self>(...)`。
                *   **`generate_template()` 方法实现**:
                    *   调用 `__quantum_config_get_meta_{struct_name}()` 获取元数据。
                    *   调用 `crate::template::generate_template::<Self>(struct_meta_ref, format)`。
            9.  **错误信息优化**: 在宏生成的代码中（特别是默认值函数），添加详细的 `#[doc(hidden)]` 注释，解释该代码的来源和用途，以便在编译错误指向这些生成代码时，用户能获得更多上下文。

*   **`core` Crate (位于 `quantum_config/core/`)**
    *   **`core/src/lib.rs`**:
        *   **职责**: Crate 入口点。声明所有公共模块 (`pub mod error;` 等)。定义并导出 `pub trait Config`。通过 `prelude` 模块选择性地重新导出最常用的公共 API。
    *   **`core/src/prelude.rs`**:
        *   **职责**: 提供一个方便的模块，用户可以通过 `use quantum_config::prelude::*;` 导入常用项。
        *   **导出内容**: `Config` trait, `QuantumConfigError` enum, `TemplateFormat` enum, `Config` 派生宏 (通过 `pub use quantum_config_derive::Config;`)。
    *   **`core/src/error.rs`**:
        *   **主要组件**: `pub enum QuantumConfigError { ... }` (使用 `#[derive(thiserror::Error, Debug)]`)。
        *   **职责**: 定义所有 `Quantum Config` 库可能产生的错误类型，确保每个错误变体都携带足够的上下文信息以便用户调试。详见第 14 节。
    *   **`core/src/meta.rs`**:
        *   **主要组件**:
            *   `pub struct QuantumConfigAppMeta { pub app_name: String, pub env_prefix: Option<String>, pub behavior_version: u32, pub max_parse_depth: u32 }`
            *   `pub struct FieldMeta { ... }` (包含字段的 Rust 名、配置名、环境变量名、`clap` 参数名、描述、默认值函数路径字符串、类型字符串、`is_option`, `is_flatten`, `is_skipped` 等标志, 以及 `clap` 直接属性)
            *   `pub struct StructMeta { pub struct_name: &'static str, pub fields: Vec<FieldMeta>, pub is_top_level_config: bool, /* 处理嵌套元数据的方式 */ }`
        *   **职责**: 定义用于在运行时表示配置结构体及其字段元数据的结构。这些结构由 `macros` crate 生成的代码填充，并由 `core` crate 的其他模块（如 `providers`, `runtime`, `template`）使用。
    *   **`core/src/paths.rs`**:
        *   **主要组件**:
            *   `pub struct ConfigFilePath { pub path: std::path::PathBuf, pub file_type: ConfigFileType, pub is_required: bool }`
            *   `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ConfigFileType { Toml, Json, Ini }`
            *   `pub fn resolve_config_files(app_meta: &QuantumConfigAppMeta) -> Result<Vec<ConfigFilePath>, QuantumConfigError>`
        *   **职责**: 实现配置文件的路径解析逻辑。
            1.  根据 `app_meta.app_name` 和 `directories::ProjectDirs` (或直接使用 `directories` 的相关函数) 确定系统级和用户级的配置目录基路径。
            2.  按照预定义的查找顺序（系统级子目录 -> 系统级应用名目录 -> 用户级子目录 -> 用户级应用名目录）和文件名模式 (`config.{ext}`, `{app_name}.{ext}`，其中 `ext` 为 `toml`, `json`, `ini`) 生成所有潜在的配置文件路径。
            3.  对于每个生成的路径，检查文件是否存在。如果存在，推断 `ConfigFileType` 并构建 `ConfigFilePath` 对象。
            4.  系统级和用户级文件默认 `is_required = false`。
            5.  返回找到的所有有效 `ConfigFilePath` 列表，按优先级顺序排列（低优先级在前）。
    *   **`core/src/providers/mod.rs`**:
        *   **职责**: 声明 `file_provider`, `env_provider`, `clap_provider` 子模块。
    *   **`core/src/providers/file_provider.rs`**:
        *   **主要组件**: `pub(crate) struct QuantumConfigFileProvider { path: std::path::PathBuf, format: figment::providers::Format, is_required: bool, max_parse_depth: u32 }`
        *   **职责**: 实现 `figment::Provider` trait。
            *   `new(path: PathBuf, format_enum: ConfigFileType, is_required: bool, max_parse_depth: u32) -> Self` 构造函数。
            *   `fn metadata(&self) -> figment::Metadata`
            *   `fn data(&self) -> Result<figment::value::Map, figment::Error>` (同步版本):
                1.  尝试读取并解析指定路径的文件，应用 `max_parse_depth` 限制（如果底层解析器支持）。
                2.  如果 `is_required` 为 `false` 且文件未找到，则返回 `Ok(figment::value::Map::new())`。
                3.  处理其他错误。
            *   `async fn data_async(&self) -> Result<figment::value::Map, figment::Error>` (异步版本, 被 `#[cfg(feature = "async")]` 包裹):
                1.  使用 `tokio::fs` 异步读取文件，应用解析深度限制。
    *   **`core/src/providers/env_provider.rs`**:
        *   **主要组件**: `pub(crate) struct QuantumConfigEnvProvider<'a> { app_meta: &'a QuantumConfigAppMeta, struct_meta: &'a StructMeta }`
        *   **职责**: 实现 `figment::Provider` trait。
            *   `new(app_meta: &'a QuantumConfigAppMeta, struct_meta: &'a StructMeta) -> Self`
            *   `fn data(&self) -> Result<figment::value::Map, figment::Error>`:
                1.  遍历 `struct_meta.fields` (递归处理嵌套结构)。
                2.  构造环境变量名（考虑 `app_meta.env_prefix`、`field_meta.env_name_override`、`flatten`、结构体路径和字段名）。
                3.  读取环境变量，处理值，转换为 `figment::value::Value` 并插入 `Map`。
    *   **`core/src/providers/clap_provider.rs`**:
        *   **主要组件**: `pub(crate) struct QuantumConfigClapProvider<'a, 'm> { matches: &'m clap::ArgMatches, struct_meta: &'a StructMeta }`
        *   **职责**: 实现 `figment::Provider` trait。
            *   `new(matches: &'m clap::ArgMatches, struct_meta: &'a StructMeta) -> Self`
            *   `fn data(&self) -> Result<figment::value::Map, figment::Error>`:
                1.  遍历 `struct_meta.fields`。
                2.  对每个 `clap` 参数，使用 `matches.value_source()` 检查来源。
                3.  特别处理布尔值（仅当来源是 `CommandLine` 时提供）。
                4.  将有效值转换为 `figment::value::Value` 并插入 `Map`。
    *   **`core/src/runtime.rs`**:
        *   **主要组件**:
            *   `pub(crate) fn load_config<T>(app_meta: QuantumConfigAppMeta, struct_meta: &'static StructMeta, clap_args_struct: GeneratedClapArgsStruct, clap_matches: clap::ArgMatches) -> Result<T, QuantumConfigError>`
            *   `#[cfg(feature = "async")] pub(crate) async fn load_config_async<T>(...) -> Result<T, QuantumConfigError>`
        *   **职责**: 编排整个配置加载过程。
            1.  初始化 `figment::Figment`。
            2.  调用 `paths::resolve_config_files()`。
            3.  按优先级顺序创建并添加 `QuantumConfigFileProvider` (系统级，用户级，命令行指定)。
            4.  创建并添加 `QuantumConfigEnvProvider`。
            5.  创建并添加 `QuantumConfigClapProvider`。
            6.  调用 `figment.extract()` 或 `figment.extract_async()`。
            7.  错误转换并返回结果。
    *   **`core/src/template.rs`**:
        *   **主要组件**:
            *   `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum TemplateFormat { Toml, Json, Ini }`
            *   `pub(crate) fn generate_template<T: Config>(struct_meta: &'static StructMeta, format: TemplateFormat) -> Result<String, QuantumConfigError>`
        *   **职责**: 根据 `StructMeta` 和指定的 `TemplateFormat` 生成配置文件模板字符串。
            1.  遍历 `struct_meta.fields` (递归处理嵌套，考虑 `flatten` 和 `skip`)。
            2.  获取字段的配置名、描述、默认值（通过调用宏生成的默认值函数）、类型字符串。
            3.  根据 `TemplateFormat` 构建输出，包含注释、键值对（默认值或占位符）。

**3.3. 建议的开发顺序 (自底向上，详细)**

1.  **工作区与 Crate 结构搭建**:
    *   创建 `quantum_config/Cargo.toml` (工作区配置)。
    *   创建 `quantum_config/core/Cargo.toml` 和 `quantum_config/core/src/lib.rs`。
    *   创建 `quantum_config/quantum_config_derive/Cargo.toml` 和 `quantum_config/quantum_config_derive/src/lib.rs`。
    *   创建 `quantum_config/tests/` 目录结构 (`integration/`, `macros/pass/`, `macros/fail/`)。
2.  **`core` Crate - 核心类型定义**:
    *   `core/src/error.rs`: 定义 `QuantumConfigError` 枚举（包含所有变体和详细上下文）。
    *   `core/src/meta.rs`: 定义 `QuantumConfigAppMeta`, `StructMeta`, `FieldMeta` 的结构（包含所有字段如 `behavior_version`, `max_parse_depth`）。
    *   `core/src/template.rs`: 定义 `TemplateFormat` 枚举。
    *   `core/src/lib.rs`: 定义 `pub trait Config` (包含 `load`, `load_async`, `generate_template` 方法签名)。
    *   `core/src/prelude.rs`: 设置基本的导出。
3.  **`quantum_config_derive` Crate - 基本派生与属性解析**:
    *   `quantum_config_derive/src/lib.rs`:
        *   实现 `#[proc_macro_derive(Config)]` 的入口。
        *   使用 `darling` 解析 `#[config(...)]` 结构体级属性（仅保留 `env_prefix`）。
        *   定义用于解析 `#[config(...)]` 的结构 (e.g., `QuantumConfigFieldOptions` 使用 `darling::FromField`)。
        *   实现对字段及其 `config` 属性的初步解析。
        *   生成一个空的 `impl Config for UserStructName {}`。
4.  **`core` Crate - 路径解析模块**:
    *   `core/src/paths.rs`: 实现 `resolve_config_files` 函数的完整逻辑。编写单元测试。
5.  **`core` Crate - 文件 Provider (同步)**:
    *   `core/src/providers/file_provider.rs`: 实现 `QuantumConfigFileProvider` 的构造函数和同步 `data()` 方法，包括 `max_parse_depth` 的传递和（如果可能）应用。编写单元测试。
6.  **`core` Crate - 运行时 (同步，部分)**:
    *   `core/src/runtime.rs`: 开始实现 `load_config` 函数。集成路径解析和文件 Provider 加载逻辑。
7.  **`quantum_config_derive` Crate - `ClapArgs` 和元数据生成**:
    *   `quantum_config_derive/src/lib.rs`:
        *   实现 `__{UserStructName}ClapArgs` 结构体的生成逻辑。
        *   实现默认值函数的生成逻辑。
        *   实现敏感默认值编译时警告逻辑。
        *   实现为用户结构体字段添加 `#[serde(default = "...")]` 和 `#[serde(flatten)]` 的逻辑。
        *   实现 `StructMeta` 和 `FieldMeta` 实例构建代码的生成逻辑，以及访问它们的静态函数 `__quantum_config_get_meta_{struct_name}`。
        *   更新 `Config` trait 的 `load` 方法实现。
8.  **集成测试 - 文件加载与 `clap` (初步)**:
    *   在 `examples/` 目录下创建示例。测试从文件加载，以及 `--config` 参数。
9.  **`core` Crate - 环境变量 Provider (同步)**:
    *   `core/src/providers/env_provider.rs`: 实现 `QuantumConfigEnvProvider`。编写单元测试。
10. **`core` Crate - 命令行参数 Provider (同步)**:
    *   `core/src/providers/clap_provider.rs`: 实现 `QuantumConfigClapProvider`。编写单元测试。
11. **`core` Crate - 运行时 (同步，完整)**:
    *   `core/src/runtime.rs`: 完成 `load_config` 函数。
12. **端到端集成测试 (同步)**:
    *   在 `quantum_config/tests/integration/` 中编写全面的集成测试。
13. **`core` Crate - 模板生成模块**:
    *   `core/src/template.rs`: 实现 `generate_template` 函数。
    *   `quantum_config_derive/src/lib.rs`: 更新 `Config` trait 的 `generate_template` 方法实现。
    *   测试模板生成功能（快照测试）。
14. **异步支持 (`async` 特性)**:
    *   更新 `Cargo.toml` 文件。
    *   `core/src/providers/file_provider.rs`: 实现 `data_async()`。
    *   `core/src/runtime.rs`: 实现 `load_config_async` 函数。
    *   `quantum_config_derive/src/lib.rs`: 实现 `Config` trait 的 `load_async` 方法。
    *   编写异步集成测试。
15. **日志与追踪支持**:
    *   在 `core` 中集成 `log` 和 `tracing` API 调用。测试特性开关。
16. **宏版本控制逻辑实现**:
    *   在 `quantum_config_derive` crate 中，移除了基于 `version` 的行为分支，仅保留对 `#[config(env_prefix = ...)]` 的支持，简化宏行为。
17. **文档完善**: 编写用户文档、API 文档 (`#[doc(...)]`)，包括所有新特性和行为。
18. **测试策略全面实施**: 按照第 15 节的策略编写和执行所有类型的测试，特别是 `trybuild` 测试。
19. **代码审查、优化和发布准备。**

---

**4. 核心模块/类/函数接口定义 (API)**

**4.1. `quantum_config_derive` Crate (宏定义)**

*   **`#[proc_macro_derive(Config, attributes(config))]`**
    *   **应用目标：** 用户定义的配置结构体。
    *   **结构体级属性 `#[config(...)]` (应用于结构体本身)：**
        *   `env_prefix = "<string_literal>"` (可选):
            *   指定应用级环境变量前缀（如 `MY_APP_`），将自动作用于所有字段的环境变量名生成。
            *   用于构成默认的配置文件路径 (e.g., `~/.config/<app_name>/`) 和其他可能的用途。
            *   如果未提供，运行时将尝试从 `CARGO_PKG_NAME` 环境变量中获取。宏需要生成逻辑来处理这种情况。
        *   `env_prefix = "<string_literal>"` (可选):
            *   指定一个全局前缀，用于所有自动生成的环境变量名。
            *   例如，如果 `env_prefix = "MY_APP_"`，则字段 `db.port` 可能对应环境变量 `MY_APP_DB_PORT`。
        *   `max_parse_depth = <integer_literal>` (可选, e.g., `64`, default e.g., `128`):
            *   建议的配置文件解析最大深度。传递给 `QuantumConfigFileProvider`。
            *   如果未提供，`QuantumConfigFileProvider` 将使用一个内部的合理默认值。
    *   **字段属性 `#[config(...)]` (应用于结构体字段)：**
        *   `default = <value_expr_string>`: 提供字段的默认值，内容为 Rust 表达式字符串。
        *   `description = "<string_literal>"`: 字段描述，用于帮助信息和模板注释。
        *   `name_config = "<string_literal>"`: 覆盖字段在配置文件中的键名。
        *   `name_env = "<string_literal>"`: 完全覆盖字段的环境变量名。
        *   `name_clap_long = "<string_literal>"`: 覆盖字段的 `clap` 长参数名。
        *   `name_clap_short = '<char_literal>'`: 指定字段的 `clap` 短参数名。
        *   `flatten`: 标记属性，用于将嵌套结构体的字段提升到父级。
        *   `skip`: 标记属性，使 `Quantum Config` 完全忽略此字段。
        *   `clap(...)`: 元列表属性，允许传递原生 `clap` 属性。

**4.2. `core` Crate (主库 API)**

*   **`pub trait Config: Sized + serde::de::DeserializeOwned`**
    *   **定义位置：** `quantum_config/core/src/lib.rs` (或 `prelude.rs` 并导出)
    *   **方法：**
        ```rust
        /// Loads the configuration synchronously from all configured sources.
        ///
        /// This method will parse command line arguments, read relevant configuration
        /// files from system, user, and specified paths, and merge them with
        /// environment variables according to predefined priority.
        ///
        /// # Errors
        /// Returns `QuantumConfigError` if any part of the loading process fails,
        /// such as file I/O errors, parsing errors, or missing required values
        /// without defaults.
        fn load() -> Result<Self, crate::error::QuantumConfigError>;
        
        /// Loads the configuration asynchronously from all configured sources.
        ///
        /// This method performs the same operations as `load()` but uses asynchronous
        /// I/O for file operations, making it suitable for applications built on
        /// async runtimes like Tokio.
        ///
        /// This method is only available if the "async" feature is enabled.
        ///
        /// # Errors
        /// Returns `QuantumConfigError` if any part of the loading process fails.
        #[cfg(feature = "async")]
        fn load_async() -> impl std::future::Future<Output = Result<Self, crate::error::QuantumConfigError>> + Send;
        
        /// Generates a configuration file template based on the structure definition.
        ///
        /// The template will include comments (from `description` attributes) and
        /// default values (from `default` attributes). Fields without defaults
        /// that are required will be indicated according to the format.
        ///
        /// # Arguments
        /// * `format`: The desired output format for the template (TOML, JSON, or INI).
        ///
        /// # Errors
        /// Returns `QuantumConfigError` if template generation fails for any reason.
        fn generate_template(format: crate::template::TemplateFormat) -> Result<String, crate::error::QuantumConfigError>;
        ```

*   **`pub enum QuantumConfigError`**
    *   **定义位置：** `quantum_config/core/src/error.rs`
    *   **职责：** 统一的错误类型，携带详细上下文。
    *   **详细定义：** 见第 14 节 "错误处理 (`QuantumConfigError`) 详细定义"。

*   **`#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum TemplateFormat`**
    *   **定义位置：** `quantum_config/core/src/template.rs` (或 `lib.rs` 并导出)
    *   **变体：**
        *   `Toml`
        *   `Json`
        *   `Ini`

---

**5. 关键功能流程或算法描述**

**5.1. 配置加载流程 (`MyConfig::load()`) - 详细步骤**

1.  **用户调用 `UserStruct::load()`** (例如 `AppConfig::load()`).
2.  **宏生成的 `load` 方法执行**:
    a.  **初始化 `QuantumConfigAppMeta`**:
        *   仅从 `#[config(env_prefix = "...")]` 属性中解析环境变量前缀。
        *   如果 `app_name` 未在宏参数中提供，则设置一个标记或特殊值，指示 `core::runtime::load_config` 需要在运行时从 `std::env::var("CARGO_PKG_NAME").ok()` 获取。
        *   为 `behavior_version` 和 `max_parse_depth` 设置默认值（如果用户未提供）。
        *   构造 `core::meta::QuantumConfigAppMeta` 实例。
    b.  **解析命令行参数**:
        *   调用内部生成的 `__{UserStructName}ClapArgs::try_parse_from(std::env::args_os())`。
        *   获取 `clap_args_struct` 和 `clap_matches`。
    c.  **获取结构体元数据**:
        *   调用宏生成的静态函数 `__quantum_config_get_meta_{UserStructName}()`，该函数返回 `&'static core::meta::StructMeta`。
    d.  **调用核心加载逻辑**:
        *   调用 `core::runtime::load_config::<UserStruct>(app_meta, struct_meta_ref, clap_args_struct, clap_matches)`。
3.  **`core::runtime::load_config` 函数执行**:
    a.  **日志记录**: 记录开始加载配置的信息。
    b.  **`app_name` 解析 (如果需要)**: 如果 `app_meta.app_name` 指示需要从 `CARGO_PKG_NAME` 获取，则此时尝试读取环境变量。
    c.  **初始化 `Figment`**: `let mut figment = figment::Figment::new();`
    d.  **解析配置文件路径**: 调用 `core::paths::resolve_config_files(&app_meta)`。处理错误。
    e.  **添加系统和用户配置文件 Providers**:
        *   遍历返回的路径列表。对每个 `ConfigFilePath`：
            *   创建 `core::providers::QuantumConfigFileProvider::new(path.path, format_enum, path.is_required, app_meta.max_parse_depth)`。
            *   调用 `figment = figment.merge(file_provider);`。
    f.  **添加命令行指定的配置文件 Provider**:
        *   从 `clap_args_struct.config` 获取路径。如果提供：
            *   创建 `core::providers::QuantumConfigFileProvider::new(path, format_enum, true, app_meta.max_parse_depth)`。
            *   `figment = figment.merge(specified_file_provider);`
    g.  **添加环境变量 Provider**:
        *   创建 `core::providers::QuantumConfigEnvProvider::new(&app_meta, struct_meta);`
        *   `figment = figment.merge(env_provider);`
    h.  **添加命令行参数 Provider**:
        *   创建 `core::providers::QuantumConfigClapProvider::new(&clap_matches, struct_meta);`
        *   `figment = figment.merge(clap_provider);`
    i.  **提取配置**: 调用 `figment.extract::<T>()`。
    j.  **返回结果**: `Ok(extracted_config)` 或 `Err(QuantumConfigError)`.

**5.2. 异步配置加载流程 (`MyConfig::load_async()`) - 详细步骤**

此流程与同步版本 (`load()`) 非常相似，主要区别在于文件读取和 `figment` 的提取调用是异步的。仅当 `async` 特性启用时可用。

1.  **用户调用 `UserStruct::load_async().await`**.
2.  **宏生成的 `load_async` 方法执行**:
    a.  与同步版本相同的 `QuantumConfigAppMeta` 初始化、`clap` 解析、`StructMeta` 获取。
    b.  调用 `core::runtime::load_config_async::<UserStruct>(app_meta, struct_meta_ref, clap_args_struct, clap_matches).await`。
3.  **`core::runtime::load_config_async` 函数执行**:
    a.  日志记录。
    b.  `app_name` 解析 (同同步)。
    c.  初始化 `Figment` (同同步)。
    d.  解析配置文件路径 (同同步)。
    e.  **添加系统和用户配置文件 Providers (异步感知)**:
        *   遍历路径列表。对每个 `ConfigFilePath`，创建 `QuantumConfigFileProvider` (传入 `max_parse_depth`)。
        *   调用 `figment = figment.merge(file_provider);`
    f.  **添加命令行指定的配置文件 Provider (异步感知)**: 同上。
    g.  **添加环境变量 Provider**: `QuantumConfigEnvProvider` 的 `data_async` 可以委托给同步 `data`。
    h.  **添加命令行参数 Provider**: `QuantumConfigClapProvider` 的 `data_async` 可以委托给同步 `data`。
    i.  **异步提取配置**: 调用 `figment.extract_async::<T>().await`。
    j.  **返回结果**: (同同步，错误类型相同)。

**5.3. 配置文件模板生成流程 (`MyConfig::generate_template()`) - 详细步骤**

1.  **用户调用 `UserStruct::generate_template(format)`**.
2.  **宏生成的 `generate_template` 方法执行**:
    a.  **获取结构体元数据**: 调用宏生成的静态函数 `__quantum_config_get_meta_{UserStructName}()`。
    b.  **调用核心模板生成逻辑**: 调用 `core::template::generate_template::<UserStruct>(struct_meta_ref, format)`。
3.  **`core::template::generate_template` 函数执行**:
    a.  **日志记录**。
    b.  初始化一个 `String` 用于构建模板。
    c.  **递归辅助函数**: `fn build_template_recursive(struct_meta: &'static StructMeta, current_prefix: &str, builder: &mut String, format: TemplateFormat, indent_level: usize)`
    d.  调用 `build_template_recursive` 处理顶层 `struct_meta`。
    e.  **在 `build_template_recursive` 内部**:
        *   遍历 `struct_meta.fields`。
        *   对于每个未被 `#[config(skip)]` 的 `field_meta`:
            i.  **处理 `flatten`**: 如果 `field_meta.is_flatten`，则递归调用处理嵌套结构体的元数据。
            ii. **处理普通字段或非 `flatten` 的嵌套结构体**:
                *   获取配置名、描述、默认值（通过运行时调用宏生成的默认值函数）、类型字符串。
                *   根据 `TemplateFormat` 将字段信息写入 `builder`:
                    *   **TOML/INI**: 注释、键值对（默认值或占位符）。必需但无默认值的字段用注释和类型提示。
                    *   **JSON**: 键值对。必需但无默认值的字段值为 `null`。
            iii. **处理非 `flatten` 的嵌套结构体字段**: 写入节头，递归调用。
    f.  返回构建好的 `String`，或 `QuantumConfigError::TemplateGeneration`。

**5.4. 环境变量名构造逻辑 (在 `QuantumConfigEnvProvider` 中)**

*   **输入**: `app_meta.env_prefix`, `app_meta.behavior_version`, `struct_meta` (递归), `field_meta`.
*   **基础**: 字段的 Rust 名，转换为大写，用下划线 `_` 分隔。
*   **`name_env` 覆盖**: 如果 `field_meta.env_name_override` 有值，直接使用此名称（不应用前缀或结构体路径）。
*   **自动生成 (无 `name_env` 覆盖)**:
    1.  **组件列表**: 初始化空列表。
    2.  **全局前缀**: 如果 `app_meta.env_prefix` 为 `Some(prefix)`，添加 `prefix`。
    3.  **结构体路径 (处理嵌套和 `flatten`, 可能受 `behavior_version` 影响)**:
        *   从顶层结构体追溯到当前字段。
        *   对路径上的每个结构体层级（除非被 `flatten`）：获取结构体名或其字段名，转换为大写蛇形，添加到组件列表。
    4.  **字段名**: 将当前 `field_meta.rust_name` 转换为大写蛇形。添加到组件列表。
    5.  **组合**: 用下划线 `_` 连接所有组件。
*   **示例**: `app_prefix = "MYAPP"`, `AppConf { server: ServerConf { port: u16 } }` -> `MYAPP_SERVER_PORT`.

**5.5. 配置文件键名构造逻辑 (用于 `figment` 的键路径)**

*   **输入**: `app_meta.behavior_version`, `struct_meta` (递归), `field_meta`.
*   **基础**: 字段的 Rust 名 (通常蛇形 `snake_case`)。
*   **`name_config` 覆盖**: 如果 `field_meta.config_name_override` 有值，使用此名称。
*   **自动生成 (无 `name_config` 覆盖)**:
    1.  **组件列表**: 初始化空列表。
    2.  **结构体路径 (处理嵌套和 `flatten`, 可能受 `behavior_version` 影响)**:
        *   从顶层结构体追溯到当前字段。
        *   对路径上的每个结构体层级（除非被 `flatten`）：获取该层级字段的名称，添加到组件列表。
    3.  **字段名**: 将当前 `field_meta.rust_name` (或其 `name_config` 覆盖) 添加到组件列表。
    4.  **组合**: 用点号 `.` 连接所有组件。
*   **示例**: `AppConfig { server_settings: ServerConfig { port_number: u16 } }` -> `server_settings.port_number`.

---

**6. 所需软件栈与依赖 (含版本)**

**6.1. 工作区 `quantum_config/Cargo.toml`**
```toml
# quantum_config/Cargo.toml
[workspace]
resolver = "2"
members = [
    "quantum_config_derive",
]

[workspace.package]
edition = "2021"
version = "1.0.0"
license = "Apache-2.0"
authors = ["Kirky-X <Kirky-X@outlook.com>"]
repository = "https://github.com/Kirky-X/quantum_config"
description = "Quantum Config: A flexible and easy-to-use configuration library for Rust applications."
readme = "README.md"
keywords = ["config", "configuration", "settings", "figment", "clap"]
categories = ["config", "command-line-interface"]

[workspace.dependencies]
# Serde
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
toml_edit = { version = "0.22.17", features = ["parse", "display"] }
rust-ini = "0.21.1"

# Figment (核心依赖)
figment = { version = "0.10.19", features = ["toml", "json", "env", "ini", "tokio"] }

# Clap (命令行解析)
clap = { version = "4.5.39", features = ["derive", "env"] }

# Error Handling
thiserror = "2.0.12"

# Logging & Tracing
log = "0.4.27"
tracing = "0.1.41"
tracing-subscriber = { version = "0.3.19", features = ["fmt", "env-filter", "json"] }

# Async Runtime (for async API)
tokio = { version = "1.45.0", features = ["fs", "rt"] }

# Filesystem Paths
directories = "6.0.0"

# Proc Macro Crates (仅 derive 宏包直接依赖)
syn = { version = "2.0.100", features = ["full", "extra-traits"] }
quote = "1.0.40"
proc-macro2 = "1.0.95"
darling = "0.20.11"

# Quantum Config crates themselves (用于成员间依赖)
quantum_config_derive = { path = "quantum_config_derive", version = "0.2.0" }

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
# opt-level = 0
# debug = true
```

**6.2. `core` Crate `quantum_config/core/Cargo.toml`**
```toml
# quantum_config/core/Cargo.toml
[package]
name = "core"
version = "0.2.0"
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Core runtime library for Quantum Config, providing configuration loading and management."
readme.workspace = true
keywords.workspace = true
categories.workspace = true

[dependencies]
# Inherited from workspace
serde = { workspace = true }
serde_json = { workspace = true }
toml_edit = { workspace = true }
rust-ini = { workspace = true }
figment = { workspace = true }
clap = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true, optional = true }
tokio = { workspace = true, optional = true }
directories = { workspace = true }

macros = { workspace = true }

[features]
default = ["log-facade"]
log-facade = ["log"]
tracing-support = ["tracing", "dep:tracing-subscriber", "log-facade"]
async = ["dep:tokio", "figment/tokio"]
```

**6.3. `macros` Crate `quantum_config/quantum_config_derive/Cargo.toml`**
```toml
# quantum_config/quantum_config_derive/Cargo.toml
[package]
name = "quantum_config_derive"
version = "0.2.0"
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Procedural macros for the Quantum Config configuration library."
readme.workspace = true
keywords.workspace = true
categories.workspace = true

[lib]
proc-macro = true

[dependencies]
# Inherited from workspace
syn = { workspace = true }
quote = { workspace = true }
proc-macro2 = { workspace = true }
darling = { workspace = true }
# Optional, for better diagnostics
# proc-macro-error = "1.0.4"

[dev-dependencies]
trybuild = "1.0.97"
```

---

**7. 关键设计决策解释**

*   **宏与运行时分离：** 编译时宏负责代码生成和元数据收集，运行时库负责实际的加载逻辑。这提高了可维护性、可测试性和代码清晰度。宏的复杂性得到控制。
*   **基于 `figment`：** 利用成熟库处理底层的配置合并和提取，避免重复造轮子，确保健壮性。
*   **`clap` 集成方式：** 宏生成一个内部的 `ClapArgs` 结构体，而不是直接修改用户结构体以派生 `Parser`。这提供了更大的灵活性来定制 `clap` 参数，而不会污染用户结构体的属性。
*   **命令行参数 Provider (`QuantumConfigClapProvider`) 行为：** 明确决定通过检查 `clap::ArgMatches::value_source()` 来仅处理用户显式提供的命令行参数（特别是对布尔值），以避免 `clap` 的内部默认值不期望地覆盖低优先级配置源。
*   **异步 API (`async` feature)：** 通过 Cargo 特性提供可选的异步支持，并明确将 `tokio` 作为依赖项（当特性启用时），确保库的自包含性。
*   **配置文件查找顺序：** 详细定义了系统级、用户级配置文件的查找路径和内部文件名优先级 (`config.{ext}` vs `{app_name}.{ext}` 以及子目录优先)，并决定加载所有找到的匹配文件，优先级由添加顺序决定。
*   **错误处理：** 使用 `thiserror` 定义统一的 `QuantumConfigError` 枚举，提供清晰的错误来源信息和丰富的上下文。
*   **模板生成中对必需字段的处理：** JSON 中用 `null`，TOML/INI 中用注释和类型提示。
*   **`default` 属性实现：** 通过生成静态默认值函数并使用 `#[serde(default = "...")]` 指向它，而不是直接嵌入表达式字符串。这允许更复杂的默认值表达式。
*   **嵌套结构体处理：** 子结构体需 `Deserialize`，可选 `Config` (如果需要其字段的 `config` 特性)。命名规则和 `flatten` 行为已定义，以直观地映射配置。
*   **`Option<T>` 字段处理：** 作为可选值的标准方式，行为符合 `serde` 预期，未提供值时为 `None`。
*   **日志与追踪 (`log` 和 `tracing`)**: 提供了基础的 `log` 支持，并通过特性 `tracing-support` 提供更高级的 `tracing` 集成，给予用户选择。
*   **工作区结构**: 采用 Cargo 工作区管理 `core` 和 `macros` 两个 crate，便于版本和依赖的统一管理。
*   **`toml_edit` 的选择**: 使用 `toml_edit` 而非 `toml` 是为了在模板生成（未来可能还有配置写回）时更好地控制 TOML 格式和注释。
*   **移除显式宏版本控制：** 不再提供 `version` 属性控制行为，库行为以当前版本为准，并在变更时通过语义化版本管理与迁移指南保障兼容性。
*   **主动安全硬化措施：**
    *   **解析深度限制：** 通过内置默认策略和 `QuantumConfigFileProvider` 的实现，防御针对配置文件解析的资源耗尽攻击。
    *   **敏感默认值编译时警告：** `macros` crate 在编译时检测潜在的硬编码敏感默认值，并向开发者发出警告，提升安全意识。
*   **强化开发者体验 (`default` 表达式上下文文档)：** 通过详细文档和优化的宏错误提示，帮助用户正确使用 `#[config(default = "...")]` 这一强大但易错的功能。
*   **具体化的测试策略 (使用 `trybuild` 进行宏测试)：** 确保宏的各种预期行为（成功和失败案例）都得到验证，提升宏的健壮性。
*   **决策：v1.0 采用全封装的加载流程，暂不提供自定义 Provider 注入点。**
    *   **理由：** 为了最大化 v1.0 的易用性和 API 简洁性，我们选择将配置加载流程完全封装。这带来的明确权衡是牺牲了对自定义配置源（如远程配置中心、数据库等）的直接扩展能力。需要此功能的用户在当前版本需要自行编排 `figment`，或者等待未来版本可能引入的 Builder API。提供公开的 Builder API 以支持自定义 Provider 注入已被记录为未来可能的发展方向 (v2.0 考虑项)。

---

**8. 数据模型 (用户定义与内部元数据)**

**8.1. 用户定义的配置结构体 (示例与约束)**

用户通过定义普通的 Rust 结构体来指定其应用程序的配置。这些结构体必须满足以下条件才能与 `Quantum Config` 一起使用：

*   **必须派生 `#[derive(Config)]`**: 这是启用 `Quantum Config` 功能的入口点。
*   **可选应用结构体级 `#[config(...)]` 属性**: 当前仅支持 `env_prefix`。
*   **（隐式）必须可被 `serde::Deserialize`**: 派生宏会确保这一点，或者用户可以自行添加 `#[derive(serde::Deserialize)]`。通常也会派生 `Debug`, `Clone`, `serde::Serialize` 等。
*   **字段类型**: 字段类型必须是 `serde::Deserialize` 本身支持的，或者是其他也满足这些条件的嵌套结构体。
*   **字段属性**: 可以使用 `#[config(...)]` 属性来定制字段的行为。

**示例：**
```rust
// main.rs or lib.rs where config is defined
use quantum_config::prelude::*;

#[derive(Config, Debug, serde::Serialize)]
#[config(env_prefix = "MY_APP_")]
struct AppConfig {
    #[config(description = "Server host address")]
    host: String,

    #[config(default = "8080", description = "Server port number")]
    port: u16,

    #[config(description = "Enable feature X", name_clap_long = "enable-x")]
    feature_x_enabled: bool,

    #[config(skip, description = "This field is ignored by Quantum Config")]
    #[allow(dead_code)]
    temp_data: String,

    database: DatabaseConfig,

    #[quantum_config_opt(description = "Optional API key for external service")]
    api_key: Option<String>,

    #[quantum_config_opt(
        description = "Path to the log file",
        name_config = "log_file_path",
        name_env = "APP_LOG_LOCATION",
        name_clap_long = "log-path"
    )]
    logs: std::path::PathBuf,

    #[quantum_config_opt(flatten)]
    worker: WorkerConfig,
}

#[derive(Config, Debug, serde::Deserialize, serde::Serialize)] // Config for its fields' defaults
// no version attribute; behavior version removed
struct DatabaseConfig {
    #[quantum_config_opt(default = r#""postgres://user:pass@localhost/mydb".to_string()"#)]
    url: String,
    #[quantum_config_opt(default = "5")]
    pool_size: u32,
}

#[derive(Config, Debug, serde::Deserialize, serde::Serialize)]
struct WorkerConfig {
    #[quantum_config_opt(default = "4")]
    threads: usize,
    #[quantum_config_opt(description = "Queue name for workers")]
    queue_name: String,
}
```

**8.2. 内部元数据结构 (`LingoAppMeta`, `StructMeta`, `FieldMeta`)**

这些结构定义在 `lingo/core/src/meta.rs` 中，用于在运行时表示从编译时收集的配置信息。

*   **`pub struct LingoAppMeta`**:
    *   `pub app_name: String`: 应用程序的名称。
    *   `pub env_prefix: Option<String>`: 全局环境变量前缀（可由 `#[config(env_prefix = ...)]` 指定）。
    *   `pub behavior_version: u32`: 宏行为版本（已移除版本属性控制，固定为当前库行为）。
    *   `pub max_parse_depth: u32`: 配置文件解析深度限制（移除属性控制，保留默认策略）。

*   **`pub struct FieldMeta`**: (每个实例代表用户结构体中的一个字段)
    *   `pub rust_name: &'static str`: 字段在 Rust 结构体中的原始名称。
    *   `pub config_name_override: Option<&'static str>`: 来自 `#[quantum_config_opt(name_config = "...")]` 的值。
    *   `pub env_name_override: Option<&'static str>`: 来自 `#[quantum_config_opt(name_env = "...")]` 的值。
    *   `pub clap_long_override: Option<&'static str>`: 来自 `#[quantum_config_opt(name_clap_long = "...")]` 的值。
    *   `pub clap_short_override: Option<char>`: 来自 `#[quantum_config_opt(name_clap_short = "...")]` 的值。
    *   `pub description: Option<&'static str>`: 来自 `#[quantum_config_opt(description = "...")]`。
    *   `pub default_fn_path_str: Option<&'static str>`: 指向宏生成的默认值函数的完整路径字符串。
    *   `pub type_name_str: &'static str`: 字段类型的字符串表示。
    *   `pub is_option: bool`: 标记该字段是否是 `Option<T>` 类型。
    *   `pub is_flatten: bool`: 标记该字段是否有 `#[quantum_config_opt(flatten)]`。
    *   `pub is_skipped: bool`: 标记该字段是否有 `#[quantum_config_opt(skip)]`。
    *   `pub clap_direct_attrs_meta: Option<ClapAttrsMeta>`: 结构化表示来自 `#[quantum_config_opt(clap(...))]` 的原生 `clap` 属性。`ClapAttrsMeta` 会包含如 `long: Option<String>`, `short: Option<char>`, `help: Option<String>`, `action_code: Option<TokenStream>`, `value_parser_code: Option<TokenStream>` 等。

*   **`pub struct StructMeta`**: (每个实例代表一个用户定义的配置结构体，或一个嵌套的配置结构体)
    *   `pub struct_name: &'static str`: 结构体的 Rust 名称。
    *   `pub fields: Vec<FieldMeta>`: 该结构体包含的所有字段的元数据列表。
    *   `pub is_top_level_config: bool`: 标记这是否是用户直接派生 `Config` 的顶层结构体。
    *   `pub nested_struct_meta_map: std::collections::HashMap<&'static str, &'static StructMeta>`: 用于存储此结构体中类型为其他配置结构体的字段的元数据。键是字段的 `rust_name`，值是对应嵌套结构体的 `StructMeta` 引用。`macros` 负责构建这个图。

---

**9. 部署考虑**

*   **作为库：** `Lingo` 主要作为库被其他 Rust 应用程序依赖。
*   **依赖项：** 最终应用程序的二进制大小会受到 `Lingo` 及其传递依赖项的影响。`tokio` (如果启用 `async` 特性) 是一个相对较大的依赖。`macros` crate 的依赖（如 `syn`, `quote`, `darling`）仅在编译时需要，不影响最终二进制大小。
*   **特性标志：** 用户可以通过 Cargo 特性 (`async`, `tracing-support`) 来控制包含哪些可选功能，从而管理依赖项和最终二进制大小。
*   **跨平台：**
    *   配置文件路径解析使用 `directories` crate，它处理不同操作系统的标准目录。
    *   核心逻辑是平台无关的 Rust 代码。
    *   需要确保在 Windows, macOS, Linux 上都能正确查找和读取文件，并进行相应测试。
*   **编译时间：** 过程宏（尤其是复杂的）会增加编译时间。`macros` crate 的设计应力求高效，避免不必要的复杂计算或代码生成。用户在大型项目中引入 `Lingo` 可能会观察到编译时间的增加。
*   **MSRV (Minimum Supported Rust Version)：** 由 `Lingo` 及其所有依赖项中最高的 MSRV 决定。当前选择的依赖版本通常需要较新的 Rust 编译器 (e.g., Rust 1.70+)。这应在项目 `README.md` 中明确说明。

---

**10. 非功能性需求实现策略**

**10.1. 类型安全**
*   **实现：** 利用 Rust 强大的静态类型系统。所有配置值在加载后都具有明确的类型。`serde` 在反序列化时进行类型检查。
*   **保证：** 编译时和运行时（通过 `serde`）的类型检查，防止类型不匹配的配置错误。

**10.2. 易用性 (开发者体验 DX)**
*   **实现：**
    *   通过 `#[derive(Config)]` 简化配置结构体的设置。
*   通过 `#[config(...)]` 和 `#[quantum_config_opt(...)]` 属性以声明方式定制行为。
*   提供简洁的 API (`load()`, `load_async()`, `generate_template()`).
    *   清晰的错误消息 (`LingoError` 携带上下文)。
    *   自动处理多源加载和优先级。
    *   **`default` 表达式上下文文档：** 在用户文档和 `#[quantum_config_opt(default = "...")]` 的 API 文档中详细说明表达式的编译上下文、可见性规则和正确使用示例。
    *   **宏错误信息优化：** `macros` crate 尽可能提供指向用户代码的、有意义的编译错误。对于因宏展开而产生的内部错误，通过 `#[doc(hidden)]` 注释或 `proc-macro-error` 辅助提供线索。
*   **保证：** 减少样板代码，直观的 API 设计，帮助用户快速上手并避免常见错误。
*   **权衡：易用性与“魔法”的代价。**
    > `Lingo` 通过过程宏提供了极高的易用性，其代价是可能增加项目的编译时间，尤其是在大型项目中进行干净构建时。此外，当出现深层次的配置问题或宏的行为与预期不符时，调试可能会涉及宏生成的内部代码，这天然比直接调试手写代码更具挑战性。本库通过详尽的错误信息、清晰的文档（尤其是关于 `default` 表达式上下文的说明）以及推荐的测试策略（如使用 `trybuild`）来努力缓解这一问题，力求打造“成功之坑”。

**10.3. 可定制性**
*   **实现：**
    *   `#[quantum_config_opt(...)]` 提供对字段命名、默认值、描述、`clap` 参数等的细粒度控制。
    *   `#[config(...)]` 仅提供环境前缀控制（env_prefix），移除了版本与解析深度等属性。
    *   支持多种配置文件格式 (TOML, JSON, INI)。
    *   支持环境变量和命令行参数。
    *   `flatten` 选项用于定制嵌套结构体的展现方式。
*   **保证：** 用户可以根据需求调整配置加载的几乎所有方面。

**10.4. 性能**
*   **实现：**
    *   配置加载通常在应用程序启动时进行一次，不是性能关键路径。但仍需注意：
    *   `macros` crate 生成的代码应高效，避免不必要的运行时开销。
    *   `core` crate 中的 Provider 实现应尽可能高效地读取和解析数据。
    *   异步 API (`load_async()`) 使用 `tokio` 避免阻塞主线程（如果应用是异步的）。
    *   元数据结构 (`StructMeta`, `FieldMeta`) 设计为静态或首次使用时构建，后续访问开销小。
*   **保证：** 对于典型配置大小，加载时间应在可接受范围内。

**10.5. 安全性与安全硬化**
*   **实现：**
    *   不直接处理敏感信息存储（如密码加密），这是应用程序的责任。
    *   依赖 `serde` 进行安全的(反)序列化。
    *   文件路径处理应注意避免路径遍历等问题（主要使用标准库和 `directories`）。
    *   环境变量读取是标准操作。
    *   命令行参数解析由 `clap` 处理，`clap` 是一个成熟且安全的库。
    *   **解析深度限制：** `QuantumConfigFileProvider` 在调用底层文件解析器时，应用内置的解析深度限制（可在内部通过常量或配置项控制），以防止因恶意构造的深嵌套配置文件导致的资源耗尽。
    *   **敏感默认值编译时警告：** `macros` crate 在编译时扫描 `#[quantum_config_opt(default = "...")]` 的字段名，如果匹配常见的敏感词列表，则发出编译警告，提醒开发者避免在源代码中硬编码敏感默认值。
    *   **依赖项安全：** 定期审查和更新依赖项，关注 `cargo audit` 等工具的报告。
*   **保证：** 库本身不应引入新的安全漏洞，并主动采取措施提升配置相关的安全性。

**10.6. 可维护性**
*   **实现：**
    *   清晰的模块化设计 (`macros` vs `core`, `core` 内部的 `paths`, `providers`, `runtime`, `template` 等模块)。
    *   职责分离（编译时代码生成 vs. 运行时逻辑）。
    *   代码注释和本文档这样的详细设计文档。
    *   使用 `thiserror` 管理错误。
*   **保证：** 代码易于理解、修改和扩展。

**10.7. 可测试性**
*   **实现：**
    *   `core` 库的各个模块可以进行单元测试。
    *   `macros` crate 的测试使用 `trybuild` 进行编译成功/失败的验证。
    *   提供 `examples/` 作为集成测试和用法演示。
    *   详细的测试策略见第 15 节。
*   **保证：** 关键逻辑路径都有测试覆盖，宏的行为得到验证。

**10.8. 前向兼容性与宏的版本控制**
*   **实现：** 取消结构体级 `version` 属性，改为通过库的语义化版本与变更日志对行为变更进行管理。
*   **行为：**
    *   如果用户未指定 `version`，宏默认采用当前 `Quantum Config` 库版本所定义的最新行为。
    *   如果用户指定了 `version = N`，则宏（即使是更新版本的 `Quantum Config` 库中的宏）会生成与 `Quantum Config` 行为版本 `N` 相兼容的代码。
    *   这要求 `macros` crate 内部维护针对不同 `behavior_version` 的条件逻辑或代码生成路径。
*   **保证：** 用户可以控制何时采用可能不兼容的新行为，从而实现平滑升级。新功能或行为变更可以通过新的版本号引入。

---

**11. 详细的 `#[quantum_config_opt(...)]` 属性行为**

`#[quantum_config_opt(...)]` 是一个应用于结构体字段的属性，用于定制该字段在配置加载、命令行参数生成和模板生成等方面的行为。

**11.1. `default = <value_expr_string>`**
*   **类型：** `<value_expr_string>` 是一个字符串字面量，其内容必须是一个有效的 Rust 表达式。
*   **行为：**
    1.  `macros` crate 解析此字符串表达式。
    2.  宏生成一个私有的静态函数，例如 `fn __quantum_config_default_{struct_name}_{field_name}() -> FieldType { <value_expr_string 的内容作为代码> }`。此函数的返回类型 `FieldType` 必须与应用此属性的字段的类型相匹配。
    3.  宏在该字段上添加 `#[serde(default = "path::to::generated_default_fn")]` 属性，指向这个生成的函数。
    4.  当 `figment.extract()` 进行反序列化时，如果该字段在所有配置源中都没有值，`serde` 会调用此默认值函数来获取值。
    5.  此默认值也会被用于 `generate_template()` 生成的模板中。
    6.  此默认值也会被用于为对应的 `clap` 参数生成 `default_value_t` 或 `default_value_os_t` 属性（如果适用且未被 `clap(...)` 中的定义覆盖）。宏需要将表达式的结果（如果是简单类型）转换为字符串。对于复杂类型，可能无法直接映射到 `clap` 的简单默认值字符串。
    7.  **敏感默认值警告：** 如果此属性被使用，且其应用的字段名（不区分大小写）包含 "password", "secret", "token", "api_key", "passwd", "credentials", "keyfile", "private_key" 等预定义列表中的关键词，`macros` crate 将在编译时产生一个警告，建议用户不要硬编码敏感默认值。
*   **示例：**
    *   `#[quantum_config_opt(default = "8080")] port: u16,`
    *   `#[quantum_config_opt(default = r#""localhost".to_string()"#)] host: String,`
    *   `#[quantum_config_opt(default = "std::path::PathBuf::from(\"/var/log/app.log\")")] log_path: std::path::PathBuf,`
    *   `#[quantum_config_opt(default = "vec![1, 2, 3]")] numbers: Vec<i32>,`

**11.1.1. `default` 表达式的上下文与可见性**
*   **执行上下文：** 用户在 `default = "..."` 中提供的 Rust 表达式字符串，在宏展开后，会被放入一个生成的静态函数中。这个静态函数与用户定义的配置结构体位于同一个模块（或者是一个由宏创建的、对用户结构体可见的子模块）内。
*   **可见性规则：**
    1.  **当前模块项：** 表达式可以直接访问同一模块内定义的 `pub` 或非 `pub` (私有) 的函数、常量、静态变量等。
    2.  **导入项：** 如果表达式需要使用已通过 `use` 语句导入到当前模块的项，可以直接使用它们。
    3.  **完整路径：** 对于其他 crate 或当前 crate 其他模块中的 `pub` 项，必须使用其完整路径，例如 `std::time::Duration::from_secs(5)`，`crate::my_module::my_helper_fn()`，`other_crate::some_value`。
    4.  **Trait 方法：** 如果表达式调用了某个 trait 的方法 (e.g., `.to_string()`)，该 trait 必须在当前模块的作用域内（通常通过 `use some_crate::SomeTrait;` 导入）。
*   **常见陷阱与建议：**
    *   **私有辅助函数：** 如果默认值依赖于一个定义在同一模块但对生成的静态函数不可见的辅助函数，会导致编译错误。确保辅助函数对生成的默认值函数可见。
    *   **宏调用：** 表达式可以是宏调用，例如 `default = "vec![1, 2, 3]"`。确保该宏在当前模块可见。
    *   **复杂类型构造：** 对于复杂的默认值，建议将其构造逻辑封装在一个独立的、公开的或在当前模块可见的函数中，然后在 `default` 表达式中调用该函数。
*   **宏的错误提示：** 如果因为 `default` 表达式导致编译失败，`macros` crate 应（尽其所能）使错误信息指向用户提供的表达式字符串，或者在生成的默认值函数周围提供足够的上下文信息（通过 `#[doc(hidden)]` 注释），帮助用户定位问题。

**11.2. `description = "<string_literal>"`**
*   **类型：** `<string_literal>` 是一个字符串字面量。
*   **行为：**
    1.  此描述字符串将用于为对应的 `clap` 参数生成 `help = "..."` 属性（除非被 `clap(help = ...)` 覆盖）。
    2.  此描述字符串将作为注释包含在由 `generate_template()` 生成的配置文件模板中（对于支持注释的格式如 TOML, INI）。
*   **示例：** `#[quantum_config_opt(description = "The network port to listen on.")] port: u16,`

**11.3. `name_config = "<string_literal>"`**
*   **类型：** `<string_literal>` 是一个字符串字面量。
*   **行为：**
    1.  覆盖此字段在配置文件 (TOML, JSON, INI) 中的键名。
    2.  如果不提供，默认键名通常是字段的 Rust 名称 (e.g., `my_field` -> `my_field`)。
    3.  `QuantumConfigEnvProvider` 和 `QuantumConfigClapProvider` 在构造 `figment` 键路径时，如果需要从配置文件键名派生，会使用此覆盖值。
    4.  `generate_template()` 会使用此名称作为模板中的键。
*   **示例：** `#[quantum_config_opt(name_config = "serverPort")] port: u16,`

**11.4. `name_env = "<string_literal>"`**
*   **类型：** `<string_literal>` 是一个字符串字面量，通常应符合环境变量的命名约定（大写，下划线分隔）。
*   **行为：**
    1.  **完全覆盖**此字段对应的环境变量名。
    2.  如果提供此属性，`QuantumConfigEnvProvider` 将直接使用此字符串作为环境变量名来查找，而**不会**应用全局 `env_prefix` 或基于结构体路径和字段名的自动命名规则。
    3.  如果不提供，环境变量名将根据全局 `env_prefix` (如果存在)、结构体路径 (考虑嵌套和 `flatten`) 和字段的 Rust 名称 (转换为大写蛇形) 自动生成。
*   **示例：** `#[quantum_config_opt(name_env = "SERVICE_PORT")] port: u16,`

**11.5. `name_clap_long = "<string_literal>"`**
*   **类型：** `<string_literal>` 是一个字符串字面量，应符合 `clap` 长参数名的约定 (通常是小写kebab-case)。
*   **行为：**
    1.  覆盖此字段生成的 `clap` 长参数名。例如，`#[clap(long = "<value>")]`。
    2.  如果不提供，默认长参数名由 `clap` 根据字段名生成 (通常是kebab-case, e.g., `my_field` -> `my-field`)。
    3.  此属性优先于 `clap` 的自动命名，但会被 `#[quantum_config_opt(clap(long = "..."))]` 中的显式设置覆盖。
*   **示例：** `#[quantum_config_opt(name_clap_long = "server-port")] port: u16,`

**11.6. `name_clap_short = '<char_literal>'`**
*   **类型：** `<char_literal>` 是一个字符字面量 (e.g., `'p'`)。
*   **行为：**
    1.  指定此字段生成的 `clap` 短参数名。例如，`#[clap(short = '<value>')]`。
    2.  如果不提供，通常不会自动生成短参数名。
    3.  此属性会被 `#[quantum_config_opt(clap(short = ...))]` 中的显式设置覆盖。
*   **示例：** `#[quantum_config_opt(name_clap_short = 'p')] port: u16,`

**11.7. `flatten`**
*   **类型：** 标记属性 (无值，出现即为 `true`)。
*   **行为：**
    1.  **仅适用于类型为另一个结构体的字段。**
    2.  **`serde` 行为：** 宏在此字段上添加 `#[serde(flatten)]`。
    3.  **配置文件键名：** 嵌套结构体的字段键名将直接出现在父结构的命名空间下。
    4.  **环境变量名：** 自动生成的环境变量名也会将嵌套结构体的字段视为在父级。
    5.  **`clap` 参数：** 宏会将 `#[clap(flatten)]` 属性添加到生成的 `ClapArgs` 结构体中对应的嵌套结构体字段上。
    6.  **元数据影响：** `FieldMeta.is_flatten` 设为 `true`。
*   **示例：** 见 8.1 节中的 `WorkerConfig` 示例。

**11.8. `skip`**
*   **类型：** 标记属性。
*   **行为：**
    1.  此字段将被 `Quantum Config` 完全忽略（不参与配置加载、命令行参数、模板生成）。
    2.  `Quantum Config` 的 `skip` 不会自动添加 `#[serde(skip)]`。
    3.  **元数据影响：** `FieldMeta.is_skipped` 设为 `true`。
*   **示例：** `#[quantum_config_opt(skip)] transient_state: MyInternalType,`

**11.9. `clap(...)`**
*   **类型：** 接受一个元列表作为参数，其内容应该模拟 `clap` 字段属性的语法。
*   **行为：**
    1.  允许用户直接传递原生的 `clap` 属性给生成的 `ClapArgs` 结构体的对应字段。
    2.  `macros` crate 会解析这些属性并应用。
    3.  **优先级与合并：** `clap(...)` 内指定的属性通常具有最高优先级，会覆盖由其他 `quantum_config_opt` 属性（如 `description`, `default`, `name_clap_long/short`）推断出的 `clap` 属性。
*   **示例：**
    ```rust
    #[quantum_config_opt(
        description = "Number of retries",
        default = "3",
        clap(
            long = "retry-count",
            value_parser = clap::value_parser!(u8).range(1..=10),
            help = "Number of times to retry an operation (1-10)"
        )
    )]
    retries: u8,
    ```

---

**12. 嵌套结构体处理详解**

`Quantum Config` 对嵌套结构体的支持是其核心功能之一，旨在让复杂的配置结构易于管理。

*   **定义：** 用户可以在一个派生了 `#[derive(Config)]` 的结构体中，包含另一个结构体作为字段。
    ```rust
    #[derive(Config, Debug)]
    struct AppConfig {
        server: ServerConfig,
    }
    
    #[derive(serde::Deserialize, Debug)]
    // #[derive(Config)] // 可选：若 ServerConfig 的字段需要 quantum_config_opt 特性
    struct ServerConfig {
        host: String,
        port: u16,
    }
    ```
*   **对嵌套结构体的要求：**
    1.  **`serde::Deserialize`：** 嵌套结构体**必须**派生 `serde::Deserialize`。
    2.  **`Config` (可选但推荐用于其字段特性)：** 如果嵌套结构体的字段本身需要使用 `#[quantum_config_opt(...)]` 属性（例如，为其字段指定 `default` 值），那么该嵌套结构体**也应该**派生 `#[derive(Config)]`。
*   **配置加载行为 (无 `flatten`)：**
    *   **配置文件 (TOML, JSON, INI)：** 期望嵌套结构体的配置值位于以父字段名命名的节或对象中 (e.g., TOML: `[server]`, JSON: `"server": {}`).
    *   **环境变量：** 默认情况下，环境变量名会包含结构体路径 (e.g., `MY_APP_SERVER_PORT`).
    *   **命令行参数：** 默认不为嵌套字段生成顶级参数。使用 `flatten` 来暴露它们。
*   **使用 `#[quantum_config_opt(flatten)]` 处理嵌套结构体：**
    *   当在父结构体的嵌套结构体字段上使用 `#[quantum_config_opt(flatten)]` 时：
        *   **配置文件：** 嵌套结构体的字段被视为直接属于父结构体。
        *   **环境变量：** 嵌套结构体的字段的环境变量名也会被“提升”。
        *   **命令行参数：** `macros` crate 会对 `ClapArgs` 中对应字段应用 `#[clap(flatten)]`，使嵌套参数暴露在父命令级别。
*   **元数据处理 (`StructMeta`, `FieldMeta`)：**
    *   `macros` crate 在为父结构体生成 `StructMeta` 时，会递归获取或生成嵌套结构体的 `StructMeta`，并通过 `nested_struct_meta_map` 字段进行链接。

---

**13. `Option<T>` 字段处理详解**

`Option<T>` 用于表示一个配置项是完全可选的。

*   **定义：**
    ```rust
    #[derive(Config, Debug)]
    struct Config {
        #[quantum_config_opt(description = "An optional API key")]
        api_key: Option<String>,
        #[quantum_config_opt(default = "Some(3u8)")]
        retries: Option<u8>,
    }
    ```
*   **行为：**
    1.  **`serde::Deserialize`：** 如果配置源中不存在该键，字段为 `None`。
    2.  **`Quantum Config` 加载逻辑：** 如果 `Option<T>` 字段在所有配置源中都没有值，且无 `default`，则加载后为 `None`。这不是错误。
    3.  **默认值 (`#[quantum_config_opt(default = ...)]`) 与 `Option<T>`：** 可以为 `Option<T>` 字段指定 `Some(...)` 或 `None` 作为默认值。
    4.  **环境变量：** 未设置的环境变量不会为 `Option<T>` 提供值。空字符串环境变量可能导致 `Some("")`。
    5.  **命令行参数 (`clap`)：** `Option<T>` 字段生成的 `clap` 参数是可选的。未提供则为 `None`，不覆盖低优先级源。
    6.  **配置文件模板：** 根据是否有 `default`，模板中会显示默认值，或标记为可选/省略。

---

**14. 错误处理 (`QuantumConfigError`) 详细定义**

定义在 `quantum_config/core/src/error.rs`。

```rust
use crate::template::TemplateFormat;
use std::path::PathBuf;
use thiserror::Error;

/// Represents the type of configuration directory being accessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDirType {
    System,
    User,
}

impl std::fmt::Display for ConfigDirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigDirType::System => write!(f, "system"),
            ConfigDirType::User => write!(f, "user"),
        }
    }
}

/// The unified error type for all operations within the Quantum Config library.
#[derive(Error, Debug)]
pub enum QuantumConfigError {
    #[error("I/O error for path {path:?}: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("Failed to parse {format_name} file {path:?}: {source_error}")]
    FileParse {
        format_name: String,
        path: PathBuf,
        source_error: String,
    },

    #[error("Configuration extraction error: {0}")]
    Figment(#[from] figment::Error),

    #[error("Command line argument parsing error: {0}")]
    Clap(#[from] clap::Error),

    #[error("A required value was missing for key: {key_path}")]
    MissingValue { key_path: String },

    #[error("Invalid value for key '{key_path}': {message}")]
    InvalidValue { key_path: String, message: String },

    #[error("Configuration directory for {dir_type} not found. Expected at: {expected_path:?}")]
    ConfigDirNotFound {
        dir_type: ConfigDirType,
        expected_path: Option<PathBuf>,
    },

    #[error("No supported configuration files found in {dir_type} directory: {path:?}")]
    NoConfigFilesFoundInDir {
        dir_type: ConfigDirType,
        path: PathBuf,
    },

    #[error("Specified configuration file not found: {path:?}")]
    SpecifiedFileNotFound { path: PathBuf },

    #[error("Unsupported configuration file format for: {path:?}")]
    UnsupportedFormat { path: PathBuf },

    #[error("Error generating {format:?} template: {reason}")]
    TemplateGeneration {
        format: TemplateFormat,
        reason: String,
    },

    #[error("Internal Quantum Config error: {0}")]
    Internal(String),

    #[error("Failed to determine application name: {source_error}")]
    AppNameResolution { source_error: String },
}
```
**Error Handling Strategy:**
*   每个可能失败的函数返回 `Result<T, QuantumConfigError>`。
*   底层错误被转换为 `QuantumConfigError` 变体，保留源错误信息。
*   错误消息力求清晰和可操作。

---

**15. 测试策略**

为确保 `Quantum Config` 库的质量、健壮性和正确性，将采用以下多层次的测试策略：

**15.1. 单元测试 (`core` Crate)**
*   **位置：** 每个模块 (`.rs` 文件) 内部的 `#[cfg(test)] mod tests { ... }`。
*   **目标：** 测试 `core` 库中各个独立单元的逻辑。
*   **具体内容：**
    *   **`paths.rs`**: 测试路径解析的正确性，模拟不同操作系统环境。
    *   **`providers/*.rs`**: 对每个 Provider 进行独立测试。`QuantumConfigEnvProvider` 测试需设置和清理环境变量。`QuantumConfigFileProvider` 测试文件读写、解析、错误处理和深度限制。
    *   **`template.rs`**: 对每种格式的模板生成进行快照测试 (使用 `insta` crate)。
    *   **`runtime.rs`**: 测试内部辅助函数，`app_name` 解析逻辑。

**15.2. 宏测试 (`macros` Crate)**
*   **位置：** `quantum_config/macros/tests/macros/` (或 `quantum_config/macros/tests/`)。
*   **工具：** **强制要求使用 `trybuild` crate。**
*   **目标：** 验证 `#[derive(Config)]` 宏在各种输入下的行为。
*   **具体内容：**
    *   **`pass/` 子目录：** 包含所有应该成功编译的宏用法示例（简单、嵌套、`flatten`、所有 `quantum_config` 和 `quantum_config_opt` 属性等）。
    *   **`fail/` 子目录：** 包含所有应该导致编译失败的用法，并验证编译器错误信息（例如，属性参数类型错误、`default` 表达式语法错误等）。测试敏感默认值警告是否按预期触发。

**15.3. 集成测试 (项目根 `lingo/tests/` 目录)**
*   **位置：** `quantum_config/tests/integration/` (或直接在 `quantum_config/tests/`)。
*   **目标：** 测试 `core` 和 `macros` 两个 crate 协同工作的端到端行为。
*   **具体内容：**
    *   **优先级测试**: 创建复杂配置，通过文件、环境变量、命令行参数同时设置不同值，断言最终加载符合预期优先级。
    *   **多格式文件测试**: 测试从 `.toml`, `.json`, `.ini` 文件中加载。
    *   **嵌套与 `flatten` 测试**: 测试嵌套结构体在所有配置源中的加载和命名映射。
    *   **`Option<T>` 与默认值测试**: 测试 `Option<T>` 和 `default` 属性在各种情况下的行为。
    *   **`async` API 测试**: 使用 `#[tokio::test]` 测试 `load_async()`。
    *   **模板生成测试**: 对代表性结构体生成所有格式模板，进行快照或内容验证。
    *   **错误处理测试**: 故意制造错误条件，断言返回的 `QuantumConfigError` 类型和内容符合预期。
    *   **安全特性测试**: 测试解析深度限制。

**15.4. 文档测试**
*   **位置：** `README.md`，`quantum_config/core/src/lib.rs`，`quantum_config/core/src/prelude.rs` 及其他公共 API 文档注释。
*   **工具：** `cargo test --doc`。
*   **目标：** 确保所有文档中的代码示例正确、可编译、可运行，并符合描述。
*   **要求：** 所有公开的函数、结构体、枚举、宏都应有清晰的 Rustdoc 文档和（如果适用）用法示例。

