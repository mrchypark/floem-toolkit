local M = {}

local key_codes = {
    a = 0,
    s = 1,
    d = 2,
    v = 9,
    k = 40,
    delete = 51,
    tab = 48,
    space = 49,
    pagedown = 121,
}

local function current_script_dir()
    local source = debug.getinfo(1, "S").source
    if type(source) == "string" and source:sub(1, 1) == "@" then
        source = source:sub(2)
        return source:match("^(.*)/[^/]+$")
    end
    return nil
end

local function default_repo_root()
    local script_dir = current_script_dir()
    if script_dir ~= nil then
        return script_dir:gsub("/scripts/ime$", "")
    end
    return hs.fs.currentDir()
end

local function env(key, default)
    local value = os.getenv(key)
    if value == nil or value == "" then
        return default
    end
    return value
end

local function parse_number(value, default)
    local n = tonumber(value)
    if n == nil then
        return default
    end
    return n
end

local function default_window_size()
    return {
        w = parse_number(os.getenv("IME_WINDOW_W"), 800),
        h = parse_number(os.getenv("IME_WINDOW_H"), 632),
    }
end

M.repo_root = env("IME_REPO_ROOT", default_repo_root())
M.capture_script = M.repo_root .. "/scripts/ime/capture.sh"
M.artifacts_root = env("IME_ARTIFACTS_ROOT", M.repo_root .. "/artifacts/ime")
M.scenario = env("IME_SCENARIO", "ime")
M.scenario_log_file = env("IME_SCENARIO_LOG_FILE", "")
M.state_log_file = env("IME_STATE_LOG_FILE", "")
M.window_override = nil
M.window_id_override = "0"

function M.set_scenario(name)
    M.scenario = name
end

function M.set_scenario_log_file(path)
    if path ~= nil then
        M.scenario_log_file = tostring(path)
    end
end

function M.set_state_log_file(path)
    if path ~= nil then
        M.state_log_file = tostring(path)
    end
end

function M.log(message)
    local line = string.format("[ime][%s] %s", M.scenario, message)
    if M.scenario_log_file ~= nil and M.scenario_log_file ~= "" then
        local file = io.open(M.scenario_log_file, "a")
        if file ~= nil then
            file:write(line)
            file:write("\n")
            file:close()
        end
    end
    if os.getenv("IME_LOG_TO_STDOUT") == "1" then
        print(line)
    end
end

local function read_file(path)
    if path == nil or path == "" then
        return nil
    end
    local file = io.open(path, "r")
    if file == nil then
        return nil
    end
    local content = file:read("*a")
    file:close()
    return content
end

function M.wait_for_state_log_pattern(pattern, timeout_seconds)
    return M.wait_for_state_log_pattern_after(0, pattern, timeout_seconds)
end

function M.state_log_mark()
    local content = read_file(M.state_log_file)
    if content == nil then
        return 0
    end
    return #content
end

function M.wait_for_state_log_pattern_after(offset, pattern, timeout_seconds)
    local timeout = timeout_seconds or 2.0
    local deadline = hs.timer.secondsSinceEpoch() + timeout
    local start_offset = offset or 0
    while hs.timer.secondsSinceEpoch() <= deadline do
        local content = read_file(M.state_log_file)
        if content ~= nil then
            local haystack = content
            if start_offset > 0 and start_offset < #content then
                haystack = content:sub(start_offset + 1)
            elseif start_offset >= #content then
                haystack = ""
            end
            if haystack:find(pattern, 1, true) ~= nil then
                M.log(string.format("state_log matched pattern=%s after=%d", pattern, start_offset))
                return true
            end
        end
        M.sleep(0.05)
    end
    error(string.format("timed out waiting for state log pattern after %d: %s", start_offset, pattern))
end

local function frontmost_app_name()
    local app = hs.application.frontmostApplication()
    if app == nil then
        return "nil"
    end
    return app:name() or "unknown"
end

local function assert_frontmost_showcase(context)
    local name = frontmost_app_name()
    if name ~= "floem-showcase" then
        error(string.format("%s requires floem-showcase frontmost, got %s", context, name))
    end
    return name
end

local function valid_frame(frame)
    return frame ~= nil and frame.w ~= nil and frame.h ~= nil and frame.w > 0 and frame.h > 0
end

local function centered_screen_frame()
    local screen = hs.screen.primaryScreen() or hs.screen.mainScreen()
    if screen == nil then
        return nil
    end
    local screen_frame = screen:frame()
    local size = default_window_size()
    return {
        x = math.floor(screen_frame.x + (screen_frame.w - size.w) / 2),
        y = math.floor(screen_frame.y + (screen_frame.h - size.h) / 2),
        w = size.w,
        h = size.h,
    }
end

function M.set_window_frame(frame)
    if valid_frame(frame) then
        M.window_override = frame
    end
end

function M.set_window_id(window_id)
    if window_id ~= nil and tostring(window_id) ~= "" then
        M.window_id_override = tostring(window_id)
    end
end

function M.window_frame()
    if valid_frame(M.window_override) then
        return M.window_override
    end
    local fallback = centered_screen_frame()
    if fallback ~= nil then
        return fallback
    end
    error("could not determine floem-showcase window bounds")
end

function M.rel_point(rel_x, rel_y)
    local frame = M.window_frame()
    return {
        x = math.floor(frame.x + frame.w * rel_x),
        y = math.floor(frame.y + frame.h * rel_y),
    }
end

function M.relative(name, default_x, default_y)
    local upper = string.upper(name)
    local x = parse_number(os.getenv("IME_" .. upper .. "_X"), default_x)
    local y = parse_number(os.getenv("IME_" .. upper .. "_Y"), default_y)
    return x, y
end

function M.sleep(seconds)
    hs.timer.usleep(math.floor(seconds * 1000000))
end

local function normalized_modifiers(modifiers)
    if modifiers == nil or #modifiers == 0 then
        return {}
    end

    local translated = {}
    for _, modifier in ipairs(modifiers) do
        local lowered = string.lower(modifier)
        local mapped = ({
            cmd = "cmd",
            command = "cmd",
            shift = "shift",
            alt = "alt",
            option = "alt",
            ctrl = "ctrl",
            control = "ctrl",
        })[lowered]
        if mapped == nil then
            error(string.format("unsupported modifier: %s", tostring(modifier)))
        end
        table.insert(translated, mapped)
    end

    return translated
end

function M.click_rel(rel_x, rel_y)
    local point = M.rel_point(rel_x, rel_y)
    hs.mouse.absolutePosition(point)
    hs.eventtap.event.newMouseEvent(hs.eventtap.event.types.leftMouseDown, point):post()
    M.sleep(0.01)
    hs.eventtap.event.newMouseEvent(hs.eventtap.event.types.leftMouseUp, point):post()
    M.sleep(0.08)
    M.log(string.format("click_rel %.2f,%.2f frontmost=%s", rel_x, rel_y, assert_frontmost_showcase("click_rel")))
    return point
end

function M.activate_window()
    M.click_rel(0.50, 0.03)
    M.sleep(0.12)
end

function M.key_strokes(text)
    for c in text:gmatch(".") do
        M.key(c)
        M.sleep(0.03)
    end
end

function M.key(key, modifiers)
    local code = key_codes[string.lower(key)]
    if code == nil then
        error(string.format("unsupported key: %s", tostring(key)))
    end
    local normalized = normalized_modifiers(modifiers or {})
    M.log(string.format("key %s frontmost=%s", tostring(key), assert_frontmost_showcase("key")))
    hs.eventtap.event.newKeyEvent(normalized, code, true):post()
    M.sleep(0.01)
    hs.eventtap.event.newKeyEvent(normalized, code, false):post()
    M.sleep(0.03)
end

function M.clear_field(rel_x, rel_y)
    M.click_rel(rel_x, rel_y)
    M.key("a", { "cmd" })
    M.key("delete")
    M.sleep(0.05)
end

function M.page_down(times)
    for _ = 1, times do
        M.key("pagedown")
        M.sleep(0.15)
    end
end

function M.scroll_steps(delta_y, times, delay_seconds)
    local delay = delay_seconds or 0.08
    for _ = 1, times do
        hs.eventtap.event.newScrollEvent({ 0, delta_y }, {}, "pixel"):post()
        M.sleep(delay)
    end
end

function M.normalize_scroll_position()
    M.activate_window()
    M.click_rel(0.50, 0.45)
    M.sleep(0.12)
    M.scroll_steps(20, 24, 0.05)
    M.sleep(0.15)
end

function M.capture(step_name)
    local frame = M.window_frame()
    local window_id = M.window_id_override or "0"
    local output_file = string.format("%s/%s/%s.png", M.artifacts_root, M.scenario, step_name)
    local command = string.format(
        "bash %q %q %q %d %d %d %d",
        M.capture_script,
        output_file,
        window_id,
        math.floor(frame.x),
        math.floor(frame.y),
        math.floor(frame.w),
        math.floor(frame.h)
    )
    local ok, reason, code = os.execute(command)
    if not ok then
        error(string.format("capture failed: %s (%s)", tostring(reason), tostring(code)))
    end
    M.log(string.format("capture: %s (window:%s)", output_file, tostring(window_id)))
    return output_file
end

return M
