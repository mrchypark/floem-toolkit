#![deny(missing_docs)]
//! A checkbox view for boolean selection.

use crate::{
    style_class,
    view::{IntoView, View},
    views::{
        self, container, create_value_container_signals, h_stack, svg, value_container, Decorators,
        ValueContainer,
    },
};
use floem_reactive::{SignalGet, SignalUpdate};
use std::fmt::Display;

style_class!(
    /// The style class that is applied to the checkbox.
    pub CheckboxClass
);

style_class!(
    /// The style class that is applied to the labeled checkbox stack.
    pub LabeledCheckboxClass
);

style_class!(
    /// The style class that is applied to the checkmark inside a checkbox.
    pub CheckboxMarkClass
);

const CHECKBOX_MARK_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 16 16" fill="none"><path d="M13.2 4.8L6.5 11.5L2.8 7.8" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

fn checkbox_mark(checked: impl SignalGet<bool> + 'static) -> impl View {
    svg(CHECKBOX_MARK_SVG)
        .class(CheckboxMarkClass)
        .style(move |s| s.apply_if(!checked.get(), |s| s.display(taffy::style::Display::None)))
}

fn checkbox_control(checked: impl SignalGet<bool> + 'static) -> impl View {
    container(checkbox_mark(checked))
        .class(CheckboxClass)
        .keyboard_navigable()
        .style(|s| s.items_center().justify_center())
}

/// # A customizable checkbox view for boolean selection.
///
/// The `Checkbox` struct provides several constructors, each offering different levels of
/// customization and ease of use. The simplest is the [Checkbox::new_rw] constructor, which gets direct access to a signal and will update it when the checkbox is clicked.
///
/// Choose the constructor that best fits your needs based on whether you require labeling
/// and how you prefer to manage the checkbox's state (via closure or direct signal manipulation).
pub struct Checkbox;

impl Checkbox {
    /// Creates a new checkbox with a closure that determines its checked state.
    ///
    /// This method is useful when you want to create a checkbox whose state is determined by a closure.
    /// The state can be dynamically updated by the closure, and the checkbox will reflect these changes.
    ///
    /// You can add an `on_update` handler to the returned `ValueContainer` to handle changes.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(checked: impl Fn() -> bool + 'static) -> ValueContainer<bool> {
        let (inbound_signal, outbound_signal) = create_value_container_signals(checked);

        value_container(
            checkbox_control(inbound_signal.read_only()).on_click_stop(move |_| {
                let checked = inbound_signal.get_untracked();
                outbound_signal.set(!checked);
            }),
            move || outbound_signal.get(),
        )
    }

    /// Creates a new checkbox with a signal that provides and updates its checked state.
    ///
    /// This method is ideal when you need a checkbox that not only reflects a signal's state but also updates it.
    /// Clicking the checkbox will toggle its state and update the signal accordingly.
    pub fn new_rw(
        checked: impl SignalGet<bool> + SignalUpdate<bool> + Copy + 'static,
    ) -> impl IntoView {
        checkbox_control(checked).on_click_stop(move |_| {
            checked.update(|val| *val = !*val);
        })
    }

    /// Creates a new labeled checkbox with a closure that determines its checked state.
    ///
    /// This method is useful when you want a labeled checkbox whose state is determined by a closure.
    /// The label is also provided by a closure, allowing for dynamic updates.
    pub fn labeled<S: Display + 'static>(
        checked: impl Fn() -> bool + 'static,
        label: impl Fn() -> S + 'static,
    ) -> ValueContainer<bool> {
        let (inbound_signal, outbound_signal) = create_value_container_signals(checked);

        value_container(
            h_stack((
                checkbox_control(inbound_signal.read_only()),
                views::label(label),
            ))
            .class(LabeledCheckboxClass)
            .on_click_stop(move |_| {
                let checked = inbound_signal.get_untracked();
                outbound_signal.set(!checked);
            })
            .style(|s| s.items_center().justify_center()),
            move || outbound_signal.get(),
        )
    }

    /// Creates a new labeled checkbox with a signal that provides and updates its checked state.
    ///
    /// This method is ideal when you need a labeled checkbox that not only reflects a signal's state but also updates it.
    /// Clicking the checkbox will toggle its state and update the signal accordingly.
    pub fn labeled_rw<S: Display + 'static>(
        checked: impl SignalGet<bool> + SignalUpdate<bool> + Copy + 'static,
        label: impl Fn() -> S + 'static,
    ) -> impl IntoView {
        h_stack((checkbox_control(checked), views::label(label)))
            .class(LabeledCheckboxClass)
            .style(|s| s.items_center().justify_center())
            .on_click_stop(move |_| {
                checked.update(|val| *val = !*val);
            })
    }
}

/// Renders a checkbox the provided checked signal. See also [`Checkbox::new`] and [`Checkbox::new_rw`].
pub fn checkbox(checked: impl Fn() -> bool + 'static) -> ValueContainer<bool> {
    Checkbox::new(checked)
}

/// Renders a checkbox using the provided checked signal. See also [`Checkbox::labeled`] and [`Checkbox::labeled_rw`].
pub fn labeled_checkbox<S: Display + 'static>(
    checked: impl Fn() -> bool + 'static,
    label: impl Fn() -> S + 'static,
) -> ValueContainer<bool> {
    Checkbox::labeled(checked, label)
}

#[cfg(test)]
mod tests {
    use super::CHECKBOX_MARK_SVG;
    use floem_renderer::usvg;

    #[test]
    fn checkbox_mark_svg_parses() {
        assert!(usvg::Tree::from_str(CHECKBOX_MARK_SVG, &usvg::Options::default()).is_ok());
    }

    #[test]
    fn checkbox_mark_svg_uses_current_color() {
        assert!(CHECKBOX_MARK_SVG.contains("currentColor"));
    }
}
