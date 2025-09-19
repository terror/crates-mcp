## crates-mcp

[![CI](https://github.com/terror/crates-mcp/actions/workflows/ci.yaml/badge.svg)](https://github.com/terror/crates-mcp/actions/workflows/ci.yaml)

**crates-mcp** is a model context protocol server implementation for retrieving
information about Rust crates.

It exposes three tools, namely `list_crates`, `lookup_crate` and
`generate_docs`.

### `generate_docs`

This tool allows clients to run `cargo doc <options>` inside the current
directory. The `cargo doc` command is what populates `target/doc` with a
hierarchy of HTML files, which is what the tools below use to get information
from.

### `list_crates`

This tool simply looks at what's in `target/doc` and outputs what crates are
available. If you generated documentation with `cargo doc`, this means it will
output the name of the crate you're building, in addition to the names of all
of its dependencies.

### `lookup_crate`

This tool allows clients to easily find information about a crate. Clients
specify a crate name and it gives them, by default, a JSON string with every
documentation item for that crate (e.g. functions, structs, macros, etc.).

Clients can specify options for this tool to filter its output, such as:

- `name`: The name of the Rust crate (required)
- `query`: Search term to filter items by name or description
- `item_type`: Filter by item type (function, struct, enum, trait, macro, type,
  constant, module)
- `limit`: Maximum number of items to return
- `offset`: Number of items to skip for pagination

## Installation

For now, you can clone the repository, build from source, and then use the
binary as input to clients:

```
git clone https://github.com/terror/crates-mcp
cd crates-mcp
cargo build --release
claude mcp add crates /absolute/path/to/crates-mcp/binary
```

## Prior Art

There are seemingly a
[bunch of similar projects](https://github.com/search?q=crates%20mcp&type=repositories)
out there, however I noticed most of them make HTTP requests to
[docs.rs](https://docs.rs/), which doesn't seem necessary for my use case. That
is, when I'm building a crate using a tool like claude code I only care about
the crate I'm building and what it depends on, not whatever you can find on
[docs.rs](https://docs.rs/). Generating documentation locally and then providing
tools for interacting with that generated documentation is sufficient.
