use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem_theme::current_resolved_theme;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub struct Card {
    header: Option<floem::AnyView>,
    content: Option<floem::AnyView>,
    footer: Option<floem::AnyView>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            header: None,
            content: None,
            footer: None,
        }
    }

    pub fn header(mut self, view: impl IntoView + 'static) -> Self {
        self.header = Some(view.into_any());
        self
    }

    pub fn content(mut self, view: impl IntoView + 'static) -> Self {
        self.content = Some(view.into_any());
        self
    }

    pub fn footer(mut self, view: impl IntoView + 'static) -> Self {
        self.footer = Some(view.into_any());
        self
    }

    pub fn build(self) -> impl IntoView {
        let recipe = RwSignal::new(current_resolved_theme().card_recipe());
        let mut children = Vec::new();
        if let Some(header) = self.header {
            children.push(header);
        }
        if let Some(content) = self.content {
            children.push(content);
        }
        if let Some(footer) = self.footer {
            children.push(footer);
        }
        let sections = children.v_stack().style(move |s| {
            let recipe = recipe.get();
            s.background(to_color(recipe.background))
                .color(to_color(recipe.foreground))
                .border(1.0)
                .border_color(to_color(recipe.border))
                .border_radius(recipe.radius)
                .padding(recipe.padding)
                .row_gap(recipe.padding / 2.0)
        });
        ThemeBind::new(sections, move |theme| {
            recipe.set(theme.card_recipe());
        })
    }
}
