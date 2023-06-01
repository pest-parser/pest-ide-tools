local M = {}

---Setup pest.nvim
function M.setup(config)
    vim.filetype.add({
        extension = {
            pest = 'pest'
        },
    })
end

return M
