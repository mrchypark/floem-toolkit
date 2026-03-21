use crate::primitives::theme_bind::ThemeBind;
use floem::event::EventListener;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{create_effect, RwSignal};
use floem::views::{self, text_input, PlaceholderTextClass, TextInputClass};
use floem_theme::{current_resolved_theme, ComponentSize};
use floem_tokens::ColorScale;
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

fn to_color(color: ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

struct ObservedInputState {
    last_emitted: String,
}

impl ObservedInputState {
    fn new(initial: String) -> Self {
        Self {
            last_emitted: initial,
        }
    }

    fn take_changed(&mut self, current: &str) -> Option<String> {
        if self.last_emitted == current {
            None
        } else {
            self.last_emitted = current.to_string();
            Some(self.last_emitted.clone())
        }
    }

    fn mark_seen(&mut self, current: &str) {
        self.last_emitted.clear();
        self.last_emitted.push_str(current);
    }
}

fn sanitize_single_line(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

fn input_debug_enabled_from(getter: impl Fn(&str) -> Option<OsString>) -> bool {
    getter("FLOEM_IME_DEBUG").is_some() || getter("FLOEM_DEBUG_TEXT_INPUT_IME").is_some()
}

fn input_debug_enabled() -> bool {
    input_debug_enabled_from(|key| std::env::var_os(key))
}

fn input_debug_log(label: &str, stage: &str, focused: bool, value: &str) {
    if !input_debug_enabled() {
        return;
    }
    eprintln!(
        "[floem-ui-input] label={} stage={} focused={} value={:?}",
        label, stage, focused, value
    );
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputDebugState {
    pub focused: bool,
    pub composing: bool,
    pub active: bool,
    pub value: String,
}

pub struct Input {
    value: String,
    buffer: Option<RwSignal<String>>,
    placeholder: Option<String>,
    invalid: bool,
    disabled: bool,
    on_input: Option<Box<dyn Fn(String)>>,
    on_debug_state_change: Option<Box<dyn Fn(InputDebugState)>>,
    diagnostic_label: Option<String>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            buffer: None,
            placeholder: None,
            invalid: false,
            disabled: false,
            on_input: None,
            on_debug_state_change: None,
            diagnostic_label: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn bind(mut self, buffer: RwSignal<String>) -> Self {
        self.buffer = Some(buffer);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_input(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_input = Some(Box::new(handler));
        self
    }

    pub fn on_debug_state_change(mut self, handler: impl Fn(InputDebugState) + 'static) -> Self {
        self.on_debug_state_change = Some(Box::new(handler));
        self
    }

    pub fn diagnostic_label(mut self, label: impl Into<String>) -> Self {
        self.diagnostic_label = Some(label.into());
        self
    }

    pub fn build(self) -> impl IntoView {
        let initial = current_resolved_theme().input_recipe(ComponentSize::Md, self.invalid);
        let recipe = RwSignal::new(initial);
        let hover_border = RwSignal::new(current_resolved_theme().input_hover_border(self.invalid));
        let focus_ring = RwSignal::new(current_resolved_theme().input_focus_ring());
        let disabled = self.disabled;
        let invalid = self.invalid;
        let buffer = self
            .buffer
            .unwrap_or_else(|| RwSignal::new(self.value.clone()));
        let observed = Rc::new(RefCell::new(ObservedInputState::new(
            buffer.get_untracked(),
        )));
        let focused = RwSignal::new(false);
        let on_input = self.on_input.map(Rc::new);
        let on_debug_state_change = self.on_debug_state_change.map(Rc::new);
        let debug_name = self
            .diagnostic_label
            .map(|label| format!("floem-ui::Input::{label}"))
            .unwrap_or_else(|| "floem-ui::Input".to_string());
        let debug_label = debug_name.clone();

        {
            let buffer = buffer;
            let observed = observed.clone();
            let on_input = on_input.clone();
            let debug_label = debug_label.clone();
            let focused = focused;
            create_effect(move |_| {
                let current = buffer.get();
                let sanitized = sanitize_single_line(&current);
                if current != sanitized {
                    input_debug_log(
                        &debug_label,
                        "sanitize_single_line",
                        focused.get_untracked(),
                        &sanitized,
                    );
                    observed.borrow_mut().mark_seen(&sanitized);
                    buffer.set(sanitized);
                    return;
                }

                if let Some(next) = observed.borrow_mut().take_changed(&current) {
                    input_debug_log(
                        &debug_label,
                        "emit_on_input",
                        focused.get_untracked(),
                        &next,
                    );
                    if let Some(on_input) = on_input.as_ref() {
                        on_input(next);
                    }
                }
            });
        }

        if let Some(debug_handler) = on_debug_state_change {
            let last_debug = Rc::new(RefCell::new(None::<InputDebugState>));
            let buffer = buffer;
            let focused = focused;
            create_effect(move |_| {
                let snapshot = InputDebugState {
                    focused: focused.get(),
                    composing: false,
                    active: focused.get(),
                    value: buffer.get(),
                };
                let mut last = last_debug.borrow_mut();
                if last.as_ref() != Some(&snapshot) {
                    debug_handler(snapshot.clone());
                    *last = Some(snapshot);
                }
            });
        }

        let focus_gained_label = debug_label.clone();
        let focus_lost_label = debug_label.clone();
        let input = match self.placeholder {
            Some(placeholder) => text_input(buffer).placeholder(placeholder),
            None => text_input(buffer),
        }
        .on_event_cont(EventListener::FocusGained, move |_| {
            focused.set(true);
            input_debug_log(
                &focus_gained_label,
                "focus_gained",
                true,
                &buffer.get_untracked(),
            );
        })
        .on_event_cont(EventListener::FocusLost, move |_| {
            focused.set(false);
            input_debug_log(
                &focus_lost_label,
                "focus_lost",
                false,
                &buffer.get_untracked(),
            );
        });

        ThemeBind::new(
            views::container(input.class(PlaceholderTextClass).style(move |s| {
                let recipe = recipe.get();
                s.color(to_color(recipe.foreground))
                    .width_full()
                    .background(Color::rgba8(0, 0, 0, 0))
                    .border(0.0)
                    .padding(0.0)
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
                    .disabled(|s| s.color(to_color(recipe.placeholder)))
            }))
            .style(move |s| {
                let recipe = recipe.get();
                let ring = focus_ring.get();
                let is_focused = focused.get();
                let border = if invalid {
                    recipe.border_invalid
                } else {
                    recipe.border
                };
                let effective_border = if is_focused {
                    recipe.border_focus
                } else {
                    border
                };
                let outline_color = if is_focused {
                    to_color(ring)
                } else {
                    Color::rgba8(0, 0, 0, 0)
                };
                s.width_full()
                    .height(recipe.height)
                    .padding_left(14.0)
                    .padding_right(14.0)
                    .background(to_color(recipe.background))
                    .border(1.0)
                    .border_color(to_color(effective_border))
                    .border_radius(recipe.radius)
                    .outline(if is_focused { 2.0 } else { 0.0 })
                    .outline_color(outline_color)
                    .hover(|s| {
                        if is_focused {
                            s.border_color(to_color(recipe.border_focus))
                        } else {
                            s.border_color(to_color(hover_border.get()))
                        }
                    })
            })
            .disabled(move || disabled)
            .debug_name(debug_name),
            move |theme| {
                recipe.set(theme.input_recipe(ComponentSize::Md, invalid));
                hover_border.set(theme.input_hover_border(invalid));
                focus_ring.set(theme.input_focus_ring());
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{input_debug_enabled_from, sanitize_single_line, ObservedInputState};
    use std::ffi::OsString;

    #[test]
    fn observed_change_ignores_same_value() {
        let mut observed = ObservedInputState::new("floem".to_string());
        assert_eq!(observed.take_changed("floem"), None);
    }

    #[test]
    fn observed_change_emits_and_updates_last_value() {
        let mut observed = ObservedInputState::new("floem".to_string());
        assert_eq!(
            observed.take_changed("floem-toolkit"),
            Some("floem-toolkit".to_string())
        );
        assert_eq!(observed.take_changed("floem-toolkit"), None);
    }

    #[test]
    fn mark_seen_prevents_re_emitting() {
        let mut observed = ObservedInputState::new("alpha".to_string());
        observed.mark_seen("beta");

        assert_eq!(observed.take_changed("beta"), None);
    }

    #[test]
    fn sanitize_replaces_newlines_with_spaces() {
        assert_eq!(sanitize_single_line("alpha\nbeta\r\n"), "alpha beta  ");
    }

    #[test]
    fn input_debug_enabled_accepts_new_primary_flag() {
        let enabled =
            input_debug_enabled_from(|key| (key == "FLOEM_IME_DEBUG").then(|| OsString::from("1")));

        assert!(enabled);
    }

    #[test]
    fn input_debug_enabled_accepts_legacy_flag() {
        let enabled = input_debug_enabled_from(|key| {
            (key == "FLOEM_DEBUG_TEXT_INPUT_IME").then(|| OsString::from("1"))
        });

        assert!(enabled);
    }
}
