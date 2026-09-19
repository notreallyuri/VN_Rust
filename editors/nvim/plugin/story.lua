if vim.g.loaded_story then
  return
end
vim.g.loaded_story = true

vim.filetype.add({ extension = { story = "story" } })

local plugin = vim.fs.dirname(vim.fs.dirname(vim.fs.normalize(debug.getinfo(1, "S").source:sub(2))))
local grammar = vim.fs.joinpath(vim.fs.dirname(plugin), "tree-sitter-story")

vim.api.nvim_create_autocmd("User", {
  pattern = "TSUpdate",
  callback = function()
    require("nvim-treesitter.parsers").story = {
      install_info = { path = grammar, queries = "queries" },
    }
  end,
})
