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
    "ThatOneShortGuy/sf-formula-parser",
    -- This repo is a monorepo; plugin files live in `sf_formula.nvim/`.
    dir = vim.fn.stdpath("data") .. "/lazy/sf-formula-parser/sf_formula.nvim",
    name = "sf_formula.nvim",
    init = function()
      -- Register filetype early so Lazy can load on `ft = "sff"`.
      vim.filetype.add({ extension = { sff = "sff" } })
    end,
    ft = { "sff" },
    build = function(plugin)
      local workspace = vim.fs.normalize(vim.fn.fnamemodify(plugin.dir, ':h'))
      local manifest = vim.fs.joinpath(workspace, 'Cargo.toml')

      local cmd
      if vim.uv.os_uname().sysname == 'Windows_NT' then
        cmd = {
          'cmd.exe',
          '/c',
          'cargo',
          'build',
          '--release',
          '--manifest-path',
          manifest,
          '-p',
          'sf_formula_lsp',
        }
      else
        cmd = {
          'cargo',
          'build',
          '--release',
          '--manifest-path',
          manifest,
          '-p',
          'sf_formula_lsp',
        }
      end

      local result = vim.system(cmd, { text = true }):wait()
      if result.code ~= 0 then
        error((result.stderr or result.stdout or 'build failed'))
      end
    end,
  },
}
```

Then run `:Lazy sync`.

`sf_formula_lsp` only starts for `.sff` buffers (via `ftplugin/sff.lua`), not for every file.

## Verify

1. Open a `.sff` file.
2. Run `:LspInfo` and confirm `sf-formula-lsp` is attached.
