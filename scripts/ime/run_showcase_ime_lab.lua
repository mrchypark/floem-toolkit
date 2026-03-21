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
    source_defaults = { x = 0.12, y = 0.52 }
    target_defaults = { x = 0.40, y = 0.52 }
    reset_defaults = { x = 0.10, y = 0.19 }
end

local first_x, first_y = ime.relative("showcase_first", first_defaults.x, first_defaults.y)
local source_x, source_y = ime.relative("showcase_source", source_defaults.x, source_defaults.y)
local target_x, target_y = ime.relative("showcase_target", target_defaults.x, target_defaults.y)
local reset_x, reset_y = ime.relative("showcase_reset", reset_defaults.x, reset_defaults.y)

local fields = {
    first = { name = "first", label = "ime-first", x = first_x, y = first_y },
    source = { name = "source", label = "ime-source", x = source_x, y = source_y },
    target = { name = "target", label = "ime-target", x = target_x, y = target_y },
}
local field_order = { fields.first, fields.source, fields.target }

local function bring_ime_lab_into_view()
    if surface_mode == "ime_only" then
        ime.activate_window()
        return
    end
    ime.normalize_scroll_position()
    ime.scroll_steps(-30, 18, 0.05)
    ime.sleep(0.20)
end

local function focus_pattern(field, value)
    return string.format('%s focused=true value="%s"', field.label, value)
end

local function blur_pattern(field, value)
    return string.format('%s focused=false value="%s"', field.label, value)
end

local function ime_commit_pattern(field, text)
    return string.format('%s ime_commit text="%s"', field.label, text)
end

local function ime_preedit_accept_pattern(cursor, buffer, preedit)
    return string.format(
        'stage=ime_preedit:accept focused=true cursor=%d buffer="%s" preedit=Some("%s") commit_pending=false',
        cursor,
        buffer,
        preedit
    )
end

local function clear_field(field)
    ime.log(string.format("clear %s at %.2f,%.2f", field.name, field.x, field.y))
    ime.clear_field(field.x, field.y)
end

local function reset_all_fields()
    bring_ime_lab_into_view()
    ime.log(string.format("reset click at %.2f,%.2f", reset_x, reset_y))
    ime.click_rel(reset_x, reset_y)
    ime.sleep(0.12)
    for _, field in ipairs(field_order) do
        clear_field(field)
    end
    ime.sleep(0.12)
end

local function maybe_capture(name)
    if internal_capture_enabled then
        ime.capture(name)
    end
end

local function focus_field(field, value, options)
    local expected = focus_pattern(field, value)
    local attempts = (options and options.attempts) or 3
    local timeout = (options and options.timeout) or 1.0
    local retry_sleep = (options and options.retry_sleep) or 0.12
    local reason = (options and options.reason) or field.name

    for attempt = 1, attempts do
        ime.log(string.format("%s focus attempt %d at %.2f,%.2f", reason, attempt, field.x, field.y))
        local mark = ime.state_log_mark()
        ime.click_rel(field.x, field.y)
        if ime.state_log_contains_pattern_after(mark, expected, timeout) then
            return
        end
        ime.log(string.format("%s focus retry missing pattern=%s", reason, expected))
        bring_ime_lab_into_view()
        ime.sleep(retry_sleep)
    end

    error(string.format("failed to focus %s with value=%s", field.label, value))
end

local function type_and_expect_preedit(text, cursor, buffer, preedit, inter_key_delay, timeout_seconds)
    local mark = ime.state_log_mark()
    ime.key_strokes(text, inter_key_delay)
    ime.wait_for_state_log_pattern_after(
        mark,
        ime_preedit_accept_pattern(cursor, buffer, preedit),
        timeout_seconds or 2.0
    )
end

local function click_handoff(from_field, from_value, to_field, to_value, options)
    local timeout = (options and options.timeout) or 2.0
    local blur_timeout = (options and options.blur_timeout) or timeout
    local commit_text = options and options.commit_text
    local context = (options and options.context) or string.format("%s->%s", from_field.name, to_field.name)

    ime.log(string.format("%s click %s at %.2f,%.2f", context, to_field.name, to_field.x, to_field.y))
    local mark = ime.state_log_mark()
    ime.click_rel(to_field.x, to_field.y)
    ime.wait_for_state_log_pattern_after(mark, focus_pattern(to_field, to_value), timeout)
    ime.wait_for_state_log_pattern_after(mark, blur_pattern(from_field, from_value), blur_timeout)
    if commit_text ~= nil then
        ime.wait_for_state_log_pattern_after(mark, ime_commit_pattern(from_field, commit_text), blur_timeout)
    end
end

local function tab_handoff(from_field, from_value, to_field, to_value, options)
    local timeout = (options and options.timeout) or 2.0
    local context = (options and options.context) or string.format("%s->%s", from_field.name, to_field.name)

    ime.log(string.format("%s tab to %s", context, to_field.name))
    local mark = ime.state_log_mark()
    ime.key("tab")
    ime.wait_for_state_log_pattern_after(mark, focus_pattern(to_field, to_value), timeout)
    ime.wait_for_state_log_pattern_after(mark, blur_pattern(from_field, from_value), timeout)
end

local function prepare_case(options)
    bring_ime_lab_into_view()
    if options and options.capture_name ~= nil then
        maybe_capture(options.capture_name)
    end
    reset_all_fields()
end

local function run_case_a()
    prepare_case({ capture_name = "checkpoint_visible" })
    focus_field(fields.first, "", { reason = "case_a focus first" })
    type_and_expect_preedit("dks", 0, "", "안")
    maybe_capture("case_a_first_hangul")
end

local function run_case_b()
    prepare_case()
    focus_field(fields.source, "", { reason = "case_b focus source" })
    type_and_expect_preedit("dkv", 0, "", "앞")
    click_handoff(fields.source, "앞", fields.target, "", {
        context = "case_b",
    })
    maybe_capture("case_b_click_handoff")
end

local function run_case_c()
    prepare_case()
    focus_field(fields.source, "", {
        reason = "case_c focus source",
        attempts = 4,
        timeout = 1.2,
    })
    type_and_expect_preedit("dkv", 0, "", "앞")
    tab_handoff(fields.source, "앞", fields.target, "", {
        context = "case_c",
    })
    maybe_capture("case_c_tab_handoff")
end

local function run_case_d()
    prepare_case()
    focus_field(fields.source, "", { reason = "case_d focus source" })
    type_and_expect_preedit("dkv", 0, "", "앞")
    click_handoff(fields.source, "앞", fields.target, "", {
        context = "case_d initial handoff",
        blur_timeout = 10.0,
        commit_text = "앞",
    })
    focus_field(fields.source, "앞", {
        reason = "case_d refocus source",
        timeout = 2.0,
        attempts = 3,
    })
    type_and_expect_preedit("dks", 3, "앞", "안")
    click_handoff(fields.source, "앞안", fields.target, "", {
        context = "case_d final handoff",
    })
    maybe_capture("case_d_reentry_handoff")
end

local function run_case_e()
    prepare_case()
    focus_field(fields.first, "", { reason = "case_e focus first" })
    ime.log("case_e type 안녕하세요")
    local mark = ime.state_log_mark()
    ime.key_strokes("dkssudgktpdy", 0.12)
    ime.wait_for_state_log_pattern_after(mark, focus_pattern(fields.first, "안녕하세"), 2.0)
    tab_handoff(fields.first, "안녕하세요", fields.source, "", {
        context = "case_e",
    })
    maybe_capture("case_e_initial_phrase")
end

local function run_case_f()
    prepare_case()
    ime.log(string.format("case_f click first and immediately type at %.2f,%.2f", fields.first.x, fields.first.y))
    local mark = ime.state_log_mark()
    ime.click_rel(fields.first.x, fields.first.y)
    ime.key_strokes("dkssudgktpdy", 0.03)
    ime.wait_for_state_log_pattern_after(mark, focus_pattern(fields.first, "안녕하세"), 3.0)
    maybe_capture("case_f_immediate_initial_phrase")
end

if scenario == "case_a" then
    run_case_a()
elseif scenario == "case_b" then
    run_case_b()
elseif scenario == "case_c" then
    run_case_c()
elseif scenario == "case_d" then
    run_case_d()
elseif scenario == "case_e" then
    run_case_e()
elseif scenario == "case_f" then
    run_case_f()
else
    run_case_e()
    run_case_d()
    run_case_a()
    run_case_b()
    run_case_c()
end

ime.log("finished showcase IME lab scenarios")
