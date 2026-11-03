# Bitcask 风格键值存储引擎

一个使用 Rust 实现的轻量级持久化键值存储引擎。它实现了 Bitcask 的核心思路：在磁盘上维护仅追加的日志，并在内存中保存指向最新数据的索引。

这是一个用于学习和作品展示的实现，并非生产级数据库。

## 功能特性

- 支持持久化的 `set`、`get` 和 `rm` 操作
- 使用仅追加的 JSON 命令日志保存数据
- 使用内存 `HashMap` 保存键到日志偏移量的索引
- 启动时扫描日志并重建内存索引，实现数据恢复
- 当过期日志达到阈值时自动压缩日志
- 基于 `clap` 的命令行工具
- 覆盖命令行行为、持久化、覆盖写、删除和日志压缩的集成测试

## 工作原理

```text
set key value  ──> 将 {"Set": ...} 追加写入 kvs.log
                        │
                        └──> index[key] =（文件偏移量，记录长度）

get key        ──> 在内存中查找偏移量 ──> 只读取对应的一条记录
```

程序重启时，存储引擎会从头到尾扫描 `kvs.log`，重建内存索引。当旧记录累积到一定大小后，压缩过程会将每个键的最新值写入新日志，从而回收无效记录占用的空间。

## 快速开始

环境要求：已安装 Rust 和 Cargo（Rust 2018 或更高版本）。

```bash
git clone https://github.com/mianyangli/bitcask.git
cd bitcask/kvs

cargo run -- set language rust
cargo run -- get language
# rust

cargo run -- rm language
cargo run -- get language
# Key not found
```

命令行工具会在当前工作目录中创建并使用 `kvs.log` 保存数据。

## 命令说明

```text
kvs set <KEY> <VALUE>   写入键值对
kvs get <KEY>           读取键对应的值
kvs rm <KEY>            删除一个键
kvs --version           查看版本号
```

## 测试

```bash
cargo test
```

测试覆盖了命令行参数校验、持久化恢复、覆盖写入、不存在的键、删除操作和日志压缩。

## 项目结构

```text
src/lib.rs       存储引擎和公开的 `KvStore` API
src/bin/kv.rs    命令行入口
tests/tests.rs   集成测试
```

## 当前限制与后续方向

- 仅支持单进程本地磁盘访问，未实现网络协议和并发控制。
- 全量键索引保存在内存中，需要保证索引能够放入内存。
- 为便于阅读，日志采用 JSON，而非更紧凑的二进制格式。
- 暂未实现崩溃一致性校验、校验和、文件锁、分段日志和后台压缩。

## 许可证

本仓库仅用于学习与作品展示。
