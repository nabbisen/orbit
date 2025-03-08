use std::env;
use std::path::PathBuf;

use rfd::FileDialog;

use crate::app::{message::Message, state::State};

/// get dialog to choose file
pub fn file_dialog(_state: &State, _message: &Message) -> FileDialog {
    let default_directory = "."; // todo

    let file_dialog = FileDialog::new()
        .add_filter("Excel", &["xlsx"])
        .add_filter("All files", &["*"])
        .set_directory(default_directory);
    file_dialog
}

/// get full path of user desktop
pub fn desktop_path() -> PathBuf {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .unwrap();
    PathBuf::from(home).join("Desktop")
}
