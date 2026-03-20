use floem::prelude::*;
use floem::reactive::create_effect;
use floem_theme::{current_theme_signal, resolved_from_definition, ResolvedTheme};

pub struct ThemeBind;

impl ThemeBind {
    pub fn new<V: IntoView + 'static>(
        child: V,
        on_change: impl Fn(ResolvedTheme) + 'static,
    ) -> impl IntoView {
        let signal = current_theme_signal();
        create_effect(move |_| {
            on_change(resolved_from_definition(&signal.get()));
        });
        child
    }
}
