mod patch;
mod provider;
mod recipes;
mod resolved;
mod theme;

pub use patch::{merge_theme, Patch, ThemePatch};
pub use provider::{
    current_resolved_theme, current_theme_signal, resolved_from_definition, ThemeContext,
    ThemeProvider, ThemeScope,
};
pub use recipes::*;
pub use resolved::ResolvedTheme;
pub use theme::{ColorDerivation, ThemeDefinition};

#[cfg(test)]
mod tests {
    use crate::{
        current_theme_signal, merge_theme, resolved_from_definition, ButtonVariant,
        ColorDerivation, ComponentSize, Patch, ThemeDefinition, ThemePatch, ThemeProvider,
        ThemeScope,
    };
    use floem::prelude::SignalGet;
    use floem::views;
    use floem_tokens::ThemeMode;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn theme_patch_replaces_lists() {
        let base = ThemeDefinition::default();
        let patch = ThemePatch {
            chart_series: Some(vec![
                floem_tokens::TokenSet::new(base.mode)
                    .semantic
                    .colors
                    .primary,
            ]),
            ..ThemePatch::default()
        };
        let merged = merge_theme(&base, &patch);
        assert_eq!(merged.chart_series.len(), 1);
    }

    #[test]
    fn recipe_source_is_serializable_theme() {
        let theme = ThemeDefinition::new(ThemeMode::Light);
        assert_eq!(theme.color_derivation, ColorDerivation::Hybrid);
    }

    #[test]
    fn resolved_theme_rebuilds_from_mode() {
        let light = ThemeDefinition::new(ThemeMode::Light);
        let dark = ThemeDefinition::new(ThemeMode::Dark);
        let light_resolved = resolved_from_definition(&light);
        let dark_resolved = resolved_from_definition(&dark);
        assert_ne!(
            light_resolved.token_set.semantic.colors.background,
            dark_resolved.token_set.semantic.colors.background
        );
    }

    #[test]
    fn theme_patch_applies_semantic_override() {
        let base = ThemeDefinition::default();
        let patch = ThemePatch {
            ring: crate::Patch::Set(floem_tokens::ColorScale::rgb(1, 2, 3)),
            ..ThemePatch::default()
        };
        let merged = merge_theme(&base, &patch);
        let resolved = resolved_from_definition(&merged);
        assert_eq!(
            resolved.token_set.semantic.colors.ring,
            floem_tokens::ColorScale::rgb(1, 2, 3)
        );
        assert_eq!(
            resolved.chart_theme().tooltip.background,
            resolved.token_set.chart.tooltip_bg
        );
    }

    #[test]
    fn chart_derivation_recomputes_from_semantic_overrides() {
        let base = ThemeDefinition::default();
        let patch = ThemePatch {
            foreground: crate::Patch::Set(floem_tokens::ColorScale::rgb(10, 20, 30)),
            popover: crate::Patch::Set(floem_tokens::ColorScale::rgb(40, 50, 60)),
            ring: crate::Patch::Set(floem_tokens::ColorScale::rgb(70, 80, 90)),
            ..ThemePatch::default()
        };
        let merged = merge_theme(&base, &patch);
        let resolved = resolved_from_definition(&merged);
        assert_eq!(
            resolved.token_set.chart.axis,
            floem_tokens::alpha(floem_tokens::ColorScale::rgb(10, 20, 30), 0.72)
        );
        assert_eq!(
            resolved.token_set.chart.grid,
            floem_tokens::alpha(floem_tokens::ColorScale::rgb(10, 20, 30), 0.18)
        );
        assert_eq!(
            resolved.token_set.chart.tooltip_bg,
            floem_tokens::ColorScale::rgb(40, 50, 60)
        );
        assert_eq!(
            resolved.token_set.chart.tooltip_text,
            floem_tokens::ColorScale::rgb(10, 20, 30)
        );
        assert_eq!(
            resolved.token_set.chart.selection,
            floem_tokens::alpha(floem_tokens::ColorScale::rgb(70, 80, 90), 0.25)
        );
    }

    #[test]
    fn theme_patch_can_clear_exception_and_switch_mode() {
        let base = merge_theme(
            &ThemeDefinition::new(ThemeMode::Dark),
            &ThemePatch {
                card: crate::Patch::Set(floem_tokens::ColorScale::rgb(1, 2, 3)),
                ..ThemePatch::default()
            },
        );
        let patch = ThemePatch {
            mode: Some(ThemeMode::Light),
            card: crate::Patch::Clear,
            ..ThemePatch::default()
        };
        let merged = merge_theme(&base, &patch);
        assert_eq!(merged.mode, ThemeMode::Light);
        assert_eq!(merged.exceptions.card, None);
        assert_eq!(merged.chart_series, base.chart_series);
        let resolved = resolved_from_definition(&merged);
        assert_eq!(
            resolved.token_set.semantic.colors.card,
            floem_tokens::TokenSet::new(ThemeMode::Light)
                .semantic
                .colors
                .card
        );
    }

    #[test]
    fn theme_provider_and_scope_expose_context_during_build() {
        let provider_seen = Rc::new(RefCell::new(None));
        let scoped_seen = Rc::new(RefCell::new(None));
        let restored_seen = Rc::new(RefCell::new(None));
        let patch = ThemePatch {
            primary: Patch::Set(floem_tokens::ColorScale::rgb(9, 99, 88)),
            ..ThemePatch::default()
        };

        let provider_seen_for_closure = provider_seen.clone();
        let scoped_seen_for_closure = scoped_seen.clone();
        let restored_seen_for_closure = restored_seen.clone();

        let _ = ThemeProvider::wrap(
            move || {
                *provider_seen_for_closure.borrow_mut() =
                    Some(current_theme_signal().get_untracked());
                let _scope = ThemeScope::wrap(
                    move || {
                        *scoped_seen_for_closure.borrow_mut() =
                            Some(current_theme_signal().get_untracked());
                        views::empty()
                    },
                    patch,
                );
                *restored_seen_for_closure.borrow_mut() =
                    Some(current_theme_signal().get_untracked());
                views::empty()
            },
            || ThemeDefinition::new(ThemeMode::Light),
        );

        let provider = provider_seen.borrow().clone().expect("provider theme");
        let scoped = scoped_seen.borrow().clone().expect("scoped theme");
        let restored = restored_seen.borrow().clone().expect("restored theme");
        assert_eq!(provider.mode, ThemeMode::Light);
        assert_eq!(restored.mode, ThemeMode::Light);
        assert_eq!(
            scoped.exceptions.primary,
            Some(floem_tokens::ColorScale::rgb(9, 99, 88))
        );
    }

    #[test]
    fn dark_surface_buttons_use_tinted_hover_not_opaque_fill() {
        let resolved = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));
        let outline = resolved.button_recipe(ComponentSize::Md, ButtonVariant::Outline);
        let ghost = resolved.button_recipe(ComponentSize::Md, ButtonVariant::Ghost);
        let secondary = resolved.button_recipe(ComponentSize::Md, ButtonVariant::Secondary);

        assert!(outline.hover_background.a < 255);
        assert!(ghost.hover_background.a < 255);
        assert!(secondary.hover_background.a < 255);
        assert!(outline.active_background.a > outline.hover_background.a);
        assert!(ghost.active_background.a > ghost.hover_background.a);
        assert!(secondary.active_background.a > secondary.hover_background.a);
    }

    #[test]
    fn input_recipe_uses_fixed_single_line_metrics() {
        let resolved = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));
        let recipe = resolved.input_recipe(ComponentSize::Md, false);

        assert_eq!(recipe.font_size, 14.0);
        assert_eq!(recipe.line_height, 20.0);
        assert_eq!(recipe.padding_y, 8.0);
    }

    #[test]
    fn input_interaction_helpers_are_mode_sensitive() {
        let light = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Light));
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));

        assert_ne!(
            light.input_hover_border(false),
            dark.input_hover_border(false)
        );
        assert_ne!(light.input_focus_ring(), dark.input_focus_ring());
    }

    #[test]
    fn checkbox_interaction_helpers_keep_dark_hover_translucent() {
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));
        let base = dark.checkbox_recipe();
        let hover = dark.checkbox_hover_background(false, &base);
        let active = dark.checkbox_active_background(false, &base);

        assert!(hover.a < 255);
        assert!(active.a < 255);
        assert!(active.a > hover.a);
    }

    #[test]
    fn tabs_interaction_helpers_keep_dark_inactive_hover_translucent() {
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));
        let base = dark.tabs_recipe();
        let hover = dark.tab_hover_background(false, &base);
        let selected_border = dark.tab_selected_border(true, &base);

        assert!(hover.a < 255);
        assert!(selected_border.a < 255);
    }

    #[test]
    fn select_interaction_helpers_follow_input_focus_model() {
        let light = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Light));
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));

        assert_eq!(light.select_focus_ring(), light.input_focus_ring());
        assert_eq!(dark.select_focus_ring(), dark.input_focus_ring());
        assert_ne!(light.select_hover_border(), dark.select_hover_border());
    }

    #[test]
    fn overlay_scrim_is_stronger_in_dark_mode() {
        let light = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Light));
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));

        assert!(dark.dialog_overlay_scrim().a > light.dialog_overlay_scrim().a);
        assert_eq!(light.dialog_recipe().overlay, light.dialog_overlay_scrim());
        assert_eq!(dark.dialog_recipe().overlay, dark.dialog_overlay_scrim());
    }

    #[test]
    fn chart_interaction_helpers_are_mode_sensitive() {
        let light = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Light));
        let dark = resolved_from_definition(&ThemeDefinition::new(ThemeMode::Dark));

        assert_ne!(light.chart_crosshair(), dark.chart_crosshair());
        assert_ne!(light.chart_brush_fill(), dark.chart_brush_fill());
        assert_ne!(light.chart_series_dim(0), dark.chart_series_dim(0));
        assert_eq!(light.chart_series_hover(0).a, 255);
    }
}
