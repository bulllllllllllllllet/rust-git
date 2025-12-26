# Rusty Git

一个使用 Rust 实现的简易 Git 版本控制系统。

## 功能特性

- `init`: 初始化新仓库。
- `add <path>`: 将文件添加到暂存区（支持目录）。
- `commit -m <msg>`: 提交暂存区的更改。
- `rm <path>`: 从索引和工作目录中删除文件。
- `branch <name>`: 创建新分支。
- `checkout <name>`: 切换分支。
- `merge <branch>`: 合并分支（仅支持快进合并 Fast-forward）。
- `log`: 查看提交历史记录。

## 使用方法

```bash
# 编译项目
cargo build --release

# 运行示例
./target/release/rust-git init
./target/release/rust-git add .
./target/release/rust-git commit -m "初始提交"
./target/release/rust-git log
./target/release/rust-git branch dev
./target/release/rust-git checkout dev
# ... 修改文件 ...
./target/release/rust-git add .
./target/release/rust-git commit -m "在 dev 分支上的修改"
./target/release/rust-git checkout master
./target/release/rust-git merge dev
```

## 设计原理

- **哈希算法**: 使用 SHA-256 算法生成对象的唯一标识符。
- **对象 (Objects)**: 存储在 `.git/objects` 目录下，使用 JSON 格式序列化（为了增强可读性）。
  - `Blob`: 存储文件内容。
  - `Tree`: 存储目录结构。
  - `Commit`: 存储快照信息。
- **引用 (Refs)**: 存储在 `.git/refs/heads`。
- **索引 (Index)**: 存储在 `.git/index`，采用 JSON 格式。

## 局限性

- `merge` 仅支持快进合并 (Fast-forward)。
- 跳过二进制文件或可能引起问题（假设处理的都是文本文件）。
- 尚未实现 `diff` 或 `status` 命令。
