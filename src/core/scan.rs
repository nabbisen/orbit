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
        } else if path.is_file() {
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            match extension {
                "xlsx" | "xlsm" => {
                    let found =
                        findtext_sheet::search("hej", path.to_string_lossy().to_string().as_str());
                    if let Ok(found) = found {
                        if 0 < found.len() {
                            println!("sheet: {:?}", found);
                        }
                    }
                }
                "pdf" => {
                    let found =
                        findtext_pdf::search("hej", path.to_string_lossy().to_string().as_str());
                    if let Ok(found) = found {
                        if 0 < found.len() {
                            println!("sheet: {:?}", found);
                        }
                    }
                }
                "docx" | "docm" => {
                    let found =
                        findtext_doc::search("hej", path.to_string_lossy().to_string().as_str());
                    if let Ok(found) = found {
                        if found {
                            println!("sheet: {:?}", found);
                        }
                    }
                }
                _ => {
                    let found = findtext_textfile::search(
                        "hej",
                        path.to_string_lossy().to_string().as_str(),
                    );
                    if let Ok(found) = found {
                        if 0 < found.matched.len() {
                            println!("sheet: {:?}", found);
                        }
                    }
                }
            }
        }
    });
    dirs_text.into_iter()
}
