if vim.b.did_ftplugin then
  return
end
vim.b.did_ftplugin = true

vim.bo.commentstring = "# %s"
vim.bo.comments = ":#"
vim.bo.expandtab = true
vim.bo.shiftwidth = 2
vim.bo.softtabstop = 2
vim.bo.indentexpr = "v:lua.require'story'.indent()"
vim.bo.indentkeys = "!^F,o,O,=else"

if pcall(vim.treesitter.start) then
  vim.wo[0][0].foldmethod = "expr"
  vim.wo[0][0].foldexpr = "v:lua.vim.treesitter.foldexpr()"
  vim.wo[0][0].foldlevel = 99
end

vim.b.undo_ftplugin = "setl cms< com< et< sw< sts< inde< indk< | lua pcall(vim.treesitter.stop)"
