use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::{current_resolved_theme, ComponentSize, ResolvedTheme};

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub type ButtonVariant = floem_theme::ButtonVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl From<ButtonSize> for ComponentSize {
    fn from(value: ButtonSize) -> Self {
        match value {
            ButtonSize::Sm => ComponentSize::Sm,
            ButtonSize::Md => ComponentSize::Md,
            ButtonSize::Lg => ComponentSize::Lg,
        }
    }
}

pub struct Button {
    label: String,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    on_press: Option<Box<dyn Fn()>>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            disabled: false,
            on_press: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }

    pub fn build(self) -> impl IntoView {
        let initial = current_resolved_theme().button_recipe(self.size.into(), self.variant);
        let recipe = RwSignal::new(initial);
        let label = self.label;
        let variant = self.variant;
        let size = self.size;
        let disabled = self.disabled;
        let button = views::Button::new(label)
            .disabled(move || disabled)
            .style(move |s| {
                let recipe = recipe.get();
                s.background(to_color(recipe.background))
                    .color(to_color(recipe.foreground))
                    .border(1.0)
                    .border_color(to_color(recipe.border))
                    .padding_left(recipe.padding_x)
                    .padding_right(recipe.padding_x)
                    .padding_top(recipe.padding_y)
                    .padding_bottom(recipe.padding_y)
                    .border_radius(recipe.radius)
                    .hover(|s| s.background(to_color(recipe.hover_background)))
                    .active(|s| s.background(to_color(recipe.active_background)))
                    .focus_visible(|s| s.outline(2.0).outline_color(to_color(recipe.ring)))
            });
        let button = if let Some(on_press) = self.on_press {
            button.action(on_press)
        } else {
            button
        };
        ThemeBind::new(button, move |theme: ResolvedTheme| {
            recipe.set(theme.button_recipe(size.into(), variant));
        })
    }
}
