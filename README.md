## crates-mcp

[![CI](https://github.com/terror/crates-mcp/actions/workflows/ci.yaml/badge.svg)](https://github.com/terror/crates-mcp/actions/workflows/ci.yaml)

**crates-mcp** is a model context protocol server implementation for retrieving
information about Rust crates.

## Installation

```
git clone https://github.com/terror/crates-mcp
cd crates-mcp
cargo build --release
claude mcp add crates /absolute/path/to/crates-mcp/binary
```
