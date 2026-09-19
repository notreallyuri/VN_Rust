local M = {}

local function previous_code_line(lnum)
  local prev = vim.fn.prevnonblank(lnum - 1)
  while prev > 0 and vim.fn.getline(prev):match("^%s*#") do
    prev = vim.fn.prevnonblank(prev - 1)
  end
  return prev
end

function M.indent()
  local lnum = vim.v.lnum
  local prev = previous_code_line(lnum)
  if prev == 0 then
    return 0
  end

  local width = vim.fn.shiftwidth()
  local indent = vim.fn.indent(prev)
  if vim.fn.getline(prev):match(":%s*$") then
    indent = indent + width
  end
  if vim.fn.getline(lnum):match("^%s*else%f[^%w_]") then
    indent = indent - width
  end
  return math.max(indent, 0)
end

return M
