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
local arg_index = cli_index + 2
local internal_capture_enabled = os.getenv("IME_SKIP_INTERNAL_CAPTURE") == nil
local surface_mode = (params and params.surface_mode) or os.getenv("IME_SURFACE_MODE") or "plain_only"

if cli_args[arg_index] == "skip_capture" then
    internal_capture_enabled = false
    arg_index = arg_index + 1
end

local surface_mode_arg = cli_args[arg_index]
if surface_mode_arg == "embedded" or surface_mode_arg == "plain_only" then
    surface_mode = surface_mode_arg
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

ime.set_scenario("plain_input_lab")
ime.log("starting plain input lab scenarios")

local reset_x = 0.15
local reset_y = 0.235
local first_x = 0.27
local first_y = 0.43
local source_x = 0.54
local source_y = 0.43
local target_x = 0.83
local target_y = 0.43

if surface_mode == "embedded" then
    reset_x, reset_y = 0.19, 0.21
    first_x, first_y = 0.25, 0.29
    source_x, source_y = 0.56, 0.29
    target_x, target_y = 0.86, 0.29
end

reset_x, reset_y = ime.relative("plain_reset", reset_x, reset_y)
first_x, first_y = ime.relative("plain_first", first_x, first_y)
source_x, source_y = ime.relative("plain_source", source_x, source_y)
target_x, target_y = ime.relative("plain_target", target_x, target_y)

local function bring_plain_lab_into_view()
    if surface_mode ~= "embedded" then
        return
    end

    ime.normalize_scroll_position()
    ime.scroll_steps(-30, 18, 0.05)
    ime.sleep(0.20)
end

local function reset_all_fields()
    ime.activate_window()
    bring_plain_lab_into_view()
    ime.sleep(0.12)
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
    reset_all_fields()
    ime.log(string.format("case_a focus first at %.2f,%.2f", first_x, first_y))
    ime.click_rel(first_x, first_y)
    ime.wait_for_state_log_pattern('showcase::plain_input_lab_first focused=true value=""', 2.0)
    ime.key_strokes("dks")
    ime.wait_for_state_log_pattern('stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("안") commit_pending=false', 2.0)
    maybe_capture("case_a_first_hangul")
end

local function run_case_b()
    reset_all_fields()
    ime.log(string.format("case_b focus source at %.2f,%.2f", source_x, source_y))
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern('showcase::plain_input_lab_source focused=true value=""', 2.0)
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern('preedit=Some("앞")', 2.0)
    ime.log(string.format("case_b focus target at %.2f,%.2f", target_x, target_y))
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern('showcase::plain_input_lab_target focused=true value=""', 2.0)
    maybe_capture("case_b_click_handoff")
end

local function run_case_c()
    reset_all_fields()
    ime.log(string.format("case_c focus source at %.2f,%.2f", source_x, source_y))
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern('showcase::plain_input_lab_source focused=true value=""', 2.0)
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern('preedit=Some("앞")', 2.0)
    ime.key("tab")
    ime.wait_for_state_log_pattern('showcase::plain_input_lab_target focused=true value=""', 2.0)
    maybe_capture("case_c_tab_handoff")
end

local function run_case_d()
    reset_all_fields()
    ime.log(string.format("case_d focus source at %.2f,%.2f", source_x, source_y))
    local mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_source focused=true value=""', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dkv")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("앞") commit_pending=false',
        2.0
    )
    ime.log(string.format("case_d focus target at %.2f,%.2f", target_x, target_y))
    mark = ime.state_log_mark()
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_target focused=true value=""', 2.0)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_source focused=false value="앞"', 2.0)
    ime.log(string.format("case_d focus source again at %.2f,%.2f", source_x, source_y))
    mark = ime.state_log_mark()
    ime.click_rel(source_x, source_y)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_source focused=true value="앞"', 2.0)
    mark = ime.state_log_mark()
    ime.key_strokes("dks")
    ime.wait_for_state_log_pattern_after(
        mark,
        'stage=ime_preedit:accept focused=true cursor=3 buffer="앞" preedit=Some("안") commit_pending=false',
        2.0
    )
    ime.log(string.format("case_d final focus target at %.2f,%.2f", target_x, target_y))
    mark = ime.state_log_mark()
    ime.click_rel(target_x, target_y)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_source focused=false value="앞안"', 2.0)
    ime.wait_for_state_log_pattern_after(mark, 'showcase::plain_input_lab_target focused=true value=""', 2.0)
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

ime.log("finished plain input lab scenarios")
