local function default_repo_root()
    local source = debug.getinfo(1, "S").source
    if type(source) == "string" and source:sub(1, 1) == "@" then
        local script_dir = source:sub(2):match("^(.*)/[^/]+$")
        if script_dir ~= nil then
            return script_dir:gsub("/scripts/ime$", "")
        end
    end
    return hs.fs.currentDir()
end

local params = nil
if type(IME_PARAMS_FILE) == "string" and IME_PARAMS_FILE ~= "" then
    params = dofile(IME_PARAMS_FILE)
end

local root = (params and params.repo_root) or os.getenv("IME_REPO_ROOT") or default_repo_root()
local mode = (params and params.mode) or os.getenv("IME_SURFACE_MODE") or "normal"
local pid_file = (params and params.pid_file) or os.getenv("IME_LAUNCH_PID_FILE")
local app_log_file = (params and params.app_log_file) or os.getenv("FLOEM_SHOWCASE_IME_STATE_LOG_FILE")

local env = {}
if mode == "plain" then
    env.FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY = "1"
elseif mode == "normal" then
    env.FLOEM_SHOWCASE_DEBUG_IME_LAB_ONLY = "1"
end
env.FLOEM_IME_DEBUG = "1"

if app_log_file ~= nil and app_log_file ~= "" then
    env.FLOEM_SHOWCASE_IME_STATE_LOG_FILE = app_log_file
end

local task = hs.task.new(root .. "/target/debug/floem-showcase", function() return false end, {})
if task == nil then
    error("failed to create showcase task")
end

task:setEnvironment(env)
assert(task:start())

local pid = tostring(task:pid())
if pid_file ~= nil and pid_file ~= "" then
    local file = assert(io.open(pid_file, "w"))
    file:write(pid)
    file:close()
end

print(pid)
