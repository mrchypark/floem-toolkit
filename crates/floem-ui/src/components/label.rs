use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::current_resolved_theme;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn build(self) -> impl IntoView {
        let recipe = RwSignal::new(current_resolved_theme().label_recipe());
        let text = self.text;
        let label = views::label(move || text.clone()).style(move |s| {
            let recipe = recipe.get();
            s.color(to_color(recipe.foreground))
                .font_size(recipe.font_size)
        });
        ThemeBind::new(label, move |theme| {
            recipe.set(theme.label_recipe());
        })
    }
}
