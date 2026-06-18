//! Application shell (RFC-027): snora `AppLayout` with a two-level navigation:
//! a vertical sidebar for the three top-level groups (Search, AI, Settings) and
//! a horizontal tab bar for the sub-views within each group.

use crate::i18n::{MessageKey, tr};
use crate::state::{AppState, Message, NavGroup, ViewId};
use crate::views;
use iced::Element;
use snora::lucide;
use snora::{AppLayout, Icon, LayoutDirection, SideBar, SideBarItem, Tab, TabBar, render,
            widget::{app_side_bar, app_tab_bar}};

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
}

impl OrbokApp {
    pub fn with_state(state: AppState) -> Self {
        Self { state }
    }

    pub fn update(&mut self, message: Message) {
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
        let tab_bar_el: Option<Element<'_, Message>> =
            match self.state.active_view.group() {
                NavGroup::Search => {
                    Some(build_tab_bar(
                        vec![
                            Tab { id: ViewId::Search,  label: tr(locale, MessageKey::NavSearch).to_string(),  icon: None },
                            Tab { id: ViewId::Sources, label: tr(locale, MessageKey::NavSources).to_string(), icon: None },
                        ],
                        self.state.active_view,
                    ))
                }
                NavGroup::Ai => {
                    Some(build_tab_bar(
                        vec![
                            Tab { id: ViewId::Indexing, label: tr(locale, MessageKey::NavIndexing).to_string(), icon: None },
                            Tab { id: ViewId::Storage,  label: tr(locale, MessageKey::NavStorage).to_string(),  icon: None },
                            Tab { id: ViewId::Models,   label: tr(locale, MessageKey::NavModels).to_string(),   icon: None },
                        ],
                        self.state.active_view,
                    ))
                }
                NavGroup::Settings => None,
            };

        // ── Active page body ───────────────────────────────────────────
        let page_body = match self.state.active_view {
            ViewId::Search   => views::search_view(&self.state),
            ViewId::Sources  => views::sources_view(&self.state),
            ViewId::Indexing => views::indexing_view(&self.state),
            ViewId::Storage  => views::storage_view(&self.state),
            ViewId::Models   => views::models_view(&self.state),
            ViewId::Settings => views::settings_view(&self.state),
        };

        // Compose: tab bar (if any) stacked above the page body.
        let body: Element<'_, Message> = if let Some(tabs) = tab_bar_el {
            iced::widget::column![tabs, page_body]
                .spacing(0)
                .into()
        } else {
            page_body
        };

        render(AppLayout::new(body).side_bar(side_bar))
    }

    pub fn title(&self) -> String {
        tr(self.state.locale, MessageKey::AppTitle).to_string()
    }
}
