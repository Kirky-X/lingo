//! Quantum Config 元数据结构模块
//!
//! 定义了用于在运行时表示从编译时收集的配置信息的数据结构。

use std::collections::HashMap;

/// 应用程序级别的元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantumConfigAppMeta {
    /// 应用程序的名称
    pub app_name: String,
    /// 全局环境变量前缀
    pub env_prefix: Option<String>,
    /// 宏行为版本（内部语义版本，随库版本演进）
    pub behavior_version: u32,
    /// 配置文件解析深度限制（由内部默认策略与 QuantumConfigFileProvider 控制）
    pub max_parse_depth: u32,
}

impl Default for QuantumConfigAppMeta {
    fn default() -> Self {
        Self {
            app_name: "app".to_string(),
            env_prefix: None,
            behavior_version: 1,
            max_parse_depth: 128,
        }
    }
}

/// Clap 属性元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClapAttrsMeta {
    /// 长选项名
    pub long: Option<String>,
    /// 短选项字符
    pub short: Option<char>,
    /// 帮助信息
    pub help: Option<String>,
    /// 动作代码（作为字符串存储）
    pub action_code: Option<String>,
    /// 值解析器代码（作为字符串存储）
    pub value_parser_code: Option<String>,
}

/// 字段级别的元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMeta {
    /// 字段在 Rust 结构体中的原始名称
    pub rust_name: &'static str,
    /// 来自 #[config(name_config = "...")] 的值
    pub config_name_override: Option<&'static str>,
    /// 来自 #[config(name_env = "...")] 的值
    pub env_name_override: Option<&'static str>,
    /// 来自 #[config(name_clap_long = "...")] 的值
    pub clap_long_override: Option<&'static str>,
    /// 来自 #[config(name_clap_short = '...')] 的值
    pub clap_short_override: Option<char>,
    /// 来自 #[config(description = "...")] 的描述
    pub description: Option<&'static str>,
    /// 指向宏生成的默认值函数的完整路径字符串
    pub default_fn_path_str: Option<&'static str>,
    /// 字段类型的字符串表示
    pub type_name_str: &'static str,
    /// 标记该字段是否是 Option<T> 类型
    pub is_option: bool,
    /// 标记该字段是否有 #[config(flatten)]
    pub is_flatten: bool,
    /// 标记该字段是否有 #[config(skip)]
    pub is_skipped: bool,
    /// 结构化表示来自 #[config(clap(...))] 的原生 clap 属性
    pub clap_direct_attrs_meta: Option<ClapAttrsMeta>,
}

impl FieldMeta {
    /// 创建一个新的字段元数据实例
    pub fn new(rust_name: &'static str, type_name_str: &'static str) -> Self {
        Self {
            rust_name,
            config_name_override: None,
            env_name_override: None,
            clap_long_override: None,
            clap_short_override: None,
            description: None,
            default_fn_path_str: None,
            type_name_str,
            is_option: false,
            is_flatten: false,
            is_skipped: false,
            clap_direct_attrs_meta: None,
        }
    }

    /// 获取配置文件中使用的键名
    pub fn config_key_name(&self) -> &str {
        self.config_name_override.unwrap_or(self.rust_name)
    }

    /// 获取环境变量名（不包含前缀）
    pub fn env_var_name(&self) -> &str {
        self.env_name_override.unwrap_or(self.rust_name)
    }

    /// 获取 clap 长选项名
    pub fn clap_long_name(&self) -> &str {
        self.clap_long_override.unwrap_or(self.rust_name)
    }
}

/// 结构体级别的元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructMeta {
    /// 结构体的 Rust 名称
    pub struct_name: &'static str,
    /// 该结构体包含的所有字段的元数据列表
    pub fields: Vec<FieldMeta>,
    /// 标记这是否是用户直接派生 Config 的顶层结构体
    pub is_top_level_config: bool,
    /// 用于存储此结构体中类型为其他配置结构体的字段的元数据
    /// 键是字段的 rust_name，值是对应嵌套结构体的 StructMeta 引用
    pub nested_struct_meta_map: HashMap<&'static str, &'static StructMeta>,
}

impl StructMeta {
    /// 创建一个新的结构体元数据实例
    pub fn new(struct_name: &'static str, is_top_level_config: bool) -> Self {
        Self {
            struct_name,
            fields: Vec::new(),
            is_top_level_config,
            nested_struct_meta_map: HashMap::new(),
        }
    }

    /// 添加字段元数据
    pub fn add_field(&mut self, field: FieldMeta) {
        self.fields.push(field);
    }

    /// 添加嵌套结构体元数据
    pub fn add_nested_struct(&mut self, field_name: &'static str, nested_meta: &'static StructMeta) {
        self.nested_struct_meta_map.insert(field_name, nested_meta);
    }

    /// 根据字段名查找字段元数据
    pub fn find_field(&self, field_name: &str) -> Option<&FieldMeta> {
        self.fields.iter().find(|f| f.rust_name == field_name)
    }

    /// 获取所有非跳过的字段
    pub fn non_skipped_fields(&self) -> impl Iterator<Item=&FieldMeta> {
        self.fields.iter().filter(|f| !f.is_skipped)
    }

    /// 获取所有扁平化的字段
    pub fn flattened_fields(&self) -> impl Iterator<Item=&FieldMeta> {
        self.fields.iter().filter(|f| f.is_flatten)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_config_app_meta_default() {
        let meta = QuantumConfigAppMeta::default();
        assert_eq!(meta.app_name, "app");
        assert_eq!(meta.env_prefix, None);
        assert_eq!(meta.behavior_version, 1);
        assert_eq!(meta.max_parse_depth, 128);
    }

    #[test]
    fn test_quantum_config_app_meta_custom() {
        let meta = QuantumConfigAppMeta {
            app_name: "myapp".to_string(),
            env_prefix: Some("MYAPP".to_string()),
            behavior_version: 2,
            max_parse_depth: 256,
        };
        assert_eq!(meta.app_name, "myapp");
        assert_eq!(meta.env_prefix, Some("MYAPP".to_string()));
        assert_eq!(meta.behavior_version, 2);
        assert_eq!(meta.max_parse_depth, 256);
    }

    #[test]
    fn test_clap_attrs_meta() {
        let attrs = ClapAttrsMeta {
            long: Some("verbose".to_string()),
            short: Some('v'),
            help: Some("Enable verbose output".to_string()),
            action_code: Some("clap::ArgAction::SetTrue".to_string()),
            value_parser_code: None,
        };
        assert_eq!(attrs.long, Some("verbose".to_string()));
        assert_eq!(attrs.short, Some('v'));
        assert_eq!(attrs.help, Some("Enable verbose output".to_string()));
    }

    #[test]
    fn test_field_meta_new() {
        let field = FieldMeta::new("host", "String");
        assert_eq!(field.rust_name, "host");
        assert_eq!(field.type_name_str, "String");
        assert_eq!(field.config_name_override, None);
        assert!(!field.is_option);
        assert!(!field.is_flatten);
        assert!(!field.is_skipped);
    }

    #[test]
    fn test_field_meta_config_key_name() {
        let mut field = FieldMeta::new("host_name", "String");
        assert_eq!(field.config_key_name(), "host_name");

        field.config_name_override = Some("hostname");
        assert_eq!(field.config_key_name(), "hostname");
    }

    #[test]
    fn test_field_meta_env_var_name() {
        let mut field = FieldMeta::new("port", "u16");
        assert_eq!(field.env_var_name(), "port");

        field.env_name_override = Some("SERVER_PORT");
        assert_eq!(field.env_var_name(), "SERVER_PORT");
    }

    #[test]
    fn test_field_meta_clap_long_name() {
        let mut field = FieldMeta::new("debug_mode", "bool");
        assert_eq!(field.clap_long_name(), "debug_mode");

        field.clap_long_override = Some("debug");
        assert_eq!(field.clap_long_name(), "debug");
    }

    #[test]
    fn test_struct_meta_new() {
        let meta = StructMeta::new("AppConfig", true);
        assert_eq!(meta.struct_name, "AppConfig");
        assert!(meta.is_top_level_config);
        assert!(meta.fields.is_empty());
        assert!(meta.nested_struct_meta_map.is_empty());
    }

    #[test]
    fn test_struct_meta_add_field() {
        let mut meta = StructMeta::new("Config", true);
        let field = FieldMeta::new("host", "String");
        meta.add_field(field);

        assert_eq!(meta.fields.len(), 1);
        assert_eq!(meta.fields[0].rust_name, "host");
    }

    #[test]
    fn test_struct_meta_find_field() {
        let mut meta = StructMeta::new("Config", true);
        let field1 = FieldMeta::new("host", "String");
        let field2 = FieldMeta::new("port", "u16");
        meta.add_field(field1);
        meta.add_field(field2);

        let found = meta.find_field("host");
        assert!(found.is_some());
        assert_eq!(found.unwrap().rust_name, "host");

        let not_found = meta.find_field("unknown");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_struct_meta_non_skipped_fields() {
        let mut meta = StructMeta::new("Config", true);

        let field1 = FieldMeta::new("host", "String");
        let field2 = FieldMeta::new("port", "u16");
        let mut field3 = FieldMeta::new("internal", "String");
        field3.is_skipped = true;

        meta.add_field(field1);
        meta.add_field(field2);
        meta.add_field(field3);

        let non_skipped: Vec<_> = meta.non_skipped_fields().collect();
        assert_eq!(non_skipped.len(), 2);
        assert_eq!(non_skipped[0].rust_name, "host");
        assert_eq!(non_skipped[1].rust_name, "port");
    }

    #[test]
    fn test_struct_meta_flattened_fields() {
        let mut meta = StructMeta::new("Config", true);

        let mut field1 = FieldMeta::new("server", "ServerConfig");
        field1.is_flatten = true;
        let field2 = FieldMeta::new("port", "u16");

        meta.add_field(field1);
        meta.add_field(field2);

        let flattened: Vec<_> = meta.flattened_fields().collect();
        assert_eq!(flattened.len(), 1);
        assert_eq!(flattened[0].rust_name, "server");
    }

    #[test]
    fn test_struct_meta_add_nested_struct() {
        let mut parent_meta = StructMeta::new("AppConfig", true);
        let nested_meta = StructMeta::new("ServerConfig", false);

        // 在实际使用中，这里会是静态引用
        // 为了测试，我们使用 Box::leak 来创建静态引用
        let static_nested_meta: &'static StructMeta = Box::leak(Box::new(nested_meta));

        parent_meta.add_nested_struct("server", static_nested_meta);

        assert_eq!(parent_meta.nested_struct_meta_map.len(), 1);
        assert!(parent_meta.nested_struct_meta_map.contains_key("server"));

        let retrieved = parent_meta.nested_struct_meta_map.get("server").unwrap();
        assert_eq!(retrieved.struct_name, "ServerConfig");
        assert!(!retrieved.is_top_level_config);
    }

    #[test]
    fn test_field_meta_with_all_options() {
        let clap_attrs = ClapAttrsMeta {
            long: Some("host-name".to_string()),
            short: Some('h'),
            help: Some("Server hostname".to_string()),
            action_code: None,
            value_parser_code: None,
        };

        let field = FieldMeta {
            rust_name: "host",
            config_name_override: Some("hostname"),
            env_name_override: Some("SERVER_HOST"),
            clap_long_override: Some("host-name"),
            clap_short_override: Some('h'),
            description: Some("The server hostname"),
            default_fn_path_str: Some("crate::defaults::default_host"),
            type_name_str: "String",
            is_option: true,
            is_flatten: false,
            is_skipped: false,
            clap_direct_attrs_meta: Some(clap_attrs),
        };

        assert_eq!(field.config_key_name(), "hostname");
        assert_eq!(field.env_var_name(), "SERVER_HOST");
        assert_eq!(field.clap_long_name(), "host-name");
        assert_eq!(field.description, Some("The server hostname"));
        assert!(field.is_option);
        assert!(field.clap_direct_attrs_meta.is_some());
    }
}

// Backward compatibility alias