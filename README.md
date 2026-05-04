# sf_formula_lsp

Neovim support for Salesforce-style formula files (`.sff`) with:

- filetype detection
- syntax highlighting
- LSP startup for `.sff` buffers

## Prerequisites

- Neovim `0.10+`
- Rust toolchain (`cargo`) to build the LSP binary

## Install with Lazy.nvim

Add this plugin spec to your Lazy plugin list (for example in `lua/plugins/sf_formula.lua`):

```lua
return {
  {
    "ThatOneShortGuy/sf_formula.nvim",
    ft = { "sff" },
    build = function()
      require("sf_formula.build").ensure_lsp()
    end,
  },
}
```

The build hook above checks upstream git `HEAD` during `:Lazy sync` and installs/reinstalls `sf_formula_lsp` when missing or out of date.
It installs to `stdpath("data") .. "/sf_formula_nvim/bin"`, so the binary does not need to be on your shell `PATH`.

If you want to use a custom binary location instead, set:

```lua
vim.g.sf_formula_lsp_cmd = { "/absolute/path/to/sf_formula_lsp" }
```

The plugin only starts the LSP for `.sff` buffers (via `ftplugin/sff.lua`), not for every file.

## Verify

1. Open a `.sff` file.
2. Run `:LspInfo` and confirm `sf-formula-lsp` is attached.

## LSP capabilities

Current server features:

[x] Text document sync (`didOpen` + `didChange`, full document sync, parsing)
[x] Syntax diagnostics published on open and change
[x] Function-name completion (`textDocument/completion`)
[x] Completion docs for supported Salesforce functions

Current scope (not implemented yet):

[ ] Hover
[ ] Rename
[ ] Code actions
[ ] Formatting
[ ] Semantic tokens

## Run the parser on a file

From the repository root, run:

```sh
cargo run -p sf_formula_parser -- path/to/formula.sff
```

The parser prints `ok: expression is valid` when the file is valid. If invalid, it prints a diagnostic snippet and exits with a non-zero status.

You can also build and run the binary directly:

```sh
cargo build -p sf_formula_parser --release
./target/release/sf_formula_parser path/to/formula.sff
```
