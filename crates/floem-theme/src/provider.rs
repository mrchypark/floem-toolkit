use crate::{merge_theme, ResolvedTheme, ThemeDefinition, ThemePatch};
use floem::prelude::*;
use floem::reactive::{create_effect, provide_context, use_context, RwSignal};

#[derive(Clone, Copy)]
pub struct ThemeContext(pub RwSignal<ThemeDefinition>);

pub struct ThemeProvider;

impl ThemeProvider {
    pub fn wrap<V: IntoView + 'static>(
        child: impl FnOnce() -> V,
        definition: impl Fn() -> ThemeDefinition + 'static,
    ) -> impl IntoView {
        let signal = RwSignal::new(definition());
        create_effect(move |_| {
            signal.set(definition());
        });
        with_theme_context(signal, child)
    }
}

pub struct ThemeScope;

impl ThemeScope {
    pub fn wrap<V: IntoView + 'static>(
        child: impl FnOnce() -> V,
        patch: ThemePatch,
    ) -> impl IntoView {
        let parent = current_theme_signal();
        let signal = RwSignal::new(merge_theme(&parent.get_untracked(), &patch));
        create_effect(move |_| {
            signal.set(merge_theme(&parent.get(), &patch));
        });
        with_theme_context(signal, child)
    }
}

fn with_theme_context<V: IntoView + 'static>(
    signal: RwSignal<ThemeDefinition>,
    child: impl FnOnce() -> V,
) -> impl IntoView {
    let previous = use_context::<ThemeContext>();
    provide_context(ThemeContext(signal));
    let built = child();
    if let Some(previous) = previous {
        provide_context(previous);
    } else {
        provide_context(ThemeContext(RwSignal::new(ThemeDefinition::default())));
    }
    built
}

pub fn current_theme_signal() -> RwSignal<ThemeDefinition> {
    use_context::<ThemeContext>()
        .map(|context| context.0)
        .unwrap_or_else(|| RwSignal::new(ThemeDefinition::default()))
}

pub fn current_resolved_theme() -> ResolvedTheme {
    resolved_from_definition(&current_theme_signal().get_untracked())
}

pub fn resolved_from_definition(definition: &ThemeDefinition) -> ResolvedTheme {
    ResolvedTheme::from_definition(definition.clone())
}
