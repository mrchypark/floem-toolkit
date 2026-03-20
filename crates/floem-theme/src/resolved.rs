use crate::recipes::*;
use crate::theme::ThemeDefinition;
use floem_tokens::{alpha, ThemeMode, TokenSet};

#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub definition: ThemeDefinition,
    pub token_set: TokenSet,
}

impl ResolvedTheme {
    pub fn from_definition(definition: ThemeDefinition) -> Self {
        let mut token_set = TokenSet::new(definition.mode);
        if let Some(background) = definition.exceptions.background {
            token_set.semantic.colors.background = background;
        }
        if let Some(foreground) = definition.exceptions.foreground {
            token_set.semantic.colors.foreground = foreground;
        }
        if let Some(surface) = definition.exceptions.surface {
            token_set.semantic.colors.surface = surface;
        }
        if let Some(card) = definition.exceptions.card {
            token_set.semantic.colors.card = card;
        }
        if let Some(popover) = definition.exceptions.popover {
            token_set.semantic.colors.popover = popover;
        }
        if let Some(border) = definition.exceptions.border {
            token_set.semantic.colors.border = border;
        }
        if let Some(primary) = definition.exceptions.primary {
            token_set.semantic.colors.primary = primary;
        }
        if let Some(secondary) = definition.exceptions.secondary {
            token_set.semantic.colors.secondary = secondary;
        }
        if let Some(muted) = definition.exceptions.muted {
            token_set.semantic.colors.muted = muted;
        }
        if let Some(accent) = definition.exceptions.accent {
            token_set.semantic.colors.accent = accent;
        }
        if let Some(destructive) = definition.exceptions.destructive {
            token_set.semantic.colors.destructive = destructive;
        }
        if let Some(input) = definition.exceptions.input {
            token_set.semantic.colors.input = input;
        }
        if let Some(ring) = definition.exceptions.ring {
            token_set.semantic.colors.ring = ring;
        }
        token_set.chart.series = definition.chart_series.clone();
        token_set.chart.axis = alpha(token_set.semantic.colors.foreground, 0.72);
        token_set.chart.grid = alpha(token_set.semantic.colors.foreground, 0.18);
        token_set.chart.tooltip_bg = token_set.semantic.colors.popover;
        token_set.chart.tooltip_text = token_set.semantic.colors.foreground;
        token_set.chart.selection = alpha(token_set.semantic.colors.ring, 0.25);
        Self {
            definition,
            token_set,
        }
    }

    pub fn mode(&self) -> ThemeMode {
        self.definition.mode
    }

    pub fn button_recipe(&self, size: ComponentSize, variant: ButtonVariant) -> ButtonRecipe {
        let colors = &self.token_set.semantic.colors;
        let base = match variant {
            ButtonVariant::Primary => (colors.primary, colors.background, colors.primary),
            ButtonVariant::Secondary => (colors.secondary, colors.foreground, colors.border),
            ButtonVariant::Outline => (colors.background, colors.foreground, colors.border),
            ButtonVariant::Ghost => (
                alpha(colors.foreground, 0.0),
                colors.foreground,
                alpha(colors.border, 0.0),
            ),
            ButtonVariant::Destructive => {
                (colors.destructive, colors.background, colors.destructive)
            }
        };
        let (padding_x, padding_y) = match size {
            ComponentSize::Sm => (10.0, 6.0),
            ComponentSize::Md => (14.0, 8.0),
            ComponentSize::Lg => (18.0, 10.0),
        };
        ButtonRecipe {
            background: base.0,
            foreground: base.1,
            border: base.2,
            hover_background: self.button_hover_background(variant, base.0),
            active_background: self.button_active_background(variant, base.0),
            ring: colors.ring,
            radius: self.token_set.semantic.radius.md,
            padding_x,
            padding_y,
        }
    }

    pub fn input_recipe(&self, size: ComponentSize, _invalid: bool) -> InputRecipe {
        let colors = &self.token_set.semantic.colors;
        let height = match size {
            ComponentSize::Sm => 32.0,
            ComponentSize::Md => 38.0,
            ComponentSize::Lg => 44.0,
        };
        let (font_size, line_height, padding_y) = match size {
            ComponentSize::Sm => (13.0, 18.0, 6.0),
            ComponentSize::Md => (14.0, 20.0, 8.0),
            ComponentSize::Lg => (15.0, 22.0, 10.0),
        };
        InputRecipe {
            background: colors.input,
            foreground: colors.foreground,
            border: colors.border,
            border_focus: colors.ring,
            border_invalid: colors.destructive,
            placeholder: self.token_set.muted_for(colors.foreground),
            radius: self.token_set.semantic.radius.md,
            height,
            font_size,
            line_height,
            padding_y,
        }
    }

    pub fn label_recipe(&self) -> LabelRecipe {
        LabelRecipe {
            foreground: self.token_set.semantic.colors.foreground,
            font_size: self.token_set.semantic.typography.sm,
        }
    }

    pub fn card_recipe(&self) -> CardRecipe {
        CardRecipe {
            background: self.token_set.semantic.colors.card,
            foreground: self.token_set.semantic.colors.foreground,
            border: self.token_set.semantic.colors.border,
            radius: self.token_set.semantic.radius.lg,
            padding: self.token_set.semantic.spacing.lg,
        }
    }

    pub fn dialog_recipe(&self) -> DialogRecipe {
        DialogRecipe {
            panel: self.card_recipe(),
            overlay: self.dialog_overlay_scrim(),
        }
    }

    pub fn checkbox_recipe(&self) -> CheckboxRecipe {
        CheckboxRecipe {
            background: self.token_set.semantic.colors.input,
            foreground: self.token_set.semantic.colors.foreground,
            border: self.token_set.semantic.colors.border,
            checked: self.token_set.semantic.colors.primary,
            radius: self.token_set.semantic.radius.sm,
        }
    }

    pub fn input_hover_border(&self, invalid: bool) -> floem_tokens::ColorScale {
        if invalid {
            return self.token_set.semantic.colors.destructive;
        }

        match self.mode() {
            ThemeMode::Light => self
                .token_set
                .hover_for(self.token_set.semantic.colors.border),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.24),
        }
    }

    pub fn input_focus_ring(&self) -> floem_tokens::ColorScale {
        match self.mode() {
            ThemeMode::Light => alpha(self.token_set.semantic.colors.ring, 0.22),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.ring, 0.18),
        }
    }

    pub fn checkbox_hover_background(
        &self,
        checked: bool,
        base: &CheckboxRecipe,
    ) -> floem_tokens::ColorScale {
        if checked {
            return match self.mode() {
                ThemeMode::Light => floem_tokens::darken(base.checked, 0.06),
                ThemeMode::Dark => floem_tokens::darken(base.checked, 0.08),
            };
        }

        match self.mode() {
            ThemeMode::Light => self.token_set.hover_for(base.background),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.08),
        }
    }

    pub fn checkbox_active_background(
        &self,
        checked: bool,
        base: &CheckboxRecipe,
    ) -> floem_tokens::ColorScale {
        if checked {
            return match self.mode() {
                ThemeMode::Light => floem_tokens::darken(base.checked, 0.12),
                ThemeMode::Dark => floem_tokens::darken(base.checked, 0.14),
            };
        }

        match self.mode() {
            ThemeMode::Light => self.token_set.active_for(base.background),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.14),
        }
    }

    pub fn checkbox_focus_ring(&self, base: &CheckboxRecipe) -> floem_tokens::ColorScale {
        match self.mode() {
            ThemeMode::Light => alpha(base.checked, 0.26),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.ring, 0.22),
        }
    }

    pub fn tabs_recipe(&self) -> TabsRecipe {
        let colors = &self.token_set.semantic.colors;
        TabsRecipe {
            tab_background: colors.surface,
            tab_foreground: colors.foreground,
            tab_active_background: colors.primary,
            tab_active_foreground: colors.background,
            border: colors.border,
        }
    }

    pub fn tab_hover_background(
        &self,
        active: bool,
        base: &TabsRecipe,
    ) -> floem_tokens::ColorScale {
        if active {
            return self
                .button_hover_background(ButtonVariant::Primary, base.tab_active_background);
        }

        match self.mode() {
            ThemeMode::Light => self.token_set.hover_for(base.tab_background),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.08),
        }
    }

    pub fn tab_focus_ring(&self) -> floem_tokens::ColorScale {
        self.input_focus_ring()
    }

    pub fn tab_selected_border(&self, active: bool, base: &TabsRecipe) -> floem_tokens::ColorScale {
        if active {
            match self.mode() {
                ThemeMode::Light => base.tab_active_background,
                ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.18),
            }
        } else {
            base.border
        }
    }

    pub fn popover_recipe(&self) -> PopoverRecipe {
        PopoverRecipe {
            panel: CardRecipe {
                background: self.token_set.semantic.colors.popover,
                ..self.card_recipe()
            },
            shadow: alpha(self.token_set.semantic.colors.foreground, 0.15),
        }
    }

    pub fn select_recipe(&self) -> SelectRecipe {
        SelectRecipe {
            input: self.input_recipe(ComponentSize::Md, false),
            popover: self.popover_recipe(),
        }
    }

    pub fn select_hover_border(&self) -> floem_tokens::ColorScale {
        self.input_hover_border(false)
    }

    pub fn select_focus_ring(&self) -> floem_tokens::ColorScale {
        self.input_focus_ring()
    }

    pub fn axis_theme(&self) -> AxisTheme {
        AxisTheme {
            color: self.token_set.chart.axis,
        }
    }

    pub fn grid_theme(&self) -> GridTheme {
        GridTheme {
            color: self.token_set.chart.grid,
        }
    }

    pub fn series_palette(&self) -> SeriesPalette {
        SeriesPalette {
            series: self.token_set.chart.series.clone(),
        }
    }

    pub fn chart_tooltip_theme(&self) -> ChartTooltipTheme {
        ChartTooltipTheme {
            background: self.token_set.chart.tooltip_bg,
            foreground: self.token_set.chart.tooltip_text,
        }
    }

    pub fn chart_theme(&self) -> ChartTheme {
        ChartTheme {
            axis: self.axis_theme(),
            grid: self.grid_theme(),
            palette: self.series_palette(),
            tooltip: self.chart_tooltip_theme(),
        }
    }

    pub fn dialog_overlay_scrim(&self) -> floem_tokens::ColorScale {
        match self.mode() {
            ThemeMode::Light => alpha(self.token_set.semantic.colors.foreground, 0.28),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.42),
        }
    }

    pub fn popover_focus_ring(&self) -> floem_tokens::ColorScale {
        self.input_focus_ring()
    }

    pub fn chart_series_hover(&self, series_index: usize) -> floem_tokens::ColorScale {
        self.token_set
            .chart
            .series
            .get(series_index)
            .copied()
            .unwrap_or(self.token_set.semantic.colors.primary)
    }

    pub fn chart_series_dim(&self, series_index: usize) -> floem_tokens::ColorScale {
        let base = self.chart_series_hover(series_index);
        match self.mode() {
            ThemeMode::Light => alpha(base, 0.45),
            ThemeMode::Dark => alpha(base, 0.38),
        }
    }

    pub fn chart_crosshair(&self) -> floem_tokens::ColorScale {
        match self.mode() {
            ThemeMode::Light => alpha(self.token_set.semantic.colors.foreground, 0.28),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.foreground, 0.36),
        }
    }

    pub fn chart_brush_fill(&self) -> floem_tokens::ColorScale {
        match self.mode() {
            ThemeMode::Light => alpha(self.token_set.semantic.colors.ring, 0.16),
            ThemeMode::Dark => alpha(self.token_set.semantic.colors.ring, 0.22),
        }
    }

    fn button_hover_background(
        &self,
        variant: ButtonVariant,
        background: floem_tokens::ColorScale,
    ) -> floem_tokens::ColorScale {
        let colors = &self.token_set.semantic.colors;
        match (self.mode(), variant) {
            (ThemeMode::Dark, ButtonVariant::Primary) => floem_tokens::darken(background, 0.08),
            (ThemeMode::Dark, ButtonVariant::Destructive) => {
                floem_tokens::lighten(background, 0.04)
            }
            (ThemeMode::Dark, ButtonVariant::Secondary) => {
                floem_tokens::alpha(colors.foreground, 0.12)
            }
            (ThemeMode::Dark, ButtonVariant::Outline | ButtonVariant::Ghost) => {
                floem_tokens::alpha(colors.foreground, 0.08)
            }
            _ => self.token_set.hover_for(background),
        }
    }

    fn button_active_background(
        &self,
        variant: ButtonVariant,
        background: floem_tokens::ColorScale,
    ) -> floem_tokens::ColorScale {
        let colors = &self.token_set.semantic.colors;
        match (self.mode(), variant) {
            (ThemeMode::Dark, ButtonVariant::Primary) => floem_tokens::darken(background, 0.14),
            (ThemeMode::Dark, ButtonVariant::Destructive) => {
                floem_tokens::lighten(background, 0.08)
            }
            (ThemeMode::Dark, ButtonVariant::Secondary) => {
                floem_tokens::alpha(colors.foreground, 0.18)
            }
            (ThemeMode::Dark, ButtonVariant::Outline | ButtonVariant::Ghost) => {
                floem_tokens::alpha(colors.foreground, 0.14)
            }
            _ => self.token_set.active_for(background),
        }
    }
}
