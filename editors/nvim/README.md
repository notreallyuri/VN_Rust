# Neovim support for `.story` files

Filetype detection, tree-sitter highlighting and folding, auto-indent and the `vn lsp` language server for Neovim 0.11+.

## Requirements

- [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) on the `main` branch, plus the `tree-sitter` CLI to compile the parser.
- The `vn` binary on `$PATH`: `cargo install --path crates/vn_cli`.

## Setup

With lazy.nvim:

```lua
{ dir = "~/projects/VN_Rust/editors/nvim", name = "vn-story", lazy = false }
```

Then enable the language server and install the parser once:

```lua
vim.lsp.enable("vn")
```

```vim
:TSInstall story
```

Run `:TSUpdate story` after changing `grammar.js` or `scanner.c`, and `tree-sitter generate` in `editors/tree-sitter-story` before that.

## What each file does

| File | Purpose |
| --- | --- |
| `plugin/story.lua` | Maps `*.story` to the `story` filetype and registers `../tree-sitter-story` with nvim-treesitter, linking its `queries/` directory so there is one copy of the queries. |
| `ftplugin/story.lua` | Starts tree-sitter highlighting, enables tree-sitter folds (all open), sets `# ` comments and 2-space indentation. |
| `lua/story/init.lua` | `indentexpr`: indents after a line ending in `:` and dedents `else` as it is typed. |
| `lsp/vn.lua` | Config for `vim.lsp.enable("vn")`: runs `vn lsp`, rooted at the nearest `schema.json`. |

## Highlight groups

Scene names and `jump` targets use `@label`, characters `@type`, image/track/sound ids `@constant`, stage positions and `none` `@constant.builtin`, transitions `@attribute`, variables `@variable`, `call` commands `@function.call`. Dialogue and narration text is marked `@spell`.
