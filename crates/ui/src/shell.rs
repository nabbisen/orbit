//! Application shell (RFC-027): snora `AppLayout` with a two-level navigation:
//! a vertical sidebar for the three top-level groups (Search, AI, Settings) and
//! a horizontal tab bar for the sub-views within each group.
//!
//! RFC-034: [`key_to_message`] is the pure keyboard-map function. It is called
//! from `orbok` via an `iced::keyboard::on_key_press` subscription. Keeping
//! it here (in `orbok-ui`) means it is unit-testable without the iced runtime.

use crate::i18n::{MessageKey, tr};
use crate::state::{AppState, Message, NavGroup, ViewId};
use crate::views;
use iced::Element;
use snora::lucide;
use snora::{
    AppLayout, Icon, LayoutDirection, SideBar, SideBarItem, Tab, TabBar, render,
    widget::{app_side_bar, app_tab_bar},
};

/// Map a key event to a [`Message`], or `None` to let iced handle it normally.
///
/// **Text-input safety:** when `text_input_focused` is `true`, only global
/// shortcuts that do *not* intercept printable characters are fired (Ctrl/Cmd
/// combos and Escape). Arrow keys and Enter are suppressed while text input
/// has focus so typing is never hijacked.
///
/// This function is pure and contains no iced runtime state, so it can be
/// called from tests without a display server.
pub fn key_to_message(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
    text_input_focused: bool,
) -> Option<Message> {
    use iced::keyboard::Key;
    use iced::keyboard::key::Named;

    match key {
        // Ctrl/Cmd + K  →  focus global search input (works from any view).
        Key::Character(c) if c.as_str() == "k" && modifiers.command() => Some(Message::FocusSearch),
        // Ctrl/Cmd + ,  →  open Settings.
        Key::Character(c) if c.as_str() == "," && modifiers.command() => {
            Some(Message::Switch(ViewId::Settings))
        }
        // Escape  →  close any open overlay / dialog; restore focus to trigger.
        Key::Named(Named::Escape) => Some(Message::DismissOverlay),
        // Enter  →  submit search, but only when search input is focused.
        Key::Named(Named::Enter) if text_input_focused => Some(Message::SubmitSearch),
        // Arrow keys  →  move result selection, only when NOT typing.
        Key::Named(Named::ArrowDown) if !text_input_focused => Some(Message::SelectNextResult),
        Key::Named(Named::ArrowUp) if !text_input_focused => Some(Message::SelectPrevResult),
        // Everything else: let iced handle it (printable keys, Tab, etc.).
        _ => None,
    }
}

fn tab_action_to_msg(action: snora::TabAction<ViewId>) -> Message {
    let snora::TabAction::Pressed(id) = action;
    Message::Switch(id)
}

fn build_tab_bar(tabs: Vec<Tab<ViewId>>, active: ViewId) -> Element<'static, Message> {
    app_tab_bar(
        TabBar { tabs, active },
        &tab_action_to_msg,
        LayoutDirection::Ltr,
    )
}

/// The iced application wrapper around [`AppState`].
#[derive(Default)]
pub struct OrbokApp {
    pub state: AppState,
    /// Whether the global search text input currently holds keyboard focus.
    /// Tracked so [`key_to_message`] can distinguish text entry from navigation.
    pub search_focused: bool,
}

impl OrbokApp {
    pub fn with_state(state: AppState) -> Self {
        Self {
            state,
            search_focused: false,
        }
    }

    pub fn update(&mut self, message: Message) {
        if matches!(message, Message::FocusSearch) {
            self.search_focused = true;
        }
        // Typing in search clears the focus flag (next keypress will be text).
        if matches!(message, Message::QueryChanged(_)) {
            self.search_focused = false;
        }
        self.state.update(&message);
    }

    pub fn view(&self) -> Element<'_, Message> {
        let locale = self.state.locale;

        // ── Startup wizard takes priority ──────────────────────────────
        if self.state.wizard.is_some() {
            return views::wizard_view(&self.state);
        }

        // ── Sidebar: three top-level groups ───────────────────────────
        let sidebar_items: Vec<SideBarItem<Message, NavGroup>> = vec![
            SideBarItem {
                view_id: NavGroup::Search,
                icon: Icon::Lucide(lucide::Search),
                tooltip: tr(locale, MessageKey::NavSearch).to_string(),
                on_press: Message::SwitchGroup(NavGroup::Search),
            },
            SideBarItem {
                view_id: NavGroup::Ai,
                icon: Icon::Lucide(lucide::BrainCircuit),
                tooltip: tr(locale, MessageKey::NavAi).to_string(),
                on_press: Message::SwitchGroup(NavGroup::Ai),
            },
            SideBarItem {
                view_id: NavGroup::Settings,
                icon: Icon::Lucide(lucide::Settings),
                tooltip: tr(locale, MessageKey::NavSettings).to_string(),
                on_press: Message::SwitchGroup(NavGroup::Settings),
            },
        ];
        let side_bar = app_side_bar(
            SideBar {
                items: sidebar_items,
                active: self.state.active_view.group(),
            },
            LayoutDirection::Ltr,
        );

        // ── Tab bar: sub-views within the active group ─────────────────
        let tab_bar_el: Option<Element<'_, Message>> = match self.state.active_view.group() {
            NavGroup::Search => Some(build_tab_bar(
                vec![
                    Tab {
                        id: ViewId::Search,
                        label: tr(locale, MessageKey::NavSearch).to_string(),
                        icon: None,
                    },
                    Tab {
                        id: ViewId::Sources,
                        label: tr(locale, MessageKey::NavSources).to_string(),
                        icon: None,
                    },
                ],
                self.state.active_view,
            )),
            NavGroup::Ai => Some(build_tab_bar(
                vec![
                    Tab {
                        id: ViewId::Indexing,
                        label: tr(locale, MessageKey::NavIndexing).to_string(),
                        icon: None,
                    },
                    Tab {
                        id: ViewId::Storage,
                        label: tr(locale, MessageKey::NavStorage).to_string(),
                        icon: None,
                    },
                    Tab {
                        id: ViewId::Models,
                        label: tr(locale, MessageKey::NavModels).to_string(),
                        icon: None,
                    },
                ],
                self.state.active_view,
            )),
            NavGroup::Settings => None,
        };

        // ── Active page body ───────────────────────────────────────────
        let page_body = match self.state.active_view {
            ViewId::Search => views::search_view(&self.state),
            ViewId::Sources => views::sources_view(&self.state),
            ViewId::Indexing => views::indexing_view(&self.state),
            ViewId::Storage => views::storage_view(&self.state),
            ViewId::Models => views::models_view(&self.state),
            ViewId::Settings => views::settings_view(&self.state),
        };

        // Compose: tab bar (if any) stacked above the page body.
        let body: Element<'_, Message> = if let Some(tabs) = tab_bar_el {
            iced::widget::column![tabs, page_body].spacing(0).into()
        } else {
            page_body
        };

        render(AppLayout::new(body).side_bar(side_bar))
    }

    pub fn title(&self) -> String {
        tr(self.state.locale, MessageKey::AppTitle).to_string()
    }

    /// Map the active snora token palette to an `iced::Theme` so iced uses
    /// the correct background, text, and accent colors. Without this, iced
    /// always renders with its built-in Light theme regardless of which snora
    /// preset is active.
    ///
    /// `iced::Theme::Custom` accepts an `iced::theme::Palette` with six roles.
    /// We map the snora palette's semantic roles to those six fields.
    pub fn iced_theme(&self) -> iced::Theme {
        use snora::design::style::color::to_iced_color;
        let p = &self.state.tokens.palette;
        let is_dark = matches!(
            self.state.theme,
            crate::theme::Theme::Dark | crate::theme::Theme::HighContrastDark
        );
        let palette = iced::theme::Palette {
            background: to_iced_color(p.background),
            text: to_iced_color(p.text_primary),
            primary: to_iced_color(p.accent),
            success: to_iced_color(p.success),
            warning: to_iced_color(p.warning),
            danger: to_iced_color(p.danger),
        };
        let name = if is_dark { "orbok-dark" } else { "orbok-light" };
        iced::Theme::custom(name, palette)
    }
}
