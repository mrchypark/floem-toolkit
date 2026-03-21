use floem::event::{Event, EventListener};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{create_effect, RwSignal};
use floem::views::{self, text_input, PlaceholderTextClass, TextInputClass};
use floem::{AnyView, ViewId};
use floem_theme::{current_resolved_theme, ComponentSize};
use floem_tokens::ColorScale;
use floem_ui::prelude::{Button, ButtonVariant};
use floem_ui::primitives::theme_bind::ThemeBind;

use crate::debug_log;

fn to_color(color: ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

#[derive(Clone)]
pub(crate) struct PlainInputLabState {
    pub(crate) first: RwSignal<String>,
    pub(crate) source: RwSignal<String>,
    pub(crate) target: RwSignal<String>,
}

impl PlainInputLabState {
    pub(crate) fn new() -> Self {
        Self {
            first: RwSignal::new(String::new()),
            source: RwSignal::new(String::new()),
            target: RwSignal::new(String::new()),
        }
    }

    pub(crate) fn reset(&self) {
        self.first.set(String::new());
        self.source.set(String::new());
        self.target.set(String::new());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PlainInputDebugState {
    focused: bool,
    value: String,
    preedit: String,
}

struct PlainInputField {
    id: ViewId,
    view: AnyView,
}

pub(crate) fn plain_input_lab_surface() -> impl IntoView {
    let state = PlainInputLabState::new();

    views::container(
        (
            views::label(|| "Plain text_input verification surface".to_string())
                .style(|s| s.font_size(28.0).font_bold()),
            views::label(|| {
                "This screen uses vendored Floem text_input directly. Use it for stable native automation, screenshots, and primitive-vs-wrapper IME comparisons."
                    .to_string()
            })
            .style(|s| s.font_size(14.0).line_height(1.5).max_width(760.0)),
            plain_input_lab_content(state),
        )
            .v_stack()
            .style(|s| s.row_gap(14.0).width_full().max_width(860.0)),
    )
    .on_event_cont(EventListener::PointerDown, move |event| {
        if let Event::PointerDown(pointer_event) = event {
            debug_log::append_state_log(format!(
                "showcase::plain_input_lab_surface pointer_down x={:.1} y={:.1}",
                pointer_event.pos.x, pointer_event.pos.y
            ));
        }
    })
    .on_event_cont(EventListener::KeyDown, move |event| {
        if let Event::KeyDown(key_event) = event {
            debug_log::append_state_log(format!(
                "showcase::plain_input_lab_surface key_down key={:?} text={:?}",
                key_event.key, key_event.key.text
            ));
        }
    })
    .on_event_cont(EventListener::ImePreedit, move |event| {
        if let Event::ImePreedit { text, cursor } = event {
            debug_log::append_state_log(format!(
                "showcase::plain_input_lab_surface ime_preedit text={:?} cursor={:?}",
                text, cursor
            ));
        }
    })
    .on_event_cont(EventListener::ImeCommit, move |event| {
        if let Event::ImeCommit(text) = event {
            debug_log::append_state_log(format!(
                "showcase::plain_input_lab_surface ime_commit text={:?}",
                text
            ));
        }
    })
    .style(|s| s.width_full().height_full().padding(28.0))
    .debug_name("showcase::plain_input_lab_surface")
}

fn plain_input_lab_content(state: PlainInputLabState) -> impl IntoView {
    let first_debug = RwSignal::new(PlainInputDebugState::default());
    let source_debug = RwSignal::new(PlainInputDebugState::default());
    let target_debug = RwSignal::new(PlainInputDebugState::default());
    let first_field = build_plain_input_field(
        state.first,
        "Plain first-syllable field",
        "showcase::plain_input_lab_first",
        first_debug,
    );
    let source_field = build_plain_input_field(
        state.source,
        "Plain source field",
        "showcase::plain_input_lab_source",
        source_debug,
    );
    let target_field = build_plain_input_field(
        state.target,
        "Plain target field",
        "showcase::plain_input_lab_target",
        target_debug,
    );
    (
        views::label(|| {
            "This lab uses vendored Floem text_input directly. It isolates primitive IME and focus behavior from floem-ui::Input."
                .to_string()
        })
        .style(|s| s.font_size(13.0).line_height(1.5).max_width(760.0)),
        (
            Button::new("Reset plain lab")
                .variant(ButtonVariant::Outline)
                .on_press({
                    let state = state.clone();
                    move || {
                        debug_log::append_state_log("showcase::plain_input_lab_reset pressed");
                        state.reset()
                    }
                })
                .build(),
            views::label(|| {
                "Use this surface to tell whether a bug lives in the primitive or only in the toolkit wrapper."
                    .to_string()
            })
            .style(|s| s.font_size(12.0).line_height(1.45).max_width(560.0)),
        )
            .h_stack()
            .style(|s| s.items_center().column_gap(12.0)),
        (
            (
                views::label(|| "First Hangul on empty input".to_string()).style(|s| s.font_bold()),
                views::label(|| "Test field".to_string())
                    .style(|s| s.font_size(12.0).font_bold()),
                plain_input_field_view(
                    "Focus test field",
                    "showcase::plain_input_lab_first",
                    first_field,
                ),
                plain_value_label(
                    "Value",
                    state.first,
                    "showcase::plain_input_lab_first_value pointer_down",
                ),
                plain_debug_card(
                    "Plain first",
                    first_debug,
                    "showcase::plain_input_lab_first_debug pointer_down",
                ),
            )
                .v_stack()
                .style(|s| s.row_gap(6.0))
                .debug_name("showcase::plain_input_lab_first_group"),
            (
                views::label(|| "Composition handoff".to_string()).style(|s| s.font_bold()),
                (
                    (
                        views::label(|| "Source field".to_string())
                            .style(|s| s.font_size(12.0).font_bold()),
                        plain_input_field_view(
                            "Focus source field",
                            "showcase::plain_input_lab_source",
                            source_field,
                        ),
                        plain_value_label(
                            "Source",
                            state.source,
                            "showcase::plain_input_lab_source_value pointer_down",
                        ),
                        plain_debug_card(
                            "Plain source",
                            source_debug,
                            "showcase::plain_input_lab_source_debug pointer_down",
                        ),
                    )
                        .v_stack()
                        .style(|s| s.row_gap(6.0))
                        .debug_name("showcase::plain_input_lab_source_group"),
                    (
                        views::label(|| "Target field".to_string())
                            .style(|s| s.font_size(12.0).font_bold()),
                        plain_input_field_view(
                            "Focus target field",
                            "showcase::plain_input_lab_target",
                            target_field,
                        ),
                        plain_value_label(
                            "Target",
                            state.target,
                            "showcase::plain_input_lab_target_value pointer_down",
                        ),
                        plain_debug_card(
                            "Plain target",
                            target_debug,
                            "showcase::plain_input_lab_target_debug pointer_down",
                        ),
                    )
                        .v_stack()
                        .style(|s| s.row_gap(6.0))
                        .debug_name("showcase::plain_input_lab_target_group"),
                )
                    .h_stack()
                    .style(|s| {
                        s.flex_wrap(floem::style::FlexWrap::Wrap)
                            .column_gap(12.0)
                            .row_gap(10.0)
                    }),
            )
                .v_stack()
                .style(|s| s.row_gap(6.0))
                .debug_name("showcase::plain_input_lab_handoff_group"),
        )
            .h_stack()
            .style(|s| {
                s.flex_wrap(floem::style::FlexWrap::Wrap)
                    .items_start()
                    .column_gap(18.0)
                    .row_gap(16.0)
            }),
    )
        .v_stack()
        .style(|s| s.row_gap(12.0))
}

fn plain_input_field_view(
    focus_button_label: &'static str,
    debug_name: &'static str,
    field: PlainInputField,
) -> impl IntoView {
    let field_id = field.id;
    let focus_debug_name = format!("{debug_name} focus_button");

    (
        views::container(
            views::label(move || focus_button_label.to_string()).style(|s| {
                s.font_size(13.0)
                    .color(to_color(ColorScale::rgba(226, 232, 240, 255)))
            }),
        )
        .on_event_cont(EventListener::PointerDown, move |_| {
            debug_log::append_state_log(format!("{focus_debug_name} pressed"));
            field_id.request_focus();
        })
        .style(|s| {
            s.min_width(220.0)
                .padding_vert(8.0)
                .justify_center()
                .items_center()
                .border(1.0)
                .border_radius(10.0)
                .border_color(to_color(ColorScale::rgba(71, 85, 105, 255)))
                .background(to_color(ColorScale::rgba(15, 23, 42, 255)))
        }),
        field.view,
    )
        .v_stack()
        .style(|s| s.row_gap(6.0))
}

fn build_plain_input_field(
    buffer: RwSignal<String>,
    placeholder: &'static str,
    debug_name: &'static str,
    debug_state: RwSignal<PlainInputDebugState>,
) -> PlainInputField {
    let focused = RwSignal::new(false);
    let preedit = RwSignal::new(String::new());
    let recipe = RwSignal::new(current_resolved_theme().input_recipe(ComponentSize::Md, false));
    let hover_border = RwSignal::new(current_resolved_theme().input_hover_border(false));
    let focus_ring = RwSignal::new(current_resolved_theme().input_focus_ring());

    create_effect(move |_| {
        let state = PlainInputDebugState {
            focused: focused.get(),
            value: buffer.get(),
            preedit: preedit.get(),
        };
        debug_log::append_state_log(format!(
            "{} focused={} value={:?}",
            debug_name, state.focused, state.value
        ));
        debug_state.set(state);
    });

    let input = text_input(buffer)
        .placeholder(placeholder)
        .on_event_cont(EventListener::PointerDown, move |_| {
            debug_log::append_state_log(format!("{debug_name} pointer_down"));
        })
        .on_event_cont(EventListener::ImePreedit, move |event| {
            if let Event::ImePreedit { text, cursor } = event {
                preedit.set(text.clone());
                debug_log::append_state_log(format!(
                    "{debug_name} ime_preedit text={text:?} cursor={cursor:?}"
                ));
            }
        })
        .on_event_cont(EventListener::ImeCommit, move |event| {
            if let Event::ImeCommit(text) = event {
                preedit.set(String::new());
                debug_log::append_state_log(format!("{debug_name} ime_commit text={text:?}"));
            }
        })
        .on_event_cont(EventListener::FocusGained, move |_| {
            focused.set(true);
        })
        .on_event_cont(EventListener::FocusLost, move |_| {
            focused.set(false);
            preedit.set(String::new());
        })
        .class(PlaceholderTextClass)
        .style(move |s| {
            let recipe = recipe.get();
            let is_focused = focused.get();
            s.width_full()
                .background(Color::rgba8(0, 0, 0, 0))
                .color(to_color(recipe.foreground))
                .font_size(recipe.font_size)
                .line_height(recipe.line_height as f32)
                .class(PlaceholderTextClass, |s| {
                    s.color(to_color(recipe.placeholder))
                        .font_size(recipe.font_size)
                })
                .class(TextInputClass, |s| {
                    s.background(Color::rgba8(0, 0, 0, 0))
                        .border(0.0)
                        .padding(0.0)
                        .outline(0.0)
                        .hover(|s| s.background(Color::rgba8(0, 0, 0, 0)))
                        .focus(|s| s.background(Color::rgba8(0, 0, 0, 0)))
                })
                .hover(|s| {
                    if is_focused {
                        s.border_color(to_color(recipe.border_focus))
                    } else {
                        s.border_color(to_color(hover_border.get()))
                    }
                })
        });
    let input_id = input.id();

    PlainInputField {
        id: input_id,
        view: ThemeBind::new(
            views::container(input)
                .style(move |s| {
                    let recipe = recipe.get();
                    let ring = focus_ring.get();
                    let is_focused = focused.get();
                    s.min_width(220.0)
                        .height(recipe.height)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .background(to_color(recipe.background))
                        .border(1.0)
                        .border_color(if is_focused {
                            to_color(recipe.border_focus)
                        } else {
                            to_color(recipe.border)
                        })
                        .border_radius(recipe.radius)
                        .outline(if is_focused { 1.0 } else { 0.0 })
                        .outline_color(if is_focused {
                            to_color(ring)
                        } else {
                            Color::rgba8(0, 0, 0, 0)
                        })
                })
                .debug_name(debug_name),
            move |theme| {
                recipe.set(theme.input_recipe(ComponentSize::Md, false));
                hover_border.set(theme.input_hover_border(false));
                focus_ring.set(theme.input_focus_ring());
            },
        )
        .into_any(),
    }
}

fn plain_value_label(
    label: &'static str,
    buffer: RwSignal<String>,
    debug_name: &'static str,
) -> impl IntoView {
    views::label(move || format!("{}: {:?}", label, buffer.get()))
        .on_event_cont(EventListener::PointerDown, move |_| {
            debug_log::append_state_log(debug_name);
        })
        .style(|s| {
            s.font_size(12.0)
                .color(to_color(ColorScale::rgba(148, 163, 184, 255)))
        })
}

fn plain_debug_card(
    label: &'static str,
    debug: RwSignal<PlainInputDebugState>,
    debug_name: &'static str,
) -> impl IntoView {
    views::container(
        views::label(move || {
            let state = debug.get();
            format!(
                "{label}\nfocused: {}\nvalue: {:?}\npreedit: {:?}",
                state.focused, state.value, state.preedit
            )
        })
        .style(|s| s.font_size(12.0).line_height(1.35)),
    )
    .on_event_cont(EventListener::PointerDown, move |_| {
        debug_log::append_state_log(debug_name);
    })
    .style(|s| {
        s.min_width(220.0)
            .padding(12.0)
            .border(1.0)
            .border_radius(12.0)
            .border_color(to_color(ColorScale::rgba(51, 65, 85, 255)))
    })
}

#[cfg(test)]
mod tests {
    use super::PlainInputLabState;
    use floem::prelude::{SignalGet, SignalUpdate};

    #[test]
    fn reset_clears_all_buffers() {
        let state = PlainInputLabState::new();
        state.first.set("one".to_string());
        state.source.set("two".to_string());
        state.target.set("three".to_string());

        state.reset();

        assert_eq!(state.first.get_untracked(), "");
        assert_eq!(state.source.get_untracked(), "");
        assert_eq!(state.target.get_untracked(), "");
    }
}
