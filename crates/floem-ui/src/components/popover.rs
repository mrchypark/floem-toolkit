use crate::primitives::theme_bind::ThemeBind;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::current_resolved_theme;
use std::rc::Rc;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub struct Popover {
    open: bool,
    state: Option<RwSignal<bool>>,
    on_open_change: Option<Box<dyn Fn(bool)>>,
    trigger: Option<floem::AnyView>,
    content: Option<floem::AnyView>,
}

impl Popover {
    pub fn new() -> Self {
        Self {
            open: false,
            state: None,
            on_open_change: None,
            trigger: None,
            content: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn bind_open(mut self, state: RwSignal<bool>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn on_open_change(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_open_change = Some(Box::new(handler));
        self
    }

    pub fn trigger(mut self, view: impl IntoView + 'static) -> Self {
        self.trigger = Some(view.into_any());
        self
    }

    pub fn content(mut self, view: impl IntoView + 'static) -> Self {
        self.content = Some(view.into_any());
        self
    }

    pub fn build(self) -> impl IntoView {
        let recipe = RwSignal::new(current_resolved_theme().popover_recipe());
        let open = self.state.unwrap_or_else(|| RwSignal::new(self.open));
        let on_open_change = self.on_open_change.map(Rc::new);

        let trigger = self
            .trigger
            .unwrap_or_else(|| views::Button::new("Open popover").into_any());
        let content = self
            .content
            .unwrap_or_else(|| views::label(|| "Popover content".to_string()).into_any());

        let trigger_button = views::container(trigger)
            .on_click_stop({
                let open = open;
                let on_open_change = on_open_change.clone();
                move |_| {
                    let next = !open.get_untracked();
                    open.set(next);
                    if let Some(on_open_change) = on_open_change.as_ref() {
                        on_open_change(next);
                    }
                }
            })
            .style(|s| s.width(180.0));

        let panel = views::container(content)
            .on_event_stop(EventListener::PointerDown, |_| {})
            .style(move |s| {
                let recipe = recipe.get();
                s.background(to_color(recipe.panel.background))
                    .color(to_color(recipe.panel.foreground))
                    .border(1.0)
                    .border_color(to_color(recipe.panel.border))
                    .border_radius(recipe.panel.radius)
                    .padding(recipe.panel.padding)
                    .margin_top(8.0)
                    .width(320.0)
                    .apply_if(!open.get(), |s| s.hide())
            });

        let popover = (trigger_button, panel)
            .v_stack()
            .keyboard_navigable()
            .request_focus(move || {
                if open.get() {
                    ()
                }
            })
            .on_event_stop(EventListener::FocusLost, {
                let open = open;
                let on_open_change = on_open_change.clone();
                move |_| {
                    if open.get_untracked() {
                        open.set(false);
                        if let Some(on_open_change) = on_open_change.as_ref() {
                            on_open_change(false);
                        }
                    }
                }
            })
            .on_event_stop(EventListener::KeyDown, {
                let open = open;
                let on_open_change = on_open_change.clone();
                move |event| {
                    if let Event::KeyDown(event) = event {
                        if event.key.logical_key == Key::Named(NamedKey::Escape)
                            && open.get_untracked()
                        {
                            open.set(false);
                            if let Some(on_open_change) = on_open_change.as_ref() {
                                on_open_change(false);
                            }
                        }
                    }
                }
            });

        ThemeBind::new(popover, move |theme| {
            recipe.set(theme.popover_recipe());
        })
    }
}
