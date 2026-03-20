use crate::primitives::theme_bind::ThemeBind;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::{current_resolved_theme, ResolvedTheme};
use std::rc::Rc;
use std::sync::Arc;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub type SharedString = Arc<str>;

#[derive(Clone)]
pub struct TabSpec {
    pub id: SharedString,
    pub label: SharedString,
}

impl TabSpec {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

pub struct Tabs {
    value: SharedString,
    state: Option<RwSignal<SharedString>>,
    on_value_change: Option<Box<dyn Fn(SharedString)>>,
    tabs: Vec<TabSpec>,
    panels: Vec<(SharedString, floem::AnyView)>,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            value: Arc::<str>::from(""),
            state: None,
            on_value_change: None,
            tabs: Vec::new(),
            panels: Vec::new(),
        }
    }

    pub fn value(mut self, value: SharedString) -> Self {
        self.value = value;
        self
    }

    pub fn bind_value(mut self, state: RwSignal<SharedString>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn on_value_change(mut self, handler: impl Fn(SharedString) + 'static) -> Self {
        self.on_value_change = Some(Box::new(handler));
        self
    }

    pub fn tabs(mut self, tabs: Vec<TabSpec>) -> Self {
        self.tabs = tabs;
        self
    }

    pub fn panel(mut self, id: SharedString, child: impl IntoView + 'static) -> Self {
        self.panels.push((id, child.into_any()));
        self
    }

    pub fn build(self) -> impl IntoView {
        let theme = RwSignal::new(current_resolved_theme());
        let recipe = RwSignal::new(theme.get_untracked().tabs_recipe());
        let current = self
            .state
            .unwrap_or_else(|| RwSignal::new(self.value.clone()));
        let on_value_change = self.on_value_change.map(Rc::new);

        let triggers = self
            .tabs
            .into_iter()
            .map(|tab| {
                let current_for_style = current;
                let current_for_click = current;
                let recipe = recipe;
                let theme = theme;
                let on_value_change = on_value_change.clone();
                let id_for_click = tab.id.clone();
                let id_for_style = tab.id.clone();
                views::Button::new(tab.label.to_string())
                    .action(move || {
                        current_for_click.set(id_for_click.clone());
                        if let Some(on_value_change) = on_value_change.as_ref() {
                            on_value_change(id_for_click.clone());
                        }
                    })
                    .style(move |s| {
                        let recipe = recipe.get();
                        let theme = theme.get();
                        let active = current_for_style.get() == id_for_style;
                        s.background(to_color(if active {
                            recipe.tab_active_background
                        } else {
                            recipe.tab_background
                        }))
                        .color(to_color(if active {
                            recipe.tab_active_foreground
                        } else {
                            recipe.tab_foreground
                        }))
                        .border(1.0)
                        .border_color(to_color(theme.tab_selected_border(active, &recipe)))
                        .border_radius(8.0)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .padding_top(8.0)
                        .padding_bottom(8.0)
                        .hover(|s| {
                            s.background(to_color(theme.tab_hover_background(active, &recipe)))
                        })
                        .focus_visible(|s| {
                            s.outline(2.0)
                                .outline_color(to_color(theme.tab_focus_ring()))
                        })
                    })
                    .into_any()
            })
            .collect::<Vec<_>>()
            .h_stack()
            .style(|s| s.column_gap(8.0));

        let panels = self
            .panels
            .into_iter()
            .map(|(id, panel)| {
                let current = current;
                views::container(panel).style(move |s| {
                    s.apply_if(current.get() != id, |s| s.hide())
                        .padding_top(12.0)
                })
            })
            .collect::<Vec<_>>()
            .v_stack();

        let tabs = (triggers, panels).v_stack().style(move |s| {
            let recipe = recipe.get();
            s.border(1.0)
                .border_color(to_color(recipe.border))
                .border_radius(12.0)
                .padding(12.0)
                .row_gap(8.0)
        });

        ThemeBind::new(tabs, move |next_theme: ResolvedTheme| {
            recipe.set(next_theme.tabs_recipe());
            theme.set(next_theme);
        })
    }
}
