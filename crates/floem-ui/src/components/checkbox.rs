use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::{current_resolved_theme, ResolvedTheme};
use std::rc::Rc;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub struct Checkbox {
    checked: bool,
    state: Option<RwSignal<bool>>,
    disabled: bool,
    on_checked_change: Option<Box<dyn Fn(bool)>>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: false,
            state: None,
            disabled: false,
            on_checked_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn bind(mut self, state: RwSignal<bool>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_checked_change(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_checked_change = Some(Box::new(handler));
        self
    }

    pub fn build(self) -> impl IntoView {
        let theme = RwSignal::new(current_resolved_theme());
        let recipe = RwSignal::new(theme.get_untracked().checkbox_recipe());
        let checked = self.state.unwrap_or_else(|| RwSignal::new(self.checked));
        let disabled = self.disabled;
        let on_checked_change = self.on_checked_change.map(Rc::new);

        let checkbox = views::checkbox(move || checked.get())
            .on_update(move |next| {
                checked.set(next);
                if let Some(on_checked_change) = on_checked_change.as_ref() {
                    on_checked_change(next);
                }
            })
            .disabled(move || disabled)
            .style(move |s| {
                let recipe = recipe.get();
                let theme = theme.get();
                let is_checked = checked.get();
                let background = if is_checked {
                    recipe.checked
                } else {
                    recipe.background
                };
                let hover_background = theme.checkbox_hover_background(is_checked, &recipe);
                let active_background = theme.checkbox_active_background(is_checked, &recipe);
                let focus_ring = theme.checkbox_focus_ring(&recipe);
                s.class(views::CheckboxClass, move |s| {
                    s.background(to_color(background))
                        .border(1.0)
                        .border_color(to_color(recipe.border))
                        .border_radius(recipe.radius)
                        .size(18.0, 18.0)
                        .items_center()
                        .justify_center()
                        .hover(|s| s.background(to_color(hover_background)))
                        .active(|s| s.background(to_color(active_background)))
                        .focus_visible(|s| s.outline(2.0).outline_color(to_color(focus_ring)))
                })
                .class(views::CheckboxMarkClass, move |s| {
                    s.size(12.0, 12.0)
                        .color(to_color(recipe.checked_foreground))
                })
            });

        ThemeBind::new(checkbox, move |next_theme: ResolvedTheme| {
            let next = next_theme.checkbox_recipe();
            theme.set(next_theme);
            recipe.set(next);
        })
    }
}
