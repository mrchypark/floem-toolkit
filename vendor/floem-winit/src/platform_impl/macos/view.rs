#![allow(clippy::unnecessary_cast)]
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ptr::NonNull;

use hangul_cd::string::StringComposer;
use icrate::Foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSCopying, NSMutableAttributedString,
    NSNotFound, NSObject, NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString, NSUInteger,
};
use objc2::declare::{Ivar, IvarDrop};
use objc2::rc::{Id, WeakId};
use objc2::runtime::{AnyObject, Sel};
use objc2::{class, declare_class, msg_send, msg_send_id, mutability, sel, ClassType};

use super::appkit::NSMenu;
use super::{
    appkit::{
        NSApp, NSCursor, NSEvent, NSEventPhase, NSResponder, NSTextInputClient, NSTrackingRectTag,
        NSView,
    },
    event::{code_to_key, code_to_location},
};
use crate::platform_impl::platform::window::position_traffic_lights;
use crate::{
    dpi::{LogicalPosition, LogicalSize},
    event::{
        DeviceEvent, ElementState, Event, Ime, KeyEvent, Modifiers, MouseButton, MouseScrollDelta,
        TouchPhase, WindowEvent,
    },
    keyboard::{Key, KeyCode, KeyLocation, ModifiersState, NamedKey, PhysicalKey},
    platform::macos::{OptionAsAlt, WindowExtMacOS},
    platform::scancode::PhysicalKeyExtScancode,
    platform_impl::platform::{
        app_state::AppState,
        event::{create_key_event, event_mods},
        util,
        window::WinitWindow,
        DEVICE_ID,
    },
    window::WindowId,
};

#[derive(Debug)]
struct CursorState {
    visible: bool,
    cursor: Id<NSCursor>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            visible: true,
            cursor: Default::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
enum ImeState {
    #[default]
    /// The IME events are disabled, so only `ReceivedCharacter` is being sent to the user.
    Disabled,

    /// The ground state of enabled IME input. It means that both Preedit and regular keyboard
    /// input could be start from it.
    Ground,

    /// The IME is in preedit.
    Preedit,

    /// The text was just commited, so the next input from the keyboard must be ignored.
    Commited,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct ModLocationMask: u8 {
        const LEFT     = 0b0001;
        const RIGHT    = 0b0010;
    }
}
impl ModLocationMask {
    fn from_location(loc: KeyLocation) -> ModLocationMask {
        match loc {
            KeyLocation::Left => ModLocationMask::LEFT,
            KeyLocation::Right => ModLocationMask::RIGHT,
            _ => unreachable!(),
        }
    }
}

fn key_to_modifier(key: &Key) -> Option<ModifiersState> {
    match key {
        Key::Named(NamedKey::Alt) => Some(ModifiersState::ALT),
        Key::Named(NamedKey::Control) => Some(ModifiersState::CONTROL),
        Key::Named(NamedKey::Super) => Some(ModifiersState::SUPER),
        Key::Named(NamedKey::Shift) => Some(ModifiersState::SHIFT),
        _ => None,
    }
}

fn get_right_modifier_code(key: &Key) -> KeyCode {
    match key {
        Key::Named(NamedKey::Alt) => KeyCode::AltRight,
        Key::Named(NamedKey::Control) => KeyCode::ControlRight,
        Key::Named(NamedKey::Shift) => KeyCode::ShiftRight,
        Key::Named(NamedKey::Super) => KeyCode::SuperRight,
        _ => unreachable!(),
    }
}

fn get_left_modifier_code(key: &Key) -> KeyCode {
    match key {
        Key::Named(NamedKey::Alt) => KeyCode::AltLeft,
        Key::Named(NamedKey::Control) => KeyCode::ControlLeft,
        Key::Named(NamedKey::Shift) => KeyCode::ShiftLeft,
        Key::Named(NamedKey::Super) => KeyCode::SuperLeft,
        _ => unreachable!(),
    }
}

fn macos_ime_debug_enabled() -> bool {
    std::env::var_os("FLOEM_DEBUG_MACOS_IME").is_some()
}

fn should_suppress_raw_character_input(ime_allowed: bool, input_source: &str) -> bool {
    ime_allowed && input_source.starts_with("com.apple.inputmethod.Korean")
}

fn tab_navigation_ime_state(current_state: ImeState, has_marked_text: bool) -> ImeState {
    if has_marked_text && current_state == ImeState::Preedit {
        ImeState::Ground
    } else {
        current_state
    }
}

fn tab_navigation_key_event() -> KeyEvent {
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::Tab),
        logical_key: Key::Named(NamedKey::Tab),
        text: Some("\t".into()),
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
        platform_specific: super::event::KeyEventExtra {
            text_with_all_modifiers: Some("\t".into()),
            key_without_modifiers: Key::Named(NamedKey::Tab),
        },
    }
}

fn control_text_keyboard_input(text: &str) -> Option<KeyEvent> {
    match text {
        "\t" | "\u{19}" => Some(tab_navigation_key_event()),
        _ => None,
    }
}

pub struct MenuItemAction(Box<dyn Fn(isize)>);

impl std::fmt::Debug for MenuItemAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MenuItemAction")
    }
}

#[derive(Debug, Default)]
pub struct ViewState {
    cursor_state: RefCell<CursorState>,
    ime_position: Cell<LogicalPosition<f64>>,
    ime_size: Cell<LogicalSize<f64>>,
    ime_text: RefCell<String>,
    ime_selection: Cell<NSRange>,
    ime_marked_range: Cell<NSRange>,
    ime_raw_marked_text: RefCell<Option<String>>,
    ime_korean_transition_pending: Cell<bool>,
    modifiers: Cell<Modifiers>,
    phys_modifiers: RefCell<HashMap<Key, ModLocationMask>>,
    tracking_rect: Cell<Option<NSTrackingRectTag>>,
    ime_state: Cell<ImeState>,
    input_source: RefCell<String>,

    /// True iff the application wants IME events.
    ///
    /// Can be set using `set_ime_allowed`
    ime_allowed: Cell<bool>,

    /// True if the current key event should be forwarded
    /// to the application, even during IME
    forward_key_to_app: Cell<bool>,

    context_menu: RefCell<Option<(Id<NSMenu>, NSPoint)>>,

    marked_text: RefCell<Id<NSMutableAttributedString>>,
    accepts_first_mouse: bool,
}

declare_class!(
    #[derive(Debug)]
    #[allow(non_snake_case)]
    pub(crate) struct WinitView {
        // Weak reference because the window keeps a strong reference to the view
        _ns_window: IvarDrop<Box<WeakId<WinitWindow>>, "__ns_window">,
        state: IvarDrop<Box<ViewState>, "_state">,
    }

    mod ivars;

    unsafe impl ClassType for WinitView {
        #[inherits(NSResponder, NSObject)]
        type Super = NSView;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "WinitView";
    }

    unsafe impl WinitView {
        #[method(initWithId:acceptsFirstMouse:)]
        unsafe fn init_with_id(
            this: *mut Self,
            window: &WinitWindow,
            accepts_first_mouse: bool,
        ) -> Option<NonNull<Self>> {
            let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
            this.map(|this| {
                let state = ViewState {
                    accepts_first_mouse,
                    ..Default::default()
                };

                Ivar::write(
                    &mut this._ns_window,
                    Box::new(WeakId::new(&window.retain())),
                );
                Ivar::write(&mut this.state, Box::new(state));

                this.setPostsFrameChangedNotifications(true);

                let notification_center: &AnyObject =
                    unsafe { msg_send![class!(NSNotificationCenter), defaultCenter] };
                // About frame change
                let frame_did_change_notification_name =
                    NSString::from_str("NSViewFrameDidChangeNotification");
                #[allow(clippy::let_unit_value)]
                unsafe {
                    let _: () = msg_send![
                        notification_center,
                        addObserver: &*this,
                        selector: sel!(frameDidChange:),
                        name: &*frame_did_change_notification_name,
                        object: &*this,
                    ];
                }

                *this.state.input_source.borrow_mut() = this.current_input_source();
                NonNull::from(this)
            })
        }
    }

    unsafe impl WinitView {
        #[method(viewDidMoveToWindow)]
        fn view_did_move_to_window(&self) {
            trace_scope!("viewDidMoveToWindow");
            if let Some(tracking_rect) = self.state.tracking_rect.take() {
                self.removeTrackingRect(tracking_rect);
            }

            let rect = self.frame();
            let tracking_rect = self.add_tracking_rect(rect, false);
            self.state.tracking_rect.set(Some(tracking_rect));
        }

        #[method(frameDidChange:)]
        fn frame_did_change(&self, _event: &NSEvent) {
            trace_scope!("frameDidChange:");
            if let Some(tracking_rect) = self.state.tracking_rect.take() {
                self.removeTrackingRect(tracking_rect);
            }

            let rect = self.frame();
            let tracking_rect = self.add_tracking_rect(rect, false);
            self.state.tracking_rect.set(Some(tracking_rect));

            // Emit resize event here rather than from windowDidResize because:
            // 1. When a new window is created as a tab, the frame size may change without a window resize occurring.
            // 2. Even when a window resize does occur on a new tabbed window, it contains the wrong size (includes tab height).
            let logical_size = LogicalSize::new(rect.size.width as f64, rect.size.height as f64);
            let size = logical_size.to_physical::<u32>(self.scale_factor());
            self.queue_event(WindowEvent::Resized(size));
            position_traffic_lights(&self.window());
        }

        #[method(drawRect:)]
        fn draw_rect(&self, rect: NSRect) {
            trace_scope!("drawRect:");

            // It's a workaround for https://github.com/rust-windowing/winit/issues/2640, don't replace with `self.window_id()`.
            if let Some(window) = self._ns_window.load() {
                AppState::handle_redraw(WindowId(window.id()));
            }

            #[allow(clippy::let_unit_value)]
            unsafe {
                let _: () = msg_send![super(self), drawRect: rect];
            }
        }

        #[method(acceptsFirstResponder)]
        fn accepts_first_responder(&self) -> bool {
            trace_scope!("acceptsFirstResponder");
            true
        }

        // This is necessary to prevent a beefy terminal error on MacBook Pros:
        // IMKInputSession [0x7fc573576ff0 presentFunctionRowItemTextInputViewWithEndpoint:completionHandler:] : [self textInputContext]=0x7fc573558e10 *NO* NSRemoteViewController to client, NSError=Error Domain=NSCocoaErrorDomain Code=4099 "The connection from pid 0 was invalidated from this process." UserInfo={NSDebugDescription=The connection from pid 0 was invalidated from this process.}, com.apple.inputmethod.EmojiFunctionRowItem
        // TODO: Add an API extension for using `NSTouchBar`
        #[method_id(touchBar)]
        fn touch_bar(&self) -> Option<Id<NSObject>> {
            trace_scope!("touchBar");
            None
        }

        #[method(resetCursorRects)]
        fn reset_cursor_rects(&self) {
            trace_scope!("resetCursorRects");
            let bounds = self.bounds();
            let cursor_state = self.state.cursor_state.borrow();
            // We correctly invoke `addCursorRect` only from inside `resetCursorRects`
            if cursor_state.visible {
                self.addCursorRect(bounds, &cursor_state.cursor);
            } else {
                self.addCursorRect(bounds, &NSCursor::invisible());
            }
        }

        #[method(showContextMenu:)]
        fn show_context_menu(&self, _n: &AnyObject) {
            if let Some((menu, position)) = self.state.context_menu.borrow_mut().take() {
                menu.popUpMenuPositioningItem(
                    std::ptr::null::<AnyObject>() as *mut _,
                    position,
                    self as *const Self as *mut _,
                );
            }
        }

        #[method(handleMenuItem:)]
        fn handle_menu_item(&self, item: &AnyObject) {
            unsafe {
                let tag: isize = msg_send![item, tag];
                self.queue_event(WindowEvent::MenuAction(tag as usize));
            }
        }
    }

    unsafe impl NSTextInputClient for WinitView {
        #[method(hasMarkedText)]
        fn has_marked_text(&self) -> bool {
            trace_scope!("hasMarkedText");
            self.state.marked_text.borrow().length() > 0
        }

        #[method(markedRange)]
        fn marked_range(&self) -> NSRange {
            trace_scope!("markedRange");
            let range = if self.state.marked_text.borrow().length() > 0 {
                let stored = self.state.ime_marked_range.get();
                if stored.location != NSNotFound as NSUInteger {
                    stored
                } else {
                    NSRange::new(0, self.state.marked_text.borrow().length())
                }
            } else {
                util::EMPTY_RANGE
            };
            self.debug_log_ime(
                "markedRange",
                &format!("range=({}, {})", range.location, range.length),
            );
            range
        }

        #[method(selectedRange)]
        fn selected_range(&self) -> NSRange {
            trace_scope!("selectedRange");
            let range = self.state.ime_selection.get();
            self.debug_log_ime(
                "selectedRange",
                &format!("range=({}, {})", range.location, range.length),
            );
            range
        }

        #[method(setMarkedText:selectedRange:replacementRange:)]
        fn set_marked_text(
            &self,
            string: &NSObject,
            selected_range: NSRange,
            replacement_range: NSRange,
        ) {
            trace_scope!("setMarkedText:selectedRange:replacementRange:");

            // SAFETY: This method is guaranteed to get either a `NSString` or a `NSAttributedString`.
            let (marked_text, preedit_string) = if string.is_kind_of::<NSAttributedString>() {
                let string: *const NSObject = string;
                let string: *const NSAttributedString = string.cast();
                let string = unsafe { &*string };
                (
                    NSMutableAttributedString::from_attributed_nsstring(string),
                    string.string().to_string(),
                )
            } else {
                let string: *const NSObject = string;
                let string: *const NSString = string.cast();
                let string = unsafe { &*string };
                (
                    NSMutableAttributedString::from_nsstring(string),
                    string.to_string(),
                )
            };

            self.debug_log_ime(
                "setMarkedText",
                &format!(
                    "text={:?} selected_range=({}, {}) replacement_range=({}, {})",
                    preedit_string,
                    selected_range.location,
                    selected_range.length,
                    replacement_range.location,
                    replacement_range.length
                ),
            );

            let korean_source = self.suppresses_raw_character_input() && self.state.ime_allowed.get();
            let previous_raw = self.state.ime_raw_marked_text.borrow().clone();
            let current_marked_display = self.state.marked_text.borrow().string().to_string();
            let carryover_commit = korean_carryover_commit_text(
                previous_raw.as_deref(),
                self.state.ime_korean_transition_pending.get(),
                &preedit_string,
                &current_marked_display,
            );
            let preserve_empty_clear = korean_source
                && preedit_string.is_empty()
                && should_preserve_korean_marked_text_on_empty_preedit(
                    previous_raw.as_deref(),
                    &current_marked_display,
                );
            let (marked_text, preedit_string, cursor_range) = if preserve_empty_clear {
                if self.state.ime_korean_transition_pending.get() {
                    self.debug_log_ime("setMarkedText:ignored_transition_clear", "");
                } else {
                    self.debug_log_ime(
                        "setMarkedText:preserve_empty_clear",
                        &current_marked_display,
                    );
                }
                return;
            } else if korean_source
                && is_korean_jamo_sequence(&preedit_string)
                && previous_raw.as_deref().is_some_and(|raw| {
                    raw.chars().count() >= 2 && raw.chars().last() == preedit_string.chars().next()
                })
            {
                self.state.ime_korean_transition_pending.set(true);
                self.debug_log_ime("setMarkedText:ignored_transition_echo", &preedit_string);
                return;
            } else if korean_source && is_korean_jamo_sequence(&preedit_string) {
                let (raw_preedit, display_preedit) =
                    compose_korean_preedit(previous_raw.as_deref(), &preedit_string)
                        .unwrap_or_else(|| (preedit_string.clone(), preedit_string.clone()));
                *self.state.ime_raw_marked_text.borrow_mut() = Some(raw_preedit);
                self.state.ime_korean_transition_pending.set(false);
                let display_ns = NSString::from_str(&display_preedit);
                (
                    NSMutableAttributedString::from_nsstring(&display_ns),
                    display_preedit.clone(),
                    Some((display_preedit.len(), display_preedit.len())),
                )
            } else {
                if let Some(commit_text) = carryover_commit {
                    self.debug_log_ime("setMarkedText:synthesize_carryover_commit", &commit_text);
                    self.queue_event(WindowEvent::Ime(Ime::Commit(commit_text)));
                }
                *self.state.ime_raw_marked_text.borrow_mut() = None;
                self.state.ime_korean_transition_pending.set(false);
                let cursor_range = if preedit_string.is_empty() {
                    None
                } else {
                    let ns_string = NSString::from_str(&preedit_string);
                    let len = ns_string.length();
                    let location = selected_range.location.min(len);
                    let end = selected_range.end().min(len);
                    let lowerbound_utf8 = unsafe { ns_string.substringToIndex(location) }.len();
                    let upperbound_utf8 = unsafe { ns_string.substringToIndex(end) }.len();
                    Some((lowerbound_utf8, upperbound_utf8))
                };
                (marked_text, preedit_string, cursor_range)
            };

            // Update marked text.
            *self.state.marked_text.borrow_mut() = marked_text;

            // Notify IME is active if application still doesn't know it.
            if self.state.ime_state.get() == ImeState::Disabled {
                *self.state.input_source.borrow_mut() = self.current_input_source();
                self.queue_event(WindowEvent::Ime(Ime::Enabled));
            }

            if self.hasMarkedText() {
                self.state.ime_state.set(ImeState::Preedit);
            } else {
                // In case the preedit was cleared, set IME into the Ground state.
                self.state.ime_state.set(ImeState::Ground);
            }

            // Send WindowEvent for updating marked text
            self.queue_event(WindowEvent::Ime(Ime::Preedit(preedit_string, cursor_range)));
        }

        #[method(unmarkText)]
        fn unmark_text(&self) {
            trace_scope!("unmarkText");
            *self.state.marked_text.borrow_mut() = NSMutableAttributedString::new();
            *self.state.ime_raw_marked_text.borrow_mut() = None;
            self.state.ime_korean_transition_pending.set(false);

            let input_context = self.inputContext().expect("input context");
            input_context.discardMarkedText();

            self.queue_event(WindowEvent::Ime(Ime::Preedit(String::new(), None)));
            if self.is_ime_enabled() {
                // Leave the Preedit self.state
                self.state.ime_state.set(ImeState::Ground);
            } else {
                warn!("Expected to have IME enabled when receiving unmarkText");
            }
        }

        #[method_id(validAttributesForMarkedText)]
        fn valid_attributes_for_marked_text(&self) -> Id<NSArray<NSAttributedStringKey>> {
            trace_scope!("validAttributesForMarkedText");
            NSArray::new()
        }

        #[method_id(attributedSubstringForProposedRange:actualRange:)]
        fn attributed_substring_for_proposed_range(
            &self,
            range: NSRange,
            actual_range: *mut NSRange,
        ) -> Option<Id<NSAttributedString>> {
            trace_scope!("attributedSubstringForProposedRange:actualRange:");
            self.debug_log_ime(
                "attributedSubstringForProposedRange",
                &format!("range=({}, {})", range.location, range.length),
            );
            let text = self.state.ime_text.borrow();
            utf16_range_to_byte_range(&text, range).map(|(byte_start, byte_end, actual_range_value)| {
                if !actual_range.is_null() {
                    unsafe {
                        *actual_range = actual_range_value;
                    }
                }
                let substring = NSString::from_str(&text[byte_start..byte_end]);
                NSAttributedString::from_nsstring(&substring)
            })
        }

        #[method(characterIndexForPoint:)]
        fn character_index_for_point(&self, _point: NSPoint) -> NSUInteger {
            trace_scope!("characterIndexForPoint:");
            0
        }

        #[method(firstRectForCharacterRange:actualRange:)]
        fn first_rect_for_character_range(
            &self,
            _range: NSRange,
            _actual_range: *mut NSRange,
        ) -> NSRect {
            trace_scope!("firstRectForCharacterRange:actualRange:");
            let window = self.window();
            let content_rect = window.contentRectForFrameRect(window.frame());
            let base_x = content_rect.origin.x as f64;
            let base_y = (content_rect.origin.y + content_rect.size.height) as f64;
            let x = base_x + self.state.ime_position.get().x;
            let y = base_y - self.state.ime_position.get().y;
            let LogicalSize { width, height } = self.state.ime_size.get();
            NSRect::new(NSPoint::new(x as _, y as _), NSSize::new(width, height))
        }

        #[method(insertText:replacementRange:)]
        fn insert_text(&self, string: &NSObject, replacement_range: NSRange) {
            trace_scope!("insertText:replacementRange:");

            // SAFETY: This method is guaranteed to get either a `NSString` or a `NSAttributedString`.
            let string = if string.is_kind_of::<NSAttributedString>() {
                let string: *const NSObject = string;
                let string: *const NSAttributedString = string.cast();
                unsafe { &*string }.string().to_string()
            } else {
                let string: *const NSObject = string;
                let string: *const NSString = string.cast();
                unsafe { &*string }.to_string()
            };

            let is_control = string.chars().next().map_or(false, |c| c.is_control());
            self.debug_log_ime(
                "insertText",
                &format!(
                    "string={:?} is_control={} replacement_range=({}, {})",
                    string,
                    is_control,
                    replacement_range.location,
                    replacement_range.length
                ),
            );

            if is_control && self.state.ime_allowed.get() && self.hasMarkedText() {
                if let Some(key_event) = control_text_keyboard_input(&string) {
                    self.debug_log_ime(
                        "insertText:control_keyboard_navigation",
                        &format!("string={:?}", string),
                    );
                    queue_tab_navigation_key_down(self, false, key_event);
                    return;
                }
            }

            let standalone_korean_ime_preedit = self.suppresses_raw_character_input()
                && self.state.ime_allowed.get()
                && !self.hasMarkedText()
                && !is_control
                && is_korean_jamo_text(&string);

            if self.state.ime_state.get() == ImeState::Disabled
                && (standalone_korean_ime_preedit
                    || (self.hasMarkedText() && self.state.ime_allowed.get() && !is_control))
            {
                *self.state.input_source.borrow_mut() = self.current_input_source();
                self.queue_event(WindowEvent::Ime(Ime::Enabled));
                self.state.ime_state.set(ImeState::Ground);
            }

            if standalone_korean_ime_preedit {
                let (raw_preedit, display_preedit) =
                    compose_korean_preedit(None, &string)
                        .unwrap_or_else(|| (string.clone(), string.clone()));
                let preedit = NSString::from_str(&display_preedit);
                *self.state.marked_text.borrow_mut() =
                    NSMutableAttributedString::from_nsstring(&preedit);
                *self.state.ime_raw_marked_text.borrow_mut() = Some(raw_preedit);
                self.state.ime_korean_transition_pending.set(false);
                self.queue_event(WindowEvent::Ime(Ime::Preedit(
                    display_preedit.clone(),
                    Some((display_preedit.len(), display_preedit.len())),
                )));
                self.state.ime_state.set(ImeState::Preedit);
                self.debug_log_ime("insertText:preedit", "standalone_korean_ime_preedit=true");
            } else if self.hasMarkedText() && self.state.ime_allowed.get() && !is_control {
                let transition_echo = self.state.ime_korean_transition_pending.get()
                    && is_korean_jamo_text(&string)
                    && self
                        .state
                        .ime_raw_marked_text
                        .borrow()
                        .as_ref()
                        .and_then(|raw| raw.chars().last())
                        == string.chars().next();
                if transition_echo {
                    self.debug_log_ime("insertText:ignored_transition_echo", &string);
                    return;
                }
                *self.state.ime_raw_marked_text.borrow_mut() = None;
                self.state.ime_korean_transition_pending.set(false);
                self.queue_event(WindowEvent::Ime(Ime::Preedit(String::new(), None)));
                self.queue_event(WindowEvent::Ime(Ime::Commit(string)));
                self.state.ime_state.set(ImeState::Commited);
                self.debug_log_ime("insertText:committed", "standalone_korean_ime_preedit=false");
            }
        }

        // Basically, we're sent this message whenever a keyboard event that doesn't generate a "human
        // readable" character happens, i.e. newlines, tabs, and Ctrl+C.
        #[method(doCommandBySelector:)]
        fn do_command_by_selector(&self, _command: Sel) {
            trace_scope!("doCommandBySelector:");
            self.debug_log_ime("doCommandBySelector:start", "selector invoked");
            // We shouldn't forward any character from just commited text, since we'll end up sending
            // it twice with some IMEs like Korean one. We'll also always send `Enter` in that case,
            // which is not desired given it was used to confirm IME input.
            if self.state.ime_state.get() == ImeState::Commited {
                return;
            }

            self.state.forward_key_to_app.set(true);
            self.debug_log_ime("doCommandBySelector:set_forward", "forward_key_to_app=true");

            if self.hasMarkedText() && self.state.ime_state.get() == ImeState::Preedit {
                // Leave preedit so that we also report the key-up for this key.
                self.state.ime_state.set(ImeState::Ground);
            }
        }
    }

    unsafe impl WinitView {
        #[method(keyDown:)]
        fn key_down(&self, event: &NSEvent) {
            trace_scope!("keyDown:");
            {
                let mut prev_input_source = self.state.input_source.borrow_mut();
                let current_input_source = self.current_input_source();
                if *prev_input_source != current_input_source && self.is_ime_enabled() {
                    *prev_input_source = current_input_source;
                    drop(prev_input_source);
                    *self.state.ime_raw_marked_text.borrow_mut() = None;
                    self.state.ime_korean_transition_pending.set(false);
                    self.state.ime_state.set(ImeState::Disabled);
                    self.queue_event(WindowEvent::Ime(Ime::Disabled));
                }
            }

            // Get the characters from the event.
            let old_ime_state = self.state.ime_state.get();
            self.state.forward_key_to_app.set(false);
            let event = replace_event(event, self.window().option_as_alt());
            self.debug_log_ime("keyDown:start", &format!("repeat={}", event.is_a_repeat()));

            // The `interpretKeyEvents` function might call
            // `setMarkedText`, `insertText`, and `doCommandBySelector`.
            // It's important that we call this before queuing the KeyboardInput, because
            // we must send the `KeyboardInput` event during IME if it triggered
            // `doCommandBySelector`. (doCommandBySelector means that the keyboard input
            // is not handled by IME and should be handled by the application)
            if self.state.ime_allowed.get() {
                let events_for_nsview = NSArray::from_slice(&[&*event]);
                unsafe { self.interpretKeyEvents(&events_for_nsview) };
                self.debug_log_ime("keyDown:after_interpret", "interpretKeyEvents finished");

                // If the text was commited we must treat the next keyboard event as IME related.
                if self.state.ime_state.get() == ImeState::Commited {
                    // Remove any marked text, so normal input can continue.
                    *self.state.marked_text.borrow_mut() = NSMutableAttributedString::new();
                }
            }

            self.update_modifiers(&event, false);

            let had_ime_input = match self.state.ime_state.get() {
                ImeState::Commited => {
                    // Allow normal input after the commit.
                    self.state.ime_state.set(ImeState::Ground);
                    true
                }
                ImeState::Preedit => true,
                // `key_down` could result in preedit clear, so compare old and current state.
                _ => old_ime_state != self.state.ime_state.get(),
            };
            self.debug_log_ime(
                "keyDown:after_ime_state",
                &format!("old_ime_state={:?} had_ime_input={}", old_ime_state, had_ime_input),
            );

            if !had_ime_input || self.state.forward_key_to_app.get() {
                let key_event = create_key_event(&event, true, event.is_a_repeat(), None);
                let suppress_raw_character = self.suppresses_raw_character_input()
                    && !self.state.forward_key_to_app.get()
                    && matches!(key_event.logical_key, Key::Character(_));
                self.debug_log_ime(
                    "keyDown:queue_keyboard_input",
                    &format!(
                        "logical={:?} text={:?} suppress_raw_character={}",
                        key_event.logical_key,
                        key_event.text,
                        suppress_raw_character
                    ),
                );
                if !suppress_raw_character {
                    self.queue_event(WindowEvent::KeyboardInput {
                        device_id: DEVICE_ID,
                        event: key_event,
                        is_synthetic: false,
                    });
                }
            }
        }

        #[method(keyUp:)]
        fn key_up(&self, event: &NSEvent) {
            trace_scope!("keyUp:");

            let event = replace_event(event, self.window().option_as_alt());
            self.update_modifiers(&event, false);

            // We want to send keyboard input when we are currently in the ground state.
            if matches!(
                self.state.ime_state.get(),
                ImeState::Ground | ImeState::Disabled
            ) {
                self.queue_event(WindowEvent::KeyboardInput {
                    device_id: DEVICE_ID,
                    event: create_key_event(&event, false, false, None),
                    is_synthetic: false,
                });
            }
        }

        #[method(flagsChanged:)]
        fn flags_changed(&self, event: &NSEvent) {
            trace_scope!("flagsChanged:");

            self.update_modifiers(event, true);
        }

        #[method(insertTab:)]
        fn insert_tab(&self, _sender: Option<&AnyObject>) {
            trace_scope!("insertTab:");
            let window = self.window();
            if let Some(first_responder) = window.firstResponder() {
                if *first_responder == ***self {
                    queue_tab_navigation_key_down(self, false, tab_navigation_key_event());
                }
            }
        }

        #[method(insertBackTab:)]
        fn insert_back_tab(&self, _sender: Option<&AnyObject>) {
            trace_scope!("insertBackTab:");
            let window = self.window();
            if let Some(first_responder) = window.firstResponder() {
                if *first_responder == ***self {
                    queue_tab_navigation_key_down(self, true, tab_navigation_key_event());
                }
            }
        }

        // Allows us to receive Cmd-. (the shortcut for closing a dialog)
        // https://bugs.eclipse.org/bugs/show_bug.cgi?id=300620#c6
        #[method(cancelOperation:)]
        fn cancel_operation(&self, _sender: Option<&AnyObject>) {
            trace_scope!("cancelOperation:");

            let event = NSApp()
                .currentEvent()
                .expect("could not find current event");

            self.update_modifiers(&event, false);
            let event = create_key_event(&event, true, event.is_a_repeat(), None);

            self.queue_event(WindowEvent::KeyboardInput {
                device_id: DEVICE_ID,
                event,
                is_synthetic: false,
            });
        }

        #[method(mouseDown:)]
        fn mouse_down(&self, event: &NSEvent) {
            trace_scope!("mouseDown:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Pressed);
        }

        #[method(mouseUp:)]
        fn mouse_up(&self, event: &NSEvent) {
            trace_scope!("mouseUp:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Released);
        }

        #[method(rightMouseDown:)]
        fn right_mouse_down(&self, event: &NSEvent) {
            trace_scope!("rightMouseDown:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Pressed);
        }

        #[method(rightMouseUp:)]
        fn right_mouse_up(&self, event: &NSEvent) {
            trace_scope!("rightMouseUp:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Released);
        }

        #[method(otherMouseDown:)]
        fn other_mouse_down(&self, event: &NSEvent) {
            trace_scope!("otherMouseDown:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Pressed);
        }

        #[method(otherMouseUp:)]
        fn other_mouse_up(&self, event: &NSEvent) {
            trace_scope!("otherMouseUp:");
            self.mouse_motion(event);
            self.mouse_click(event, ElementState::Released);
        }

        // No tracing on these because that would be overly verbose

        #[method(mouseMoved:)]
        fn mouse_moved(&self, event: &NSEvent) {
            self.mouse_motion(event);
        }

        #[method(mouseDragged:)]
        fn mouse_dragged(&self, event: &NSEvent) {
            self.mouse_motion(event);
        }

        #[method(rightMouseDragged:)]
        fn right_mouse_dragged(&self, event: &NSEvent) {
            self.mouse_motion(event);
        }

        #[method(otherMouseDragged:)]
        fn other_mouse_dragged(&self, event: &NSEvent) {
            self.mouse_motion(event);
        }

        #[method(mouseEntered:)]
        fn mouse_entered(&self, _event: &NSEvent) {
            trace_scope!("mouseEntered:");
            self.queue_event(WindowEvent::CursorEntered {
                device_id: DEVICE_ID,
            });
        }

        #[method(mouseExited:)]
        fn mouse_exited(&self, _event: &NSEvent) {
            trace_scope!("mouseExited:");

            self.queue_event(WindowEvent::CursorLeft {
                device_id: DEVICE_ID,
            });
        }

        #[method(scrollWheel:)]
        fn scroll_wheel(&self, event: &NSEvent) {
            trace_scope!("scrollWheel:");

            self.mouse_motion(event);

            let delta = {
                let (x, y) = (event.scrollingDeltaX(), event.scrollingDeltaY());
                if event.hasPreciseScrollingDeltas() {
                    let delta = LogicalPosition::new(x, y).to_physical(self.scale_factor());
                    MouseScrollDelta::PixelDelta(delta)
                } else {
                    MouseScrollDelta::LineDelta(x as f32, y as f32)
                }
            };

            // The "momentum phase," if any, has higher priority than touch phase (the two should
            // be mutually exclusive anyhow, which is why the API is rather incoherent). If no momentum
            // phase is recorded (or rather, the started/ended cases of the momentum phase) then we
            // report the touch phase.
            let phase = match event.momentumPhase() {
                NSEventPhase::NSEventPhaseMayBegin | NSEventPhase::NSEventPhaseBegan => {
                    TouchPhase::Started
                }
                NSEventPhase::NSEventPhaseEnded | NSEventPhase::NSEventPhaseCancelled => {
                    TouchPhase::Ended
                }
                _ => match event.phase() {
                    NSEventPhase::NSEventPhaseMayBegin | NSEventPhase::NSEventPhaseBegan => {
                        TouchPhase::Started
                    }
                    NSEventPhase::NSEventPhaseEnded | NSEventPhase::NSEventPhaseCancelled => {
                        TouchPhase::Ended
                    }
                    _ => TouchPhase::Moved,
                },
            };

            self.update_modifiers(event, false);

            self.queue_device_event(DeviceEvent::MouseWheel { delta });
            self.queue_event(WindowEvent::MouseWheel {
                device_id: DEVICE_ID,
                delta,
                phase,
            });
        }

        #[method(magnifyWithEvent:)]
        fn magnify_with_event(&self, event: &NSEvent) {
            trace_scope!("magnifyWithEvent:");

            let phase = match event.phase() {
                NSEventPhase::NSEventPhaseBegan => TouchPhase::Started,
                NSEventPhase::NSEventPhaseChanged => TouchPhase::Moved,
                NSEventPhase::NSEventPhaseCancelled => TouchPhase::Cancelled,
                NSEventPhase::NSEventPhaseEnded => TouchPhase::Ended,
                _ => return,
            };

            self.queue_event(WindowEvent::TouchpadMagnify {
                device_id: DEVICE_ID,
                delta: event.magnification(),
                phase,
            });
        }

        #[method(smartMagnifyWithEvent:)]
        fn smart_magnify_with_event(&self, _event: &NSEvent) {
            trace_scope!("smartMagnifyWithEvent:");

            self.queue_event(WindowEvent::SmartMagnify {
                device_id: DEVICE_ID,
            });
        }

        #[method(rotateWithEvent:)]
        fn rotate_with_event(&self, event: &NSEvent) {
            trace_scope!("rotateWithEvent:");

            let phase = match event.phase() {
                NSEventPhase::NSEventPhaseBegan => TouchPhase::Started,
                NSEventPhase::NSEventPhaseChanged => TouchPhase::Moved,
                NSEventPhase::NSEventPhaseCancelled => TouchPhase::Cancelled,
                NSEventPhase::NSEventPhaseEnded => TouchPhase::Ended,
                _ => return,
            };

            self.queue_event(WindowEvent::TouchpadRotate {
                device_id: DEVICE_ID,
                delta: event.rotation(),
                phase,
            });
        }

        #[method(pressureChangeWithEvent:)]
        fn pressure_change_with_event(&self, event: &NSEvent) {
            trace_scope!("pressureChangeWithEvent:");

            self.mouse_motion(event);

            self.queue_event(WindowEvent::TouchpadPressure {
                device_id: DEVICE_ID,
                pressure: event.pressure(),
                stage: event.stage() as i64,
            });
        }

        // Allows us to receive Ctrl-Tab and Ctrl-Esc.
        // Note that this *doesn't* help with any missing Cmd inputs.
        // https://github.com/chromium/chromium/blob/a86a8a6bcfa438fa3ac2eba6f02b3ad1f8e0756f/ui/views/cocoa/bridged_content_view.mm#L816
        #[method(_wantsKeyDownForEvent:)]
        fn wants_key_down_for_event(&self, _event: &NSEvent) -> bool {
            trace_scope!("_wantsKeyDownForEvent:");
            true
        }

        #[method(acceptsFirstMouse:)]
        fn accepts_first_mouse(&self, _event: &NSEvent) -> bool {
            trace_scope!("acceptsFirstMouse:");
            self.state.accepts_first_mouse
        }
    }
);

impl WinitView {
    pub(super) fn new(window: &WinitWindow, accepts_first_mouse: bool) -> Id<Self> {
        unsafe {
            msg_send_id![
                Self::alloc(),
                initWithId: window,
                acceptsFirstMouse: accepts_first_mouse,
            ]
        }
    }

    fn window(&self) -> Id<WinitWindow> {
        // TODO: Simply use `window` property on `NSView`.
        // That only returns a window _after_ the view has been attached though!
        // (which is incompatible with `frameDidChange:`)
        //
        // unsafe { msg_send_id![self, window] }
        self._ns_window.load().expect("view to have a window")
    }

    fn window_id(&self) -> WindowId {
        WindowId(self.window().id())
    }

    fn debug_log_ime(&self, stage: &str, detail: &str) {
        if !macos_ime_debug_enabled() {
            return;
        }
        eprintln!(
            "[floem-winit-macos-ime] stage={} input_source={} ime_state={:?} ime_allowed={} forward_key_to_app={} has_marked_text={} detail={}",
            stage,
            self.state.input_source.borrow().as_str(),
            self.state.ime_state.get(),
            self.state.ime_allowed.get(),
            self.state.forward_key_to_app.get(),
            self.hasMarkedText(),
            detail,
        );
    }

    fn queue_event(&self, event: WindowEvent) {
        let event = Event::WindowEvent {
            window_id: self.window_id(),
            event,
        };
        AppState::queue_event(event);
    }

    fn queue_device_event(&self, event: DeviceEvent) {
        let event = Event::DeviceEvent {
            device_id: DEVICE_ID,
            event,
        };
        AppState::queue_event(event);
    }

    fn scale_factor(&self) -> f64 {
        self.window().backingScaleFactor() as f64
    }

    fn is_ime_enabled(&self) -> bool {
        !matches!(self.state.ime_state.get(), ImeState::Disabled)
    }

    fn suppresses_raw_character_input(&self) -> bool {
        let current_input_source = self.current_input_source();
        let input_source = if current_input_source.is_empty() {
            self.state.input_source.borrow().clone()
        } else {
            current_input_source
        };
        should_suppress_raw_character_input(self.state.ime_allowed.get(), &input_source)
    }

    fn current_input_source(&self) -> String {
        self.inputContext()
            .expect("input context")
            .selectedKeyboardInputSource()
            .map(|input_source| input_source.to_string())
            .unwrap_or_default()
    }

    pub(super) fn set_ime_text_context(
        &self,
        text: String,
        selection: (usize, usize),
        marked_range: Option<(usize, usize)>,
    ) {
        let start = selection.0.min(selection.1);
        let end = selection.0.max(selection.1);
        *self.state.ime_text.borrow_mut() = text;
        self.state.ime_selection.set(NSRange::new(
            start as NSUInteger,
            (end - start) as NSUInteger,
        ));
        let marked = marked_range.map(|(a, b)| (a.min(b), a.max(b)));
        self.state.ime_marked_range.set(match marked {
            Some((start, end)) => NSRange::new(
                start as NSUInteger,
                (end.saturating_sub(start)) as NSUInteger,
            ),
            None => util::EMPTY_RANGE,
        });
        self.debug_log_ime(
            "set_ime_text_context",
            &format!("selection=({}, {}) marked={:?}", start, end - start, marked),
        );
    }

    pub(super) fn set_cursor_icon(&self, icon: Id<NSCursor>) {
        let mut cursor_state = self.state.cursor_state.borrow_mut();
        cursor_state.cursor = icon;
    }

    pub(super) fn set_context_menu(&self, menu: Id<NSMenu>, position: NSPoint) {
        *self.state.context_menu.borrow_mut() = Some((menu, position));
    }

    /// Set whether the cursor should be visible or not.
    ///
    /// Returns whether the state changed.
    pub(super) fn set_cursor_visible(&self, visible: bool) -> bool {
        let mut cursor_state = self.state.cursor_state.borrow_mut();
        if visible != cursor_state.visible {
            cursor_state.visible = visible;
            true
        } else {
            false
        }
    }

    pub(super) fn set_ime_allowed(&self, ime_allowed: bool) {
        if self.state.ime_allowed.get() == ime_allowed {
            return;
        }
        self.state.ime_allowed.set(ime_allowed);
        if self.state.ime_allowed.get() {
            // Refresh the cached input source as soon as the field becomes IME-active.
            // The user may have switched keyboard layouts after launch but before the
            // first keypress, and the startup cache would otherwise be stale.
            *self.state.input_source.borrow_mut() = self.current_input_source();
            return;
        }

        let committed_marked_text = {
            let marked_text = self.state.marked_text.borrow();
            (marked_text.length() > 0).then(|| marked_text.string().to_string())
        }
        .filter(|text| !text.is_empty());

        if let Some(marked_text) = committed_marked_text {
            // AppKit may deliver a trailing commit after focus has already moved. Flush the
            // marked text into the current field before disabling IME so the next responder
            // does not inherit the unfinished syllable.
            self.debug_log_ime("set_ime_allowed:commit_marked_text", &marked_text);
            self.queue_event(WindowEvent::Ime(Ime::Preedit(String::new(), None)));
            self.queue_event(WindowEvent::Ime(Ime::Commit(marked_text)));
        }

        let input_context = self.inputContext().expect("input context");
        input_context.discardMarkedText();

        // Clear markedText
        *self.state.marked_text.borrow_mut() = NSMutableAttributedString::new();
        *self.state.ime_raw_marked_text.borrow_mut() = None;
        self.state.ime_korean_transition_pending.set(false);

        if self.state.ime_state.get() != ImeState::Disabled {
            self.state.ime_state.set(ImeState::Disabled);
            self.queue_event(WindowEvent::Ime(Ime::Disabled));
        }
    }

    pub(super) fn set_ime_cursor_area(
        &self,
        position: LogicalPosition<f64>,
        size: LogicalSize<f64>,
    ) {
        self.state.ime_position.set(position);
        self.state.ime_size.set(size);
        let input_context = self.inputContext().expect("input context");
        input_context.invalidateCharacterCoordinates();
    }

    /// Reset modifiers and emit a synthetic ModifiersChanged event if deemed necessary.
    pub(super) fn reset_modifiers(&self) {
        if !self.state.modifiers.get().state().is_empty() {
            self.state.modifiers.set(Modifiers::default());
            self.queue_event(WindowEvent::ModifiersChanged(self.state.modifiers.get()));
        }
    }

    /// Update modifiers if `event` has something different
    fn update_modifiers(&self, ns_event: &NSEvent, is_flags_changed_event: bool) {
        use ElementState::{Pressed, Released};

        let current_modifiers = event_mods(ns_event);
        let prev_modifiers = self.state.modifiers.get();
        self.state.modifiers.set(current_modifiers);

        // This function was called form the flagsChanged event, which is triggered
        // when the user presses/releases a modifier even if the same kind of modifier
        // has already been pressed.
        //
        // When flags changed event has key code of zero it means that event doesn't carry any key
        // event, thus we can't generate regular presses based on that. The `ModifiersChanged`
        // later will work though, since the flags are attached to the event and contain valid
        // information.
        'send_event: {
            if is_flags_changed_event && ns_event.key_code() != 0 {
                let scancode = ns_event.key_code();
                let physical_key = PhysicalKey::from_scancode(scancode as u32);

                // We'll correct the `is_press` later.
                let mut event = create_key_event(ns_event, false, false, Some(physical_key));

                let key = code_to_key(physical_key, scancode);
                // Ignore processing of unkown modifiers because we can't determine whether
                // it was pressed or release reliably.
                let Some(event_modifier) = key_to_modifier(&key) else {
                    break 'send_event;
                };
                event.physical_key = physical_key;
                event.logical_key = key.clone();
                event.location = code_to_location(physical_key);
                let location_mask = ModLocationMask::from_location(event.location);

                let mut phys_mod_state = self.state.phys_modifiers.borrow_mut();
                let phys_mod = phys_mod_state
                    .entry(key)
                    .or_insert(ModLocationMask::empty());

                let is_active = current_modifiers.state().contains(event_modifier);
                let mut events = VecDeque::with_capacity(2);

                // There is no API for getting whether the button was pressed or released
                // during this event. For this reason we have to do a bit of magic below
                // to come up with a good guess whether this key was pressed or released.
                // (This is not trivial because there are multiple buttons that may affect
                // the same modifier)
                if !is_active {
                    event.state = Released;
                    if phys_mod.contains(ModLocationMask::LEFT) {
                        let mut event = event.clone();
                        event.location = KeyLocation::Left;
                        event.physical_key = get_left_modifier_code(&event.logical_key).into();
                        events.push_back(WindowEvent::KeyboardInput {
                            device_id: DEVICE_ID,
                            event,
                            is_synthetic: false,
                        });
                    }
                    if phys_mod.contains(ModLocationMask::RIGHT) {
                        event.location = KeyLocation::Right;
                        event.physical_key = get_right_modifier_code(&event.logical_key).into();
                        events.push_back(WindowEvent::KeyboardInput {
                            device_id: DEVICE_ID,
                            event,
                            is_synthetic: false,
                        });
                    }
                    *phys_mod = ModLocationMask::empty();
                } else {
                    if *phys_mod == location_mask {
                        // Here we hit a contradiction:
                        // The modifier state was "changed" to active,
                        // yet the only pressed modifier key was the one that we
                        // just got a change event for.
                        // This seemingly means that the only pressed modifier is now released,
                        // but at the same time the modifier became active.
                        //
                        // But this scenario is possible if we released modifiers
                        // while the application was not in focus. (Because we don't
                        // get informed of modifier key events while the application
                        // is not focused)

                        // In this case we prioritize the information
                        // about the current modifier state which means
                        // that the button was pressed.
                        event.state = Pressed;
                    } else {
                        phys_mod.toggle(location_mask);
                        let is_pressed = phys_mod.contains(location_mask);
                        event.state = if is_pressed { Pressed } else { Released };
                    }

                    events.push_back(WindowEvent::KeyboardInput {
                        device_id: DEVICE_ID,
                        event,
                        is_synthetic: false,
                    });
                }

                drop(phys_mod_state);

                for event in events {
                    self.queue_event(event);
                }
            }
        }

        if prev_modifiers == current_modifiers {
            return;
        }

        self.queue_event(WindowEvent::ModifiersChanged(self.state.modifiers.get()));
    }

    fn mouse_click(&self, event: &NSEvent, button_state: ElementState) {
        let button = mouse_button(event);

        self.update_modifiers(event, false);

        self.queue_event(WindowEvent::MouseInput {
            device_id: DEVICE_ID,
            state: button_state,
            button,
        });
    }

    fn mouse_motion(&self, event: &NSEvent) {
        let window_point = event.locationInWindow();
        let view_point = self.convertPoint_fromView(window_point, None);
        let view_rect = self.frame();

        if view_point.x.is_sign_negative()
            || view_point.y.is_sign_negative()
            || view_point.x > view_rect.size.width
            || view_point.y > view_rect.size.height
        {
            let mouse_buttons_down = NSEvent::pressedMouseButtons();
            if mouse_buttons_down == 0 {
                // Point is outside of the client area (view) and no buttons are pressed
                return;
            }
        }

        let x = view_point.x as f64;
        let y = view_rect.size.height as f64 - view_point.y as f64;
        let logical_position = LogicalPosition::new(x, y);

        self.update_modifiers(event, false);

        self.queue_event(WindowEvent::CursorMoved {
            device_id: DEVICE_ID,
            position: logical_position.to_physical(self.scale_factor()),
        });
    }
}

fn is_korean_jamo_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x3130..=0x318F
            | 0xA960..=0xA97F
            | 0xD7B0..=0xD7FF
            | 0xFFA0..=0xFFDC
    )
}

fn is_korean_jamo_text(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(ch) = chars.next() else {
        return false;
    };
    chars.next().is_none() && is_korean_jamo_char(ch)
}

fn is_korean_jamo_sequence(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_korean_jamo_char)
}

fn compose_korean_preedit(previous_raw: Option<&str>, incoming: &str) -> Option<(String, String)> {
    if !is_korean_jamo_sequence(incoming) {
        return None;
    }

    let mut raw = String::new();
    if let Some(previous_raw) = previous_raw.filter(|text| is_korean_jamo_sequence(text)) {
        raw.push_str(previous_raw);
    }
    raw.push_str(incoming);

    let mut composer = StringComposer::new();
    for ch in raw.chars() {
        composer.push_char(ch).ok()?;
    }
    Some((raw, composer.as_string().ok()?))
}

fn korean_carryover_commit_text(
    previous_raw: Option<&str>,
    transition_pending: bool,
    incoming: &str,
    current_marked_display: &str,
) -> Option<String> {
    if transition_pending
        && previous_raw.is_some_and(is_korean_jamo_sequence)
        && !incoming.is_empty()
        && !is_korean_jamo_sequence(incoming)
        && !current_marked_display.is_empty()
    {
        Some(current_marked_display.to_string())
    } else {
        None
    }
}

fn should_preserve_korean_marked_text_on_empty_preedit(
    previous_raw: Option<&str>,
    current_marked_display: &str,
) -> bool {
    previous_raw.is_some_and(is_korean_jamo_sequence) && !current_marked_display.is_empty()
}

fn utf16_range_to_byte_range(text: &str, range: NSRange) -> Option<(usize, usize, NSRange)> {
    if range.location == NSNotFound as NSUInteger {
        return None;
    }

    let start_utf16 = range.location as usize;
    let end_utf16 = start_utf16.saturating_add(range.length as usize);
    let mut current_utf16 = 0usize;
    let mut start_byte = None;
    let mut end_byte = None;

    for (byte_idx, ch) in text.char_indices() {
        if start_byte.is_none() && current_utf16 == start_utf16 {
            start_byte = Some(byte_idx);
        }
        if end_byte.is_none() && current_utf16 == end_utf16 {
            end_byte = Some(byte_idx);
        }
        current_utf16 += ch.len_utf16();
    }

    if start_byte.is_none() && start_utf16 == current_utf16 {
        start_byte = Some(text.len());
    }
    if end_byte.is_none() && end_utf16 == current_utf16 {
        end_byte = Some(text.len());
    }

    let start_byte = start_byte?;
    let end_byte = end_byte.unwrap_or(text.len());
    let clamped_start = start_utf16.min(current_utf16);
    let clamped_end = end_utf16.min(current_utf16);
    Some((
        start_byte,
        end_byte,
        NSRange::new(
            clamped_start as NSUInteger,
            clamped_end.saturating_sub(clamped_start) as NSUInteger,
        ),
    ))
}

/// Get the mouse button from the NSEvent.
fn mouse_button(event: &NSEvent) -> MouseButton {
    // The buttonNumber property only makes sense for the mouse events:
    // NSLeftMouse.../NSRightMouse.../NSOtherMouse...
    // For the other events, it's always set to 0.
    // MacOS only defines the left, right and middle buttons, 3..=31 are left as generic buttons,
    // but 3 and 4 are very commonly used as Back and Forward by hardware vendors and applications.
    match event.buttonNumber() {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        n => MouseButton::Other(n as u16),
    }
}

// NOTE: to get option as alt working we need to rewrite events
// we're getting from the operating system, which makes it
// impossible to provide such events as extra in `KeyEvent`.
fn replace_event(event: &NSEvent, option_as_alt: OptionAsAlt) -> Id<NSEvent> {
    let ev_mods = event_mods(event).state;
    let ignore_alt_characters = match option_as_alt {
        OptionAsAlt::OnlyLeft if event.lalt_pressed() => true,
        OptionAsAlt::OnlyRight if event.ralt_pressed() => true,
        OptionAsAlt::Both if ev_mods.alt_key() => true,
        _ => false,
    } && !ev_mods.control_key()
        && !ev_mods.super_key();

    if ignore_alt_characters {
        let ns_chars = event
            .charactersIgnoringModifiers()
            .expect("expected characters to be non-null");

        NSEvent::keyEventWithType(
            event.type_(),
            event.locationInWindow(),
            event.modifierFlags(),
            event.timestamp(),
            event.window_number(),
            None,
            &ns_chars,
            &ns_chars,
            event.is_a_repeat(),
            event.key_code(),
        )
    } else {
        event.copy()
    }
}

fn queue_tab_navigation_key_down(view: &WinitView, backwards: bool, event: KeyEvent) {
    let previous_state = view.state.ime_state.get();
    let next_state = tab_navigation_ime_state(previous_state, view.hasMarkedText());
    if next_state != previous_state {
        view.state.ime_state.set(next_state);
    }
    view.debug_log_ime(
        "tab_navigation:queue_keyboard_input",
        &format!(
            "backwards={} previous_state={:?} next_state={:?}",
            backwards, previous_state, next_state
        ),
    );
    view.queue_event(WindowEvent::KeyboardInput {
        device_id: DEVICE_ID,
        event,
        is_synthetic: false,
    });
}

#[cfg(test)]
mod tests {
    use super::{
        compose_korean_preedit, control_text_keyboard_input, is_korean_jamo_sequence,
        korean_carryover_commit_text, should_preserve_korean_marked_text_on_empty_preedit,
        should_suppress_raw_character_input, tab_navigation_ime_state, tab_navigation_key_event,
        ImeState,
    };
    use crate::{
        event::ElementState,
        keyboard::{Key, KeyCode, KeyLocation, NamedKey, PhysicalKey},
    };

    #[test]
    fn korean_jamo_sequence_detects_single_and_multi_char_jamo() {
        assert!(is_korean_jamo_sequence("ㅇ"));
        assert!(is_korean_jamo_sequence("ㅇㅏㄴ"));
        assert!(!is_korean_jamo_sequence("안"));
        assert!(!is_korean_jamo_sequence("a"));
    }

    #[test]
    fn compose_korean_preedit_appends_delta_jamo() {
        let (raw, display) = compose_korean_preedit(Some("ㅇ"), "ㅏ").expect("compose");
        assert_eq!(raw, "ㅇㅏ");
        assert_eq!(display, "아");
    }

    #[test]
    fn compose_korean_preedit_handles_longer_word() {
        let (raw, display) = compose_korean_preedit(Some("ㅇㅏ"), "ㄴ").expect("compose");
        assert_eq!(raw, "ㅇㅏㄴ");
        assert_eq!(display, "안");
    }

    #[test]
    fn korean_carryover_commit_detects_missing_startup_commit() {
        let commit =
            korean_carryover_commit_text(Some("ㅇㅏㄴ"), true, "녀", "안").expect("commit");
        assert_eq!(commit, "안");
    }

    #[test]
    fn korean_carryover_commit_ignores_regular_preedit_updates() {
        assert_eq!(
            korean_carryover_commit_text(Some("ㅎㅏ"), false, "세", "하"),
            None
        );
        assert_eq!(
            korean_carryover_commit_text(Some("ㅇㅏㄴ"), true, "ㄴ", "안"),
            None
        );
    }

    #[test]
    fn preserve_empty_korean_preedit_when_marked_text_still_exists() {
        assert!(should_preserve_korean_marked_text_on_empty_preedit(
            Some("ㅇㅏㅍ"),
            "앞"
        ));
        assert!(should_preserve_korean_marked_text_on_empty_preedit(
            Some("ㅇㅏㄴ"),
            "안"
        ));
    }

    #[test]
    fn do_not_preserve_empty_korean_preedit_without_raw_or_display_text() {
        assert!(!should_preserve_korean_marked_text_on_empty_preedit(
            None, "앞"
        ));
        assert!(!should_preserve_korean_marked_text_on_empty_preedit(
            Some("ㅇㅏㅍ"),
            ""
        ));
        assert!(!should_preserve_korean_marked_text_on_empty_preedit(
            Some("front"),
            "앞"
        ));
    }

    #[test]
    fn korean_input_source_suppresses_raw_character_input() {
        assert!(should_suppress_raw_character_input(
            true,
            "com.apple.inputmethod.Korean.2SetKorean"
        ));
    }

    #[test]
    fn non_korean_or_disabled_input_source_does_not_suppress_raw_character_input() {
        assert!(!should_suppress_raw_character_input(
            false,
            "com.apple.inputmethod.Korean.2SetKorean"
        ));
        assert!(!should_suppress_raw_character_input(
            true,
            "com.apple.keylayout.ABC"
        ));
    }

    #[test]
    fn tab_navigation_command_leaves_preedit_grounded_for_app_navigation() {
        assert_eq!(
            tab_navigation_ime_state(ImeState::Preedit, true),
            ImeState::Ground
        );
        assert_eq!(
            tab_navigation_ime_state(ImeState::Preedit, false),
            ImeState::Preedit
        );
        assert_eq!(
            tab_navigation_ime_state(ImeState::Ground, true),
            ImeState::Ground
        );
    }

    #[test]
    fn tab_navigation_key_event_matches_regular_tab_input() {
        let event = tab_navigation_key_event();
        assert_eq!(event.physical_key, PhysicalKey::Code(KeyCode::Tab));
        assert_eq!(event.logical_key, Key::Named(NamedKey::Tab));
        assert_eq!(event.text.as_deref(), Some("\t"));
        assert_eq!(event.location, KeyLocation::Standard);
        assert_eq!(event.state, ElementState::Pressed);
        assert!(!event.repeat);
        assert_eq!(
            event.platform_specific.text_with_all_modifiers.as_deref(),
            Some("\t")
        );
        assert_eq!(
            event.platform_specific.key_without_modifiers,
            Key::Named(NamedKey::Tab)
        );
    }

    #[test]
    fn control_tab_text_maps_to_tab_navigation_key_event() {
        let event = control_text_keyboard_input("\t").expect("tab key event");
        assert_eq!(event.logical_key, Key::Named(NamedKey::Tab));
        assert_eq!(event.physical_key, PhysicalKey::Code(KeyCode::Tab));

        let backtab_event = control_text_keyboard_input("\u{19}").expect("backtab key event");
        assert_eq!(backtab_event.logical_key, Key::Named(NamedKey::Tab));
        assert_eq!(backtab_event.physical_key, PhysicalKey::Code(KeyCode::Tab));

        assert!(control_text_keyboard_input("\n").is_none());
    }
}
