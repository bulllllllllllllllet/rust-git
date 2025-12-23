# Rusty Git

A simple Git implementation in Rust.

## Features

- `init`: Initialize a new repository.
- `add <path>`: Add files to staging area (supports directories).
- `commit -m <msg>`: Commit staged changes.
- `rm <path>`: Remove file from index and working directory.
- `branch <name>`: Create a new branch.
- `checkout <name>`: Switch to a branch.
- `merge <branch>`: Merge a branch (Fast-forward only).

## Usage

```bash
# Build
cargo build --release

# Run
./target/release/rust-git init
./target/release/rust-git add .
./target/release/rust-git commit -m "Initial commit"
./target/release/rust-git branch dev
./target/release/rust-git checkout dev
# ... make changes ...
./target/release/rust-git add .
./target/release/rust-git commit -m "Changes on dev"
./target/release/rust-git checkout master
./target/release/rust-git merge dev
```

## Design

- **Objects**: stored in `.git/objects` as JSON (for readability).
  - `Blob`: File content.
  - `Tree`: Directory structure.
  - `Commit`: Snapshot info.
- **Refs**: stored in `.git/refs/heads`.
- **Index**: stored in `.git/index` as JSON.

## Limitations

- `merge` only supports fast-forward.
- Binary files are skipped or might cause issues (assumes text).
- No diff/status command implemented yet.
