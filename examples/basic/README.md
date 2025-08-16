# 基本配置加载示例

这个示例展示了 QuantumConfig 的基本用法，包括如何定义配置结构体、加载配置以及处理不同的配置源。

## 运行示例

```bash
cd examples/basic
cargo run
```

## 测试不同的配置源

### 1. 使用默认配置

```bash
cargo run
```

### 2. 使用环境变量

```bash
export BASIC_HOST="0.0.0.0"
export BASIC_PORT="3000"
export BASIC_DEBUG="true"
export BASIC_LOG_LEVEL="debug"
export BASIC_WORKERS="8"
cargo run
```

### 3. 使用命令行参数

```bash
cargo run -- --host 0.0.0.0 --port 3000 --debug --log-level debug --workers 8
```

### 4. 使用配置文件

```bash
cargo run -- --config ./config.toml
```

### 5. 组合使用（展示优先级）

```bash
# 设置环境变量
export BASIC_HOST="env-host"
export BASIC_PORT="9000"

# 使用命令行参数覆盖部分配置
cargo run -- --host cli-host --debug
```

在这个例子中，最终的配置将是：

- `host`: "cli-host" (命令行参数，最高优先级)
- `port`: 9000 (环境变量)
- `debug`: true (命令行参数)
- 其他字段使用默认值

## 配置优先级

QuantumConfig 按以下优先级加载配置（后者覆盖前者）：

1. **默认值** - 代码中通过 `default` 属性定义
2. **系统配置文件** - `/etc/basic_example/config.toml`
3. **用户配置文件** - `~/.config/basic_example/config.toml`
4. **指定配置文件** - 通过 `--config` 参数指定
5. **环境变量** - 使用 `BASIC_` 前缀
6. **命令行参数** - 最高优先级

## 学习要点

1. **结构体定义**: 使用 `#[derive(Config)]` 和相关属性
2. **字段属性**: `#[quantum_config_opt(...)]` 的各种用法
3. **应用级属性**: `#[config(env_prefix)]`
4. **类型支持**: 字符串、数字、布尔值、Option 类型
5. **命名映射**: 自动生成环境变量名和命令行选项名