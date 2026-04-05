local M = {}

local function find_cmd()
	-- 1. allow user override
	if vim.g.sf_formula_lsp_cmd then
		return vim.g.sf_formula_lsp_cmd
	end

	-- 2. prefer executable on PATH
	if vim.fn.executable("sf_formula_lsp") == 1 then
		return { "sf_formula_lsp" }
	end

	-- 3. optional local dev fallback
	local plugin_dir = debug.getinfo(1, "S").source:sub(2):match("(.*/)")
	if plugin_dir then
		local candidate = vim.fs.normalize(plugin_dir .. "../../../target/release/sf_formula_lsp")
		if vim.fn.executable(candidate) == 1 then
			return { candidate }
		end

		local win_candidate = candidate .. ".exe"
		if vim.fn.executable(win_candidate) == 1 then
			return { win_candidate }
		end
	end

	return nil
end

function M.start_lsp_for_current_buffer()
	local cmd = find_cmd()
	if not cmd then
		vim.notify("sf_formula_lsp not found. Set vim.g.sf_formula_lsp_cmd or put it on PATH.", vim.log.levels.ERROR)
		return
	end

	local root = vim.fs.root(0, { ".git", "Cargo.toml" }) or vim.fn.getcwd()

	local client_id = vim.lsp.start({
		name = "sf-formula-lsp",
		cmd = cmd,
		root_dir = root,
	}, {
		reuse_client = function(client, config)
			return client.name == config.name and client.config.root_dir == config.root_dir
		end,
	})

	if not client_id then
		vim.notify("Failed to start sf_formula_lsp", vim.log.levels.ERROR)
	end
end

return M
