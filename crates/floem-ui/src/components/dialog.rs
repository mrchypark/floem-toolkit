use crate::components::Card;
use crate::primitives::theme_bind::ThemeBind;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;
use floem_theme::current_resolved_theme;

fn to_color(color: floem_tokens::ColorScale) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

pub struct Dialog {
    open: bool,
    state: Option<RwSignal<bool>>,
    on_open_change: Option<Box<dyn Fn(bool)>>,
    title: Option<floem::AnyView>,
    content: Option<floem::AnyView>,
    footer: Option<floem::AnyView>,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            open: false,
            state: None,
            on_open_change: None,
            title: None,
            content: None,
            footer: None,
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

    pub fn title(mut self, view: impl IntoView + 'static) -> Self {
        self.title = Some(view.into_any());
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
        let recipe = RwSignal::new(current_resolved_theme().dialog_recipe());
        let open = self.state.unwrap_or_else(|| RwSignal::new(self.open));
        let on_open_change = self.on_open_change.map(std::rc::Rc::new);

        let close = {
            let open = open;
            let on_open_change = on_open_change.clone();
            move || {
                open.set(false);
                if let Some(on_open_change) = on_open_change.as_ref() {
                    on_open_change(false);
                }
            }
        };

        let mut card = Card::new();
        if let Some(title) = self.title {
            card = card.header(title);
        }
        if let Some(content) = self.content {
            card = card.content(content);
        }
        if let Some(footer) = self.footer {
            card = card.footer(
                (footer, views::Button::new("Close").action(close))
                    .h_stack()
                    .style(|s| s.justify_between().column_gap(12.0)),
            );
        } else {
            card = card.footer(
                views::container(views::Button::new("Close").action(close))
                    .style(|s| s.justify_end()),
            );
        }

        let dialog = views::container(
            views::container(card.build())
                .on_event_stop(EventListener::PointerDown, |_| {})
                .style(|s| s.width(480.0).max_width_full()),
        )
        .keyboard_navigable()
        .request_focus(move || {
            if open.get() {
                ()
            }
        })
        .on_click_stop({
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
                    if event.key.logical_key == Key::Named(NamedKey::Escape) && open.get_untracked()
                    {
                        open.set(false);
                        if let Some(on_open_change) = on_open_change.as_ref() {
                            on_open_change(false);
                        }
                    }
                }
            }
        })
        .style(move |s| {
            let recipe = recipe.get();
            s.absolute()
                .inset(0.0)
                .items_center()
                .justify_center()
                .background(to_color(recipe.overlay))
                .padding(24.0)
                .apply_if(!open.get(), |s| s.hide())
        });

        ThemeBind::new(dialog, move |theme| {
            recipe.set(theme.dialog_recipe());
        })
    }
}
