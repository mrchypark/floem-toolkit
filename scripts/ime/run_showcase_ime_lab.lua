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

local cli_args = (_cli and _cli.args) or {}
local params = nil
if type(IME_PARAMS_FILE) == "string" and IME_PARAMS_FILE ~= "" then
    params = dofile(IME_PARAMS_FILE)
end
local cli_index = 1
if cli_args[1] ~= nil and type(cli_args[1]) == "string" and cli_args[1]:match("%.lua$") then
    cli_index = cli_index + 1
end
if cli_args[cli_index] == "--" then
    cli_index = cli_index + 1
end

local root = cli_args[cli_index]
    or (params and params.repo_root)
    or os.getenv("IME_REPO_ROOT")
    or default_repo_root()
local ime = dofile(root .. "/scripts/ime/common.lua")
if params and params.scenario_log_file then
    ime.set_scenario_log_file(params.scenario_log_file)
end
if params and params.state_log_file then
    ime.set_state_log_file(params.state_log_file)
end
local scenario = cli_args[cli_index + 1]
    or (params and params.case)
    or os.getenv("IME_CASE")
    or "all"
local surface_mode = cli_args[cli_index + 2]
    or (params and params.surface_mode)
    or os.getenv("IME_SURFACE_MODE")
    or "embedded"
local arg_index = cli_index + 3
local internal_capture_enabled = os.getenv("IME_SKIP_INTERNAL_CAPTURE") == nil

if surface_mode == "skip_capture" then
    surface_mode = "embedded"
    internal_capture_enabled = false
elseif cli_args[arg_index] == "skip_capture" then
    internal_capture_enabled = false
    arg_index = arg_index + 1
end

local window_id_arg = cli_args[arg_index]
if window_id_arg == nil or window_id_arg == "" then
    local env_window_id = (params and params.window_id) or os.getenv("IME_WINDOW_ID")
    if env_window_id ~= nil and env_window_id ~= "" then
        window_id_arg = "window:" .. env_window_id
    end
end
if type(window_id_arg) == "string" and window_id_arg:match("^window:") then
    ime.set_window_id(window_id_arg:gsub("^window:", ""))
    arg_index = arg_index + 1
end

local frame_x = tonumber(cli_args[arg_index])
local frame_y = tonumber(cli_args[arg_index + 1])
local frame_w = tonumber(cli_args[arg_index + 2])
local frame_h = tonumber(cli_args[arg_index + 3])

if not (frame_x and frame_y and frame_w and frame_h) then
    frame_x = tonumber((params and params.window_x) or os.getenv("IME_WINDOW_X"))
    frame_y = tonumber((params and params.window_y) or os.getenv("IME_WINDOW_Y"))
    frame_w = tonumber((params and params.window_w) or os.getenv("IME_WINDOW_W"))
    frame_h = tonumber((params and params.window_h) or os.getenv("IME_WINDOW_H"))
end

if frame_x and frame_y and frame_w and frame_h then
    ime.set_window_frame({
        x = frame_x,
        y = frame_y,
        w = frame_w,
        h = frame_h,
    })
end

ime.set_scenario("showcase_ime_lab")
ime.log("starting showcase IME lab scenarios")
ime.log(string.format("surface_mode=%s", surface_mode))

local first_defaults = { x = 0.22, y = 0.80 }
local source_defaults = { x = 0.52, y = 0.80 }
local target_defaults = { x = 0.82, y = 0.80 }
local reset_defaults = { x = 0.14, y = 0.71 }

if surface_mode == "ime_only" then
    first_defaults = { x = 0.24, y = 0.375 }
    source_defaults = { x = 0.49, y = 0.375 }
    target_defaults = { x = 0.76, y = 0.375 }
    reset_defaults = { x = 0.10, y = 0.19 }
end

local first_x, first_y = ime.relative("showcase_first", first_defaults.x, first_defaults.y)
local source_x, source_y = ime.relative("showcase_source", source_defaults.x, source_defaults.y)
local target_x, target_y = ime.relative("showcase_target", target_defaults.x, target_defaults.y)
local reset_x, reset_y = ime.relative("showcase_reset", reset_defaults.x, reset_defaults.y)

local function bring_ime_lab_into_view()
    if surface_mode == "ime_only" then
        ime.activate_window()
        return
    end
    ime.normalize_scroll_position()
    ime.scroll_steps(-30, 18, 0.05)
    ime.sleep(0.20)
end

local function reset_all_fields()
    bring_ime_lab_into_view()
    ime.log(string.format("reset click at %.2f,%.2f", reset_x, reset_y))
    ime.click_rel(reset_x, reset_y)
    ime.sleep(0.12)
    ime.log(string.format("clear first at %.2f,%.2f", first_x, first_y))
    ime.clear_field(first_x, first_y)
    ime.log(string.format("clear source at %.2f,%.2f", source_x, source_y))
    ime.clear_field(source_x, source_y)
    ime.log(string.format("clear target at %.2f,%.2f", target_x, target_y))
    ime.clear_field(target_x, target_y)
end

local function maybe_capture(name)
    if internal_capture_enabled then
        ime.capture(name)
    end
end

local function run_case_a()
    bring_ime_lab_into_view()
    maybe_capture("checkpoint_visible")
    reset_all_fields()
    ime.log(string.format("case_a click first at %.2f,%.2f", first_x, first_y))
    local mark = ime.state_log_mark()
    ime.click_rel(first_x, first_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-first focused=true value=""', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dks")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("안") commit_pending=false',
        2.0
    )
    maybe_capture("case_a_first_hangul")
end

local function run_case_b()
    bring_ime_lab_into_view()
    reset_all_fields()
    ime.log(string.format("case_b click source at %.2f,%.2f", source_x, source_y))
    local mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=true value=""', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("앞") commit_pending=false',
        2.0
    )
    ime.log(string.format("case_b click target at %.2f,%.2f", target_x, target_y))
    mark = ime.state_log_mark()
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-target focused=true value=""', 2.0)
    maybe_capture("case_b_click_handoff")
end

local function run_case_c()
    bring_ime_lab_into_view()
    reset_all_fields()
    ime.log(string.format("case_c click source at %.2f,%.2f", source_x, source_y))
    local mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=true value=""', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("앞") commit_pending=false',
        2.0
    )
    mark = ime.state_log_mark()
    ime.key("tab")
    ime.wait_for_state_log_pattern_after(mark, 'ime-target focused=true value=""', 2.0)
    maybe_capture("case_c_tab_handoff")
end

local function run_case_d()
    bring_ime_lab_into_view()
    reset_all_fields()
    ime.log(string.format("case_d click source at %.2f,%.2f", source_x, source_y))
    local mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=true value=""', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("앞") commit_pending=false',
        2.0
    )
    ime.log(string.format("case_d click target at %.2f,%.2f", target_x, target_y))
    mark = ime.state_log_mark()
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-target focused=true value=""', 2.0)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=false value="앞"', 2.0)
    ime.log(string.format("case_d click source again at %.2f,%.2f", source_x, source_y))
    mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=true value="앞"', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dks")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=3 buffer="앞" preedit=Some("안") commit_pending=false',
        2.0
    )
    ime.log(string.format("case_d final click target at %.2f,%.2f", target_x, target_y))
    mark = ime.state_log_mark()
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern_after(mark, 'ime-source focused=false value="앞안"', 2.0)
    ime.wait_for_state_log_pattern_after(mark, 'ime-target focused=true value=""', 2.0)
    maybe_capture("case_d_reentry_handoff")
end

if scenario == "case_a" then
    run_case_a()
elseif scenario == "case_b" then
    run_case_b()
elseif scenario == "case_c" then
    run_case_c()
elseif scenario == "case_d" then
    run_case_d()
else
    run_case_a()
    run_case_b()
    run_case_c()
    run_case_d()
end

ime.log("finished showcase IME lab scenarios")
