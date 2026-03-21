use floem::action;
use floem::event::{Event, EventListener};
use floem::kurbo::Point;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::style::FlexWrap;
use floem::views;
use floem::window::WindowConfig;
use floem::{AnyView, Application};
use floem_charts::model::{ChartBounds, LineConfig, LineDatum};
use floem_theme::{
    current_theme_signal, resolved_from_definition, ButtonVariant, Patch, ThemeDefinition,
    ThemePatch, ThemeProvider, ThemeScope,
};
use floem_tokens::{ColorScale, ThemeMode};
use floem_ui::diagnostics::InputDebugState;
use floem_ui::prelude::{
    Button, Card, Checkbox, Dialog, Input, Label, Popover, Select, SelectOption, TabSpec, Tabs,
};
use std::sync::Arc;

mod debug_log;
mod plain_input_lab;

fn main() {
    Application::new()
        .window(|_| app_view(), debug_window_config())
        .run();
}

fn plain_input_lab_only() -> bool {
    std::env::var_os("FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY").is_some()
}

fn ime_lab_only() -> bool {
    std::env::var_os("FLOEM_SHOWCASE_DEBUG_IME_LAB_ONLY").is_some()
}

fn debug_autofocus_ime_first() -> bool {
    std::env::var_os("FLOEM_SHOWCASE_DEBUG_IME_AUTOFOCUS").is_some()
}

fn components_debug_only() -> bool {
    std::env::var_os("FLOEM_SHOWCASE_DEBUG_COMPONENTS_ONLY").is_some()
}

fn checkbox_diagnostics_enabled() -> bool {
    std::env::var_os("FLOEM_SHOWCASE_DEBUG_CHECKBOX_DIAGNOSTICS").is_some()
}

fn debug_window_config() -> Option<WindowConfig> {
    if components_debug_only() {
        Some(
            WindowConfig::default()
                .size((980.0, 820.0))
                .position(Point::new(120.0, 90.0)),
        )
    } else if plain_input_lab_only() || ime_lab_only() {
        Some(
            WindowConfig::default()
                .size((800.0, 632.0))
                .position(Point::new(160.0, 120.0)),
        )
    } else {
        None
    }
}

fn app_view() -> impl IntoView {
    let mode = RwSignal::new(ThemeMode::Dark);
    ThemeProvider::wrap(
        move || showcase_shell(mode),
        move || ThemeDefinition::new(mode.get()),
    )
}

fn showcase_shell(mode: RwSignal<ThemeMode>) -> impl IntoView {
    let theme = current_theme_signal();
    let components_only = components_debug_only();
    let plain_lab_only = plain_input_lab_only();
    let ime_lab_only = ime_lab_only();

    let body = if plain_lab_only {
        (plain_input_lab::plain_input_lab_surface(),).v_stack()
    } else if ime_lab_only {
        (ime_lab_surface(),).v_stack()
    } else if components_only {
        (components_section(),).v_stack()
    } else {
        (
            hero(mode),
            foundations_section(),
            components_section(),
            charts_section(),
            scope_section(),
        )
            .v_stack()
    };

    views::scroll(body.style(|s| s.width_full().max_width(960.0).padding(28.0).row_gap(18.0)))
        .style(move |s| {
            let palette = resolved_from_definition(&theme.get()).token_set;
            s.width_full()
                .height_full()
                .background(to_color(palette.semantic.colors.background))
                .color(to_color(palette.semantic.colors.foreground))
        })
}

fn ime_lab_surface() -> impl IntoView {
    views::container(
        (
            views::label(|| "Input IME verification surface".to_string())
                .style(|s| s.font_size(28.0).font_bold()),
            views::label(|| {
                "This screen uses floem-ui::Input directly. Use it to validate the wrapper path without the main showcase scroll stack."
                    .to_string()
            })
            .style(|s| s.font_size(14.0).line_height(1.5).max_width(760.0)),
            ime_lab_content(),
        )
            .v_stack()
            .style(|s| s.row_gap(14.0).width_full().max_width(860.0)),
    )
    .style(|s| s.width_full().height_full().padding(28.0))
    .debug_name("showcase::ime_lab_surface")
}

fn ime_lab_content() -> impl IntoView {
    let ime_first_pass = RwSignal::new(String::new());
    let ime_source = RwSignal::new(String::new());
    let ime_target = RwSignal::new(String::new());

    let ime_first_pass_debug = RwSignal::new(InputDebugState::default());
    let ime_source_debug = RwSignal::new(InputDebugState::default());
    let ime_target_debug = RwSignal::new(InputDebugState::default());

    (
        views::label(|| {
            "Use this section for Korean IME verification. The first field checks the first keystroke on an empty input. The next two fields check whether an unfinished syllable follows focus when you move during composition."
                .to_string()
        })
        .style(|s| s.font_size(13.0).line_height(1.5).max_width(760.0)),
        (
            Button::new("Reset IME lab")
                .variant(ButtonVariant::Outline)
                .on_press({
                    move || {
                        ime_first_pass.set(String::new());
                        ime_source.set(String::new());
                        ime_target.set(String::new());
                    }
                })
                .build(),
            views::label(|| {
                "Expected: first Hangul syllable should compose normally, and moving focus mid-composition should not copy the unfinished syllable into the next field."
                    .to_string()
            })
            .style(|s| s.font_size(12.0).line_height(1.45).max_width(560.0)),
        )
            .h_stack()
            .style(|s| s.items_center().column_gap(12.0)),
        (
            ime_lab_panel(
                (
                views::label(|| "First Hangul on empty input".to_string()).style(|s| s.font_bold()),
                if debug_autofocus_ime_first() {
                    Input::new()
                        .bind(ime_first_pass)
                        .placeholder("Type the very first Hangul syllable here")
                        .diagnostic_label("ime_first_pass")
                        .on_debug_state_change(move |state| {
                            debug_log::append_state_log(format!(
                                "ime-first focused={} value={:?}",
                                state.focused, state.value
                            ));
                            ime_first_pass_debug.set(state)
                        })
                        .build()
                        .on_event_cont(EventListener::ImePreedit, move |event| {
                            if let Event::ImePreedit { text, cursor } = event {
                                debug_log::append_state_log(format!(
                                    "ime-first ime_preedit text={text:?} cursor={cursor:?}"
                                ));
                            }
                        })
                        .on_event_cont(EventListener::ImeCommit, move |event| {
                            if let Event::ImeCommit(text) = event {
                                debug_log::append_state_log(format!(
                                    "ime-first ime_commit text={text:?}"
                                ));
                            }
                        })
                        .request_focus(|| {})
                        .into_any()
                } else {
                    Input::new()
                        .bind(ime_first_pass)
                        .placeholder("Type the very first Hangul syllable here")
                        .diagnostic_label("ime_first_pass")
                        .on_debug_state_change(move |state| {
                            debug_log::append_state_log(format!(
                                "ime-first focused={} value={:?}",
                                state.focused, state.value
                            ));
                            ime_first_pass_debug.set(state)
                        })
                        .build()
                        .on_event_cont(EventListener::ImePreedit, move |event| {
                            if let Event::ImePreedit { text, cursor } = event {
                                debug_log::append_state_log(format!(
                                    "ime-first ime_preedit text={text:?} cursor={cursor:?}"
                                ));
                            }
                        })
                        .on_event_cont(EventListener::ImeCommit, move |event| {
                            if let Event::ImeCommit(text) = event {
                                debug_log::append_state_log(format!(
                                    "ime-first ime_commit text={text:?}"
                                ));
                            }
                        })
                        .into_any()
                },
                views::label(move || format!("Value: {:?}", ime_first_pass.get())).style(|s| {
                    s.font_size(12.0)
                        .color(to_color(ColorScale::rgba(148, 163, 184, 255)))
                }),
            )
                    .v_stack()
                    .style(|s| s.row_gap(6.0)),
                300.0,
                340.0,
            ),
            ime_lab_panel(
                (
                views::label(|| "Composition handoff".to_string()).style(|s| s.font_bold()),
                (
                    ime_lab_input_slot(
                        Input::new()
                            .bind(ime_source)
                            .placeholder("Compose here, then click next field")
                            .diagnostic_label("ime_source")
                            .on_debug_state_change(move |state| {
                                debug_log::append_state_log(format!(
                                    "ime-source focused={} value={:?}",
                                    state.focused, state.value
                                ));
                                ime_source_debug.set(state)
                            })
                            .build()
                            .on_event_cont(EventListener::ImePreedit, move |event| {
                                if let Event::ImePreedit { text, cursor } = event {
                                    debug_log::append_state_log(format!(
                                        "ime-source ime_preedit text={text:?} cursor={cursor:?}"
                                    ));
                                }
                            })
                            .on_event_cont(EventListener::ImeCommit, move |event| {
                                if let Event::ImeCommit(text) = event {
                                    debug_log::append_state_log(format!(
                                        "ime-source ime_commit text={text:?}"
                                    ));
                                }
                            }),
                    ),
                    ime_lab_input_slot(
                        Input::new()
                            .bind(ime_target)
                            .placeholder("Target should stay clean on handoff")
                            .diagnostic_label("ime_target")
                            .on_debug_state_change(move |state| {
                                debug_log::append_state_log(format!(
                                    "ime-target focused={} value={:?}",
                                    state.focused, state.value
                                ));
                                ime_target_debug.set(state)
                            })
                            .build()
                            .on_event_cont(EventListener::ImePreedit, move |event| {
                                if let Event::ImePreedit { text, cursor } = event {
                                    debug_log::append_state_log(format!(
                                        "ime-target ime_preedit text={text:?} cursor={cursor:?}"
                                    ));
                                }
                            })
                            .on_event_cont(EventListener::ImeCommit, move |event| {
                                if let Event::ImeCommit(text) = event {
                                    debug_log::append_state_log(format!(
                                        "ime-target ime_commit text={text:?}"
                                    ));
                                }
                            }),
                    ),
                )
                    .h_stack()
                    .style(|s| s.flex_wrap(FlexWrap::Wrap).column_gap(12.0).row_gap(12.0)),
                views::label(move || {
                    format!(
                        "Source: {:?} | Target: {:?}",
                        ime_source.get(),
                        ime_target.get()
                    )
                })
                .style(|s| {
                    s.font_size(12.0)
                        .color(to_color(ColorScale::rgba(148, 163, 184, 255)))
                }),
            )
                    .v_stack()
                    .style(|s| s.row_gap(6.0)),
                420.0,
                480.0,
            ),
        )
            .h_stack()
            .style(|s| {
                s.flex_wrap(FlexWrap::Wrap)
                    .items_start()
                    .column_gap(18.0)
                    .row_gap(16.0)
            }),
        (
            input_debug_card("ime-first", ime_first_pass_debug),
            input_debug_card("ime-source", ime_source_debug),
            input_debug_card("ime-target", ime_target_debug),
        )
            .h_stack()
            .style(|s| {
                s.flex_wrap(FlexWrap::Wrap)
                    .column_gap(12.0)
                    .row_gap(12.0)
            }),
    )
        .v_stack()
        .style(|s| s.row_gap(12.0))
}

fn hero(mode: RwSignal<ThemeMode>) -> impl IntoView {
    Card::new()
        .header(
            (
                views::label(|| "floem-toolkit".to_string())
                    .style(|s| s.font_size(30.0).font_bold()),
                views::label(|| {
                    "shadcn/ui visual language translated into a Floem-native toolkit."
                        .to_string()
                })
                .style(|s| s.font_size(14.0).line_height(1.5).max_width(560.0)),
            )
                .v_stack()
                .style(|s| s.row_gap(8.0)),
        )
        .content(
            (
                Label::new("Current slice").build(),
                views::label(|| {
                    "Workspace bootstrap, semantic tokens, theme resolution, chart benchmark core, and the first UI components are live in a real showcase shell."
                        .to_string()
                })
                .style(|s| s.font_size(14.0).line_height(1.5).max_width(680.0)),
                views::label(|| {
                    "Use the inspector to verify the live view tree. The input control is tagged as floem-ui::Input."
                        .to_string()
                })
                .style(|s| s.font_size(13.0).line_height(1.45).max_width(680.0)),
                mode_switcher(mode),
            )
                .v_stack()
                .style(|s| s.row_gap(12.0)),
        )
        .build()
}

fn mode_switcher(mode: RwSignal<ThemeMode>) -> impl IntoView {
    (
        Button::new("Use dark mode")
            .variant(ButtonVariant::Primary)
            .on_press(move || mode.set(ThemeMode::Dark))
            .build(),
        Button::new("Use light mode")
            .variant(ButtonVariant::Secondary)
            .on_press(move || mode.set(ThemeMode::Light))
            .build(),
        Button::new("Open inspector")
            .variant(ButtonVariant::Outline)
            .on_press(action::inspect)
            .build(),
    )
        .h_stack()
        .style(|s| s.column_gap(10.0))
}

fn foundations_section() -> impl IntoView {
    Card::new()
        .header(section_heading(
            "Foundations",
            "The first shell consumes tokens through ThemeProvider rather than hardcoded component colors.",
        ))
        .content(
            views::container(
                (
                    foundation_chip("semantic tokens"),
                    foundation_chip("light / dark"),
                    foundation_chip("component recipes"),
                    foundation_chip("chart palette"),
                )
                    .h_stack()
                    .style(|s| {
                        s.flex_wrap(FlexWrap::Wrap)
                            .column_gap(8.0)
                            .row_gap(8.0)
                    }),
            )
            .style(|s| s.margin_top(8.0)),
        )
        .build()
}

fn components_section() -> impl IntoView {
    let project_name = RwSignal::new("floem-toolkit".to_string());
    let invalid_state = RwSignal::new("Needs review".to_string());
    let accepted = RwSignal::new(true);
    let active_tab = RwSignal::new(Arc::<str>::from("overview"));
    let dialog_open = RwSignal::new(false);
    let popover_open = RwSignal::new(false);
    let selected_item = RwSignal::new(Some(Arc::<str>::from("primary")));
    let diagnostics = if checkbox_diagnostics_enabled() {
        checkbox_diagnostics_section().into_any()
    } else {
        views::empty().style(|s| s.hide()).into_any()
    };

    Card::new()
        .header(section_heading(
            "UI / Slice 1",
            "Ready-to-use defaults for buttons, cards, inputs, checkboxes, tabs, dialogs, popovers, and selects. These examples keep real state where interaction is part of the component contract.",
        ))
        .content(
            (
                diagnostics,
                subsection(
                    "Buttons",
                    (
                        Button::new("Primary").build(),
                        Button::new("Secondary")
                            .variant(ButtonVariant::Secondary)
                            .build(),
                        Button::new("Outline")
                            .variant(ButtonVariant::Outline)
                            .build(),
                        Button::new("Ghost")
                            .variant(ButtonVariant::Ghost)
                            .build(),
                        Button::new("Destructive")
                            .variant(ButtonVariant::Destructive)
                            .build(),
                    )
                        .h_stack()
                        .style(|s| {
                            s.flex_wrap(FlexWrap::Wrap)
                                .column_gap(10.0)
                                .row_gap(10.0)
                        }),
                ),
                subsection(
                    "Cards",
                    (
                        preview_card(
                            "Release brief",
                            "Use a neutral surface for compact summaries, shipping notes, and short ownership context.",
                        ),
                        preview_card(
                            "Safe defaults",
                            "Inputs, toggles, and cards stay on one spacing and contrast system, so new forms start from a usable baseline.",
                        ),
                    )
                        .h_stack()
                        .style(|s| {
                            s.flex_wrap(FlexWrap::Wrap)
                                .column_gap(20.0)
                                .row_gap(16.0)
                        }),
                ),
                subsection(
                    "Form controls",
                    form_controls_showcase(project_name, invalid_state, accepted)
                        .debug_name("showcase::form_controls_panel"),
                ),
                subsection(
                    "Tabs",
                    Tabs::new()
                        .bind_value(active_tab)
                        .tabs(vec![
                            TabSpec::new(Arc::<str>::from("overview"), Arc::<str>::from("Overview")),
                            TabSpec::new(Arc::<str>::from("theme"), Arc::<str>::from("Theme")),
                            TabSpec::new(Arc::<str>::from("charts"), Arc::<str>::from("Charts")),
                        ])
                        .panel(
                            Arc::<str>::from("overview"),
                            views::label(|| {
                                "Keep the main view focused. Neighboring tabs should reveal related context without pushing the primary task away."
                                    .to_string()
                            })
                            .style(|s| s.line_height(1.45).max_width(520.0)),
                        )
                        .panel(
                            Arc::<str>::from("theme"),
                            views::label(|| {
                                "One token path drives cards, inputs, and toggles, so contrast and spacing move together."
                                    .to_string()
                            })
                            .style(|s| s.line_height(1.45).max_width(520.0)),
                        )
                        .panel(
                            Arc::<str>::from("charts"),
                            views::label(|| {
                                "Use tabbed space for distinct tasks, not for hiding required form decisions."
                                    .to_string()
                            })
                            .style(|s| s.line_height(1.45).max_width(520.0)),
                        )
                        .build(),
                ),
                subsection(
                    "Overlay controls",
                    (
                        Dialog::new()
                            .bind_open(dialog_open)
                            .title(Label::new("Delete branch?").build())
                            .content(views::label(|| {
                                "Dialog is currently an in-tree modal shell. Overlay positioning and focus policy will be tightened in the next pass."
                                    .to_string()
                            }))
                            .build(),
                        (
                            Button::new("Open dialog")
                                .on_press(move || dialog_open.set(true))
                                .build(),
                            Popover::new()
                                .bind_open(popover_open)
                                .trigger(Button::new("Toggle popover").build())
                                .content(views::label(|| {
                                    "Popover content is live on the same theme path."
                                        .to_string()
                                }))
                                .build(),
                            Select::new()
                                .bind_value(selected_item)
                                .options(vec![
                                    SelectOption::new(
                                        Arc::<str>::from("primary"),
                                        Arc::<str>::from("Primary"),
                                    ),
                                    SelectOption::new(
                                        Arc::<str>::from("secondary"),
                                        Arc::<str>::from("Secondary"),
                                    ),
                                    SelectOption::new(
                                        Arc::<str>::from("ghost"),
                                        Arc::<str>::from("Ghost"),
                                    ),
                                ])
                                .build(),
                        )
                            .h_stack()
                            .style(|s| s.items_start().column_gap(12.0)),
                    )
                        .v_stack()
                        .style(|s| s.row_gap(12.0)),
                ),
            )
                .v_stack()
                .style(|s| s.row_gap(24.0)),
        )
        .build()
}

const CHECKBOX_MARK_REFERENCE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 16 16" fill="none"><path d="M13.2 4.8L6.5 11.5L2.8 7.8" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

fn checkbox_diagnostics_section() -> impl IntoView {
    let raw_off = RwSignal::new(false);
    let raw_on = RwSignal::new(true);
    let ui_off = RwSignal::new(false);
    let ui_on = RwSignal::new(true);

    subsection(
        "Checkbox diagnostics",
        (
            views::label(|| {
                "The same unchecked / checked pair is rendered three ways. If raw vendor and floem-ui wrapper fail the same way while the reference composition is clean, the primitive layer owns the bug."
                    .to_string()
            })
            .style(|s| s.font_size(13.0).line_height(1.5).max_width(760.0)),
            (
                checkbox_variant_column(
                    "Vendor raw",
                    views::Checkbox::new_rw(raw_off).into_any(),
                    views::Checkbox::new_rw(raw_on).into_any(),
                ),
                checkbox_variant_column(
                    "floem-ui wrapper",
                    Checkbox::new().bind(ui_off).build().into_any(),
                    Checkbox::new().bind(ui_on).build().into_any(),
                ),
                checkbox_variant_column(
                    "Reference composition",
                    reference_checkbox(false),
                    reference_checkbox(true),
                ),
            )
                .h_stack()
                .style(|s| {
                    s.flex_wrap(FlexWrap::Wrap)
                        .items_start()
                        .column_gap(16.0)
                        .row_gap(16.0)
                }),
        )
            .v_stack()
            .style(|s| s.row_gap(12.0)),
    )
}

fn checkbox_variant_column(
    title: &'static str,
    unchecked: AnyView,
    checked: AnyView,
) -> impl IntoView {
    views::container(
        (
            views::label(move || title.to_string()).style(|s| s.font_size(13.0).font_bold()),
            checkbox_variant_state("off", unchecked),
            checkbox_variant_state("on", checked),
        )
            .v_stack()
            .style(|s| s.row_gap(10.0)),
    )
    .style(|s| {
        s.min_width(180.0)
            .padding(14.0)
            .border(1.0)
            .border_radius(12.0)
            .border_color(to_color(ColorScale::rgba(100, 116, 139, 96)))
    })
}

fn checkbox_variant_state(label: &'static str, control: AnyView) -> impl IntoView {
    (
        views::label(move || label.to_string()).style(|s| {
            s.min_width(26.0)
                .font_size(12.0)
                .color(to_color(ColorScale::rgba(148, 163, 184, 210)))
        }),
        control,
    )
        .h_stack()
        .style(|s| s.items_center().column_gap(10.0))
}

fn ime_lab_panel(
    content: impl IntoView + 'static,
    min_width: f64,
    max_width: f64,
) -> impl IntoView {
    views::container(content).style(move |s| {
        s.min_width(min_width)
            .max_width(max_width)
            .flex_basis(min_width)
            .flex_grow(1.0)
    })
}

fn ime_lab_input_slot(content: impl IntoView + 'static) -> impl IntoView {
    views::container(content).style(|s| s.width(220.0).min_width(220.0).max_width(220.0))
}

fn reference_checkbox(checked: bool) -> AnyView {
    let theme = current_theme_signal();
    let icon = if checked {
        let icon_theme = theme;
        views::svg(CHECKBOX_MARK_REFERENCE_SVG)
            .style(move |s| {
                let recipe = resolved_from_definition(&icon_theme.get()).checkbox_recipe();
                s.size(12.0, 12.0)
                    .color(to_color(recipe.checked_foreground))
            })
            .into_any()
    } else {
        views::empty().style(|s| s.size(12.0, 12.0)).into_any()
    };

    views::container(icon)
        .style(move |s| {
            let recipe = resolved_from_definition(&theme.get()).checkbox_recipe();
            let background = if checked {
                recipe.checked
            } else {
                recipe.background
            };
            s.size(18.0, 18.0)
                .padding(3.0)
                .justify_center()
                .items_center()
                .background(to_color(background))
                .border(1.0)
                .border_color(to_color(recipe.border))
                .border_radius(recipe.radius)
        })
        .into_any()
}

fn form_field_title(text: &'static str) -> impl IntoView {
    Label::new(text)
        .build()
        .style(|s| s.font_size(12.0).font_bold().line_height(1.3))
}

fn form_field_help(text: &'static str) -> impl IntoView {
    views::label(move || text.to_string()).style(|s| {
        s.font_size(12.0)
            .line_height(1.45)
            .max_width(360.0)
            .color(to_color(ColorScale::rgba(148, 163, 184, 220)))
    })
}

fn form_controls_showcase(
    project_name: RwSignal<String>,
    invalid_state: RwSignal<String>,
    accepted: RwSignal<bool>,
) -> impl IntoView {
    Card::new()
        .content(
            (
                form_setting_row(
                    "Project name",
                    "Used in release notes and the workspace switcher.",
                    Input::new()
                        .bind(project_name)
                        .placeholder("Project name")
                        .build()
                        .debug_name("showcase::project_name_input")
                        .into_any(),
                    false,
                ),
                form_setting_row(
                    "Review status",
                    "Required before release. Keep the invalid state readable without overpowering the form.",
                    Input::new()
                        .bind(invalid_state)
                        .invalid(true)
                        .build()
                        .debug_name("showcase::invalid_state_input")
                        .into_any(),
                    true,
                ),
                form_toggle_row(accepted, true),
            )
                .v_stack()
                .style(|s| s.width_full().row_gap(0.0)),
        )
        .build()
        .style(|s| s.width_full().max_width(720.0))
}

fn form_setting_row(
    title: &'static str,
    help: &'static str,
    control: AnyView,
    separated: bool,
) -> impl IntoView {
    form_row_shell(
        (form_row_info(title, help), form_control_slot(control))
            .h_stack()
            .style(|s| {
                s.width_full()
                    .items_center()
                    .justify_between()
                    .flex_wrap(FlexWrap::Wrap)
                    .column_gap(24.0)
                    .row_gap(14.0)
            }),
        separated,
    )
}

fn form_toggle_row(accepted: RwSignal<bool>, separated: bool) -> impl IntoView {
    form_row_shell(
        (
            form_row_info(
                "Release gate",
                "Use a single checkbox row when one explicit approval state should be easy to scan.",
            ),
            (
                Checkbox::new()
                    .bind(accepted)
                    .build()
                    .debug_name("showcase::release_gate_checkbox"),
                views::label(move || {
                    if accepted.get() {
                        "Ready for review".to_string()
                    } else {
                        "Waiting for changes".to_string()
                    }
                })
                .style(|s| s.font_size(14.0).font_bold().line_height(1.3)),
            )
                .h_stack()
                .style(|s| s.items_center().column_gap(12.0)),
        )
            .h_stack()
            .style(|s| {
                s.width_full()
                    .items_center()
                    .justify_between()
                    .flex_wrap(FlexWrap::Wrap)
                    .column_gap(24.0)
                    .row_gap(14.0)
            }),
        separated,
    )
}

fn form_row_info(title: &'static str, help: &'static str) -> impl IntoView {
    views::container(
        (form_field_title(title), form_field_help(help))
            .v_stack()
            .style(|s| s.row_gap(6.0)),
    )
    .style(|s| s.min_width(220.0).flex_basis(0.0).flex_grow(1.0))
}

fn form_control_slot(control: AnyView) -> impl IntoView {
    views::container(control).style(|s| {
        s.width_full()
            .min_width(240.0)
            .max_width(320.0)
            .flex_basis(280.0)
            .flex_grow(1.0)
    })
}

fn form_row_shell(content: impl IntoView + 'static, separated: bool) -> impl IntoView {
    let theme = current_theme_signal();
    views::container(content).style(move |s| {
        let recipe = resolved_from_definition(&theme.get()).card_recipe();
        let s = s
            .width_full()
            .padding_top(if separated { 18.0 } else { 0.0 })
            .padding_bottom(18.0);

        if separated {
            s.border_top(1.0).border_color(to_color(recipe.border))
        } else {
            s
        }
    })
}

fn scope_section() -> impl IntoView {
    Card::new()
        .header(section_heading(
            "ThemeScope Proof",
            "A subtree override changes primary, card, and border roles without rebuilding the whole shell.",
        ))
        .content(
            (
                views::label(|| {
                    "The card below is rendered inside ThemeScope with a local token patch."
                        .to_string()
                })
                .style(|s| s.font_size(14.0).line_height(1.5)),
                ThemeScope::wrap(
                    move || {
                        Card::new()
                            .header(
                                (
                                    Label::new("Scoped surface").build(),
                                    views::label(|| {
                                        "This proves provider -> scope -> component resolution on a live view path."
                                            .to_string()
                                    })
                                    .style(|s| s.font_size(13.0).line_height(1.45)),
                                )
                                    .v_stack()
                                    .style(|s| s.row_gap(6.0)),
                            )
                            .content(
                                (
                                    Button::new("Scoped primary").build(),
                                    Button::new("Scoped outline")
                                        .variant(ButtonVariant::Outline)
                                        .build(),
                                )
                                    .h_stack()
                                    .style(|s| s.column_gap(10.0)),
                            )
                            .build()
                    },
                    ThemePatch {
                        primary: Patch::Set(ColorScale::rgb(38, 120, 92)),
                        card: Patch::Set(ColorScale::rgb(235, 246, 240)),
                        border: Patch::Set(ColorScale::rgb(125, 164, 142)),
                        foreground: Patch::Set(ColorScale::rgb(21, 46, 36)),
                        ..ThemePatch::default()
                    },
                ),
            )
                .v_stack()
                .style(|s| s.row_gap(12.0)),
        )
        .build()
}

fn charts_section() -> impl IntoView {
    let config = LineConfig {
        bounds: ChartBounds {
            width: 320.0,
            height: 120.0,
        },
    };
    let projected = config.project(&[
        LineDatum { x: 0.0, y: 12.0 },
        LineDatum { x: 1.0, y: 26.0 },
        LineDatum { x: 2.0, y: 18.0 },
        LineDatum { x: 3.0, y: 34.0 },
        LineDatum { x: 4.0, y: 22.0 },
    ]);

    Card::new()
        .header(section_heading(
            "Charts / V0",
            "The chart crate now exposes typed datum/config/domain projection for the next interactive rendering slice.",
        ))
        .content(
            (
                subsection(
                    "Projection snapshot",
                    views::label(move || {
                        projected
                            .iter()
                            .map(|point| format!("({:.0}, {:.0})", point.x, point.y))
                            .collect::<Vec<_>>()
                            .join("  ")
                    })
                    .style(|s| s.line_height(1.45).max_width(640.0)),
                ),
                subsection(
                    "Benchmarks",
                    (
                        metric_card(
                            "scale_mapping_4096",
                            "about 4.69µs",
                            "linear domain/range mapping",
                        ),
                        metric_card(
                            "nearest_point_lookup_4096",
                            "about 29.6µs",
                            "single-series hover lookup",
                        ),
                        metric_card(
                            "nearest_point_lookup_multi_series_8x1024",
                            "about 11.4µs",
                            "multi-series hover lookup",
                        ),
                    )
                        .h_stack()
                        .style(|s| {
                            s.flex_wrap(FlexWrap::Wrap)
                                .column_gap(16.0)
                                .row_gap(16.0)
                        }),
                ),
            )
                .v_stack()
                .style(|s| s.row_gap(24.0)),
        )
        .build()
}

fn section_heading(title: &'static str, body: &'static str) -> impl IntoView {
    (
        views::label(move || title.to_string()).style(|s| s.font_size(20.0).font_bold()),
        views::label(move || body.to_string())
            .style(|s| s.font_size(14.0).line_height(1.5).max_width(640.0)),
    )
        .v_stack()
        .style(|s| s.row_gap(6.0))
}

fn foundation_chip(text: &'static str) -> impl IntoView {
    views::label(move || text.to_string()).style(|s| {
        s.padding_left(10.0)
            .padding_right(10.0)
            .padding_top(6.0)
            .padding_bottom(6.0)
            .border(1.0)
            .border_radius(999.0)
    })
}

fn subsection(title: &'static str, content: impl IntoView + 'static) -> impl IntoView {
    (
        views::label(move || title.to_string())
            .style(|s| s.font_size(18.0).font_bold().line_height(1.2)),
        views::container(content).style(|s| s.margin_top(6.0)),
    )
        .v_stack()
        .style(|s| s.row_gap(14.0))
}

fn input_debug_card(title: &'static str, state: RwSignal<InputDebugState>) -> impl IntoView {
    views::container(
        (
            views::label(move || format!("Input {}", title)).style(|s| s.font_bold()),
            views::label(move || format!("focused: {}", state.get().focused)),
            views::label(move || format!("value: {:?}", state.get().value))
                .style(|s| s.line_height(1.4)),
        )
            .v_stack()
            .style(|s| s.row_gap(4.0)),
    )
    .style(|s| {
        s.min_width(180.0)
            .padding(12.0)
            .border(1.0)
            .border_radius(10.0)
            .border_color(to_color(ColorScale::rgba(100, 116, 139, 96)))
    })
}

fn preview_card(title: &'static str, body: &'static str) -> impl IntoView {
    Card::new()
        .header(Label::new(title).build())
        .content(
            views::label(move || body.to_string())
                .style(|s| s.font_size(13.0).line_height(1.45).max_width(280.0)),
        )
        .build()
        .style(|s| s.width(320.0))
}

fn metric_card(title: &'static str, value: &'static str, body: &'static str) -> impl IntoView {
    Card::new()
        .header(
            (
                views::label(move || title.to_string()).style(|s| s.font_size(14.0).font_bold()),
                views::label(move || value.to_string()).style(|s| s.font_size(22.0).font_bold()),
            )
                .v_stack()
                .style(|s| s.row_gap(6.0)),
        )
        .content(
            views::label(move || body.to_string())
                .style(|s| s.font_size(13.0).line_height(1.45).max_width(220.0)),
        )
        .build()
        .style(|s| s.width(250.0))
}

fn to_color(color: ColorScale) -> floem::peniko::Color {
    floem::peniko::Color::rgba8(color.r, color.g, color.b, color.a)
}
