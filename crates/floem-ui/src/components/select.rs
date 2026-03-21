use crate::components::SharedString;
use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use floem_theme::{current_resolved_theme, ResolvedTheme};
use std::rc::Rc;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

#[derive(Clone)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct Select {
    value: Option<SharedString>,
    state: Option<RwSignal<Option<SharedString>>>,
    on_value_change: Option<Box<dyn Fn(Option<SharedString>)>>,
    options: Vec<SelectOption>,
}

impl Select {
    pub fn new() -> Self {
        Self {
            value: None,
            state: None,
            on_value_change: None,
            options: Vec::new(),
        }
    }

    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    pub fn bind_value(mut self, state: RwSignal<Option<SharedString>>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn on_value_change(mut self, handler: impl Fn(Option<SharedString>) + 'static) -> Self {
        self.on_value_change = Some(Box::new(handler));
        self
    }

    pub fn options(mut self, options: Vec<SelectOption>) -> Self {
        self.options = options;
        self
    }

    pub fn build(self) -> impl IntoView {
        let theme = RwSignal::new(current_resolved_theme());
        let recipe = RwSignal::new(theme.get_untracked().select_recipe());
        let hover_border = RwSignal::new(theme.get_untracked().select_hover_border());
        let focus_ring = RwSignal::new(theme.get_untracked().select_focus_ring());
        let state = self
            .state
            .unwrap_or_else(|| RwSignal::new(self.value.clone()));
        let on_value_change = self.on_value_change.map(Rc::new);
        let options = self
            .options
            .into_iter()
            .filter(|option| !option.disabled)
            .collect::<Vec<_>>();

        let selected = RwSignal::new(selected_option(state.get_untracked().clone(), &options));

        let dropdown = Dropdown::custom(
            move || selected.get(),
            |item: SelectOption| floem::views::text(item.label.to_string()).into_any(),
            options.clone(),
            |item: SelectOption| floem::views::text(item.label.to_string()).into_any(),
        )
        .on_accept(move |option| {
            let next = Some(option.value.clone());
            selected.set(option);
            state.set(next.clone());
            if let Some(on_value_change) = on_value_change.as_ref() {
                on_value_change(next);
            }
        })
        .style(move |s| {
            let recipe = recipe.get();
            s.background(to_color(recipe.input.background))
                .color(to_color(recipe.input.foreground))
                .border(1.0)
                .border_color(to_color(recipe.input.border))
                .border_radius(recipe.input.radius)
                .height(recipe.input.height)
                .padding_left(12.0)
                .padding_right(12.0)
                .hover(|s| s.border_color(to_color(hover_border.get())))
                .focus_visible(|s| {
                    s.border_color(to_color(recipe.input.border_focus))
                        .outline(1.0)
                        .outline_color(to_color(focus_ring.get()))
                })
        });

        ThemeBind::new(dropdown, move |next_theme: ResolvedTheme| {
            recipe.set(next_theme.select_recipe());
            hover_border.set(next_theme.select_hover_border());
            focus_ring.set(next_theme.select_focus_ring());
            theme.set(next_theme);
            selected.set(selected_option(state.get(), &options));
        })
    }
}

fn selected_option(value: Option<SharedString>, options: &[SelectOption]) -> SelectOption {
    if let Some(value) = value {
        if let Some(option) = options.iter().find(|option| option.value == value) {
            return option.clone();
        }
    }
    options
        .first()
        .cloned()
        .unwrap_or_else(|| SelectOption::new("", ""))
}
