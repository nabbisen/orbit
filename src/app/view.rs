use std::borrow::Cow;

use iced::widget::{Column, scrollable, text};
use iced::{Element, Fill, Font};

use crate::core::scan::walk;
use crate::core::utils::desktop_path;

use super::font::monospace_font;
use super::{message::Message, state::State};

/// iced view function
pub fn handle(_state: &State) -> Element<Message> {
    let monospace_font = Font::with_name(monospace_font());
    let desktop_path = desktop_path().to_string_lossy().into_owned();
    let dirs = walk(Cow::Borrowed(desktop_path.as_str()));
    let rows: Vec<Element<Message>> = dirs
        .into_iter()
        .map(|x| Element::from(text(x).font(monospace_font)))
        .collect();
    scrollable(Column::with_children(rows).width(Fill)).into()
}
