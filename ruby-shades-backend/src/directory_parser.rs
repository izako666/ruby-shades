use std::{ffi::OsStr, fs::read_dir, path::Path, sync::Mutex};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
/**
 * This file is a service that reads the source_dir, reads all the directories and files within it, and organizes it in Vec<PathObject>
*/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathObject {
    path: String,
    name: String,
    nested_paths: Vec<PathObject>,
}

static PATH_OBJECT: Lazy<Mutex<Option<PathObject>>> = Lazy::new(|| Mutex::new(None));

pub fn initialize(source_dir: &str) {
    let path = Path::new(source_dir);
    let mut main_path_obj = PathObject {
        name: path
            .file_name()
            .unwrap_or(OsStr::new("default"))
            .to_str()
            .unwrap_or("default")
            .to_string(),
        path: String::from(source_dir),
        nested_paths: Vec::new(),
    };
    explore(path, &mut main_path_obj);

    let mut path_object = PATH_OBJECT.lock().unwrap();
    *path_object = Some(main_path_obj);
}
fn explore(base_path: &Path, data: &mut PathObject) {
    match read_dir(base_path) {
        Ok(rd) => {
            let mut nested_paths: Vec<PathObject> = Vec::new();

            for dir_entry in rd {
                if let Ok(ref entry) = dir_entry {
                    let mut is_symbolic_link: bool = false;
                    if let Ok(metadata) = entry.metadata() {
                        is_symbolic_link = metadata.is_symlink();
                    }
                    if entry.path().to_str().is_none()
                        || entry.file_name().to_str().is_none()
                        || is_symbolic_link
                    {
                        continue;
                    }
                    let path_str: String = String::from(entry.path().to_str().unwrap());
                    let name_str: String = String::from(entry.file_name().to_str().unwrap());
                    let mut dirs: PathObject = PathObject {
                        path: path_str,
                        name: name_str,
                        nested_paths: Vec::new(),
                    };
                    if entry.path().is_dir() {
                        explore(entry.path().as_path(), &mut dirs);
                    }

                    nested_paths.push(dirs);
                }
            }

            if nested_paths.len() > 0 {
                data.nested_paths = nested_paths;
            }
        }

        Err(error) => {
            panic!("{}", error);
        }
    }
}
