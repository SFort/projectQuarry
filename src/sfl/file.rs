#![allow(dead_code)]
use sfl::Loghelper;
use std::{
    fs,
    path::{Path, PathBuf},
};
pub fn get_contents(path: &str, splitters: Vec<&str>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut paths = Path::new(&path)
        .read_dir()
        .expect("readdir failed")
        .filter_map(|entry| match entry {
            Ok(r) => {
                if r.file_name().into_string().unwrap().starts_with('.') {
                    return None;
                }
                Some(r.path())
            }
            Err(_) => None,
        })
        .collect::<Vec<PathBuf>>();
    let mut split_paths = Vec::new();
    if !splitters.is_empty() {
        for splits in splitters.iter() {
            let splits: PathBuf = [path, splits].iter().collect();
            if let Some(entry) = remove_item(&mut paths, splits) {
                split_paths.push(entry);
            }
        }
    }
    (paths, split_paths)
}
pub fn remove_item(vec: &mut Vec<PathBuf>, item: PathBuf) -> Option<PathBuf> {
    let pos = vec.iter().position(|x| *x == *item)?;
    Some(vec.remove(pos))
}
pub fn copy(from: PathBuf, to: &PathBuf, mut log: &mut Loghelper) {
    let mut to = to.clone();
    if let Some(dir_name) = from.file_name() {
        to.push(dir_name);

        if from.is_dir() {
            if !to.exists() {
                fs::DirBuilder::new().recursive(true).create(&to).unwrap();
            }
            for entry in get_contents(from.to_str().unwrap(), vec![]).0.iter() {
                copy(entry.into(), &to, &mut log);
            }
        } else {
            log.desc(format!("Moving{:#?}", from.file_name().unwrap()));
            fs::copy(&from, to).unwrap();
        }
    }
}
pub fn clear_extra(from: &PathBuf, to: &PathBuf, path: String) {
    if to.is_dir() {
        for entry in get_contents(to.to_str().unwrap(), vec![]).0.iter() {
            if let Some(name) = entry.file_name() {
                let mut from2 = from.clone();
                from2.push(&path);
                from2.push(&name);
                if !from2.exists() || (from2.is_dir() && from2.exists()) {
                    clear_extra(&from2, &entry, name.to_str().unwrap().to_string());
                }
            }
        }
        let _ = fs::remove_dir(&to);
    } else {
        fs::remove_file(to).unwrap();
    }
}
