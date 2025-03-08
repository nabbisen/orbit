#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::Font;

mod app;
use app::{
    consts::{APP_THEME, APP_TITLE},
    font::app_default_font,
    subscription, update, view,
};
mod core;

/// app entry point
pub fn run() -> iced::Result {
    let app = iced::application(APP_TITLE, update::handle, view::handle)
        .default_font(Font::with_name(app_default_font()))
        .subscription(subscription::handle)
        .theme(|_state| APP_THEME);
    app.run()
}
