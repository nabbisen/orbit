//! Application shell (RFC-027): snora `AppLayout` with the sidebar
//! navigation rail, dispatching to the page view functions.

use crate::i18n::{MessageKey, tr};
use crate::state::{AppState, Message, ViewId};
use crate::views;
use iced::Element;
use snora::{AppLayout, LayoutDirection, SideBar, SideBarItem, render, widget::app_side_bar};

/// The iced application wrapper around [`AppState`].
#[derive(Default)]
pub struct OrbokApp {
    pub state: AppState,
}

impl OrbokApp {
    pub fn with_state(state: AppState) -> Self {
        Self { state }
    }

    /// iced update entry. Pure transitions only; `orbok-app` layers
    /// backend effects on top.
    pub fn update(&mut self, message: Message) {
        self.state.update(&message);
    }

    /// iced view entry: sidebar + active page (RFC-027 component
    /// mapping: AppShell → AppLayout, SidebarNav → app_side_bar).
    pub fn view(&self) -> Element<'_, Message> {
        let locale = self.state.locale;
        let icon_for = |view: ViewId| -> &'static str {
            match view {
                ViewId::Search => "🔍",
                ViewId::Sources => "📁",
                ViewId::Indexing => "⏳",
                ViewId::Storage => "💾",
                ViewId::Models => "🧠",
                ViewId::Settings => "⚙",
            }
        };
        let tooltip_for = |view: ViewId| -> &'static str {
            match view {
                ViewId::Search => tr(locale, MessageKey::NavSearch),
                ViewId::Sources => tr(locale, MessageKey::NavSources),
                ViewId::Indexing => tr(locale, MessageKey::NavIndexing),
                ViewId::Storage => tr(locale, MessageKey::NavStorage),
                ViewId::Models => tr(locale, MessageKey::NavModels),
                ViewId::Settings => tr(locale, MessageKey::NavSettings),
            }
        };
        let items = ViewId::ALL
            .iter()
            .map(|view| SideBarItem {
                view_id: *view,
                icon: icon_for(*view).into(),
                tooltip: tooltip_for(*view).into(),
                on_press: Message::Switch(*view),
            })
            .collect();
        let side_bar = app_side_bar(
            SideBar {
                items,
                active: self.state.active_view,
            },
            LayoutDirection::Ltr,
        );

        let body = match self.state.active_view {
            ViewId::Search => views::search_view(&self.state),
            ViewId::Sources => views::sources_view(&self.state),
            ViewId::Indexing => views::indexing_view(&self.state),
            ViewId::Storage => views::storage_view(&self.state),
            ViewId::Models => views::models_view(&self.state),
            ViewId::Settings => views::settings_view(&self.state),
        };

        render(AppLayout::new(body).side_bar(side_bar))
    }

    /// Window title.
    pub fn title(&self) -> String {
        tr(self.state.locale, MessageKey::AppTitle).to_string()
    }
}
