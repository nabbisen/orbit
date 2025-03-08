use std::{borrow::Cow, fs};

use crate::app::consts::SKIPPED_DIRS;

pub fn walk(dirpath: Cow<'_, str>) -> impl Iterator<Item = String> {
    let mut dirs_text = vec![];
    let entries = fs::read_dir(dirpath.as_ref()).expect("ooops");
    entries.flatten().for_each(|x| {
        let path = x.path();
        if path.is_dir()
            && !SKIPPED_DIRS.contains(&path.file_name().unwrap().to_string_lossy().as_ref())
        {
            let dirname = path.to_string_lossy();
            dirs_text.push(dirname.clone().into_owned());
            let child_dirs = walk(dirname);
            dirs_text.extend(child_dirs);
        }
    });
    dirs_text.into_iter()
}
