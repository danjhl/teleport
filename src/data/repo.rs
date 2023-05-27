use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

static DATA_PATH: &str = ".cache/teleport/";
static MARKED: &str = "marked";
static BOOKMARKED: &str = "bookmarked";

pub struct Repository {}

pub trait MarksRepository {
    fn get_marks(&self) -> BTreeMap<String, String>;
    fn get_bookmarks(&self) -> BTreeMap<String, String>;
    fn add_mark(&self, path: Option<String>) -> Result<usize, String>;
    fn store_bookmarks(&self, bookmarks: BTreeMap<String, String>);
    fn clear_marks(&self);
}

impl MarksRepository for Repository {
    fn get_marks(&self) -> BTreeMap<String, String> {
        get_marks_for(MARKED)
    }

    fn get_bookmarks(&self) -> BTreeMap<String, String> {
        get_marks_for(BOOKMARKED)
    }

    fn add_mark(&self, path: Option<String>) -> Result<usize, String> {
        let dir = env::current_dir().expect("Current directory");
        let marked_path = get_marks_file_path(MARKED);
        let marked = self.get_marks();
        let key = marked.len();

        let value = if path.is_none() {
            dir
        } else {
            let mut path_buf = PathBuf::new();
            let path_uw = path.unwrap();
            path_buf.push(&path_uw);

            if !path_buf.is_dir() {
                return Err("Path arg must be a directory".to_string());
            }

            if path_buf.is_absolute() {
                path_buf
            } else {
                dir.join(path_buf)
            }
        };

        let new_line = mark_entry(&value, key);

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(marked_path)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", new_line) {
            return Err(e.to_string());
        }

        Ok(key)
    }

    fn store_bookmarks(&self, bookmarks: BTreeMap<String, String>) {
        let bookmarked_path = get_marks_file_path(BOOKMARKED);
        let mut content = String::new();

        for (key, value) in bookmarks {
            content.push_str(&key);
            content.push(',');
            content.push_str(&value);
            content.push('\n');
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(bookmarked_path)
            .unwrap();

        if let Err(e) = write!(file, "{}", content) {
            eprintln!("Couldn't write to file: {}", e)
        }
    }

    fn clear_marks(&self) {
        let marked_path = get_marks_file_path(MARKED);
        let file = File::create(marked_path).unwrap();
        let result = file.set_len(0);
        if result.is_err() {
            panic!("Could not clear marks: {}", result.unwrap_err());
        }
    }
}

fn get_marks_for(file: &'static str) -> BTreeMap<String, String> {
    let marked_path = get_marks_file_path(file);

    let file = OpenOptions::new().read(true).open(marked_path).unwrap();

    let lines = io::BufReader::new(file).lines();
    let mut map = BTreeMap::new();

    for line in lines {
        let split_o = line.unwrap();
        let mut split = split_o.split(',');
        let key_o = split.next();
        let dir_o = split.next();

        if key_o.is_none() || dir_o.is_none() {
            continue;
        }

        let key = key_o.unwrap();
        let dir = dir_o.unwrap();

        map.insert(String::from(key), String::from(dir));
    }

    map
}

fn get_marks_file_path(file: &'static str) -> PathBuf {
    let home_dir = home::home_dir().expect("Unable to get home directory");

    let mut data_path = PathBuf::new();
    data_path.push(home_dir.as_path());
    data_path.push(DATA_PATH);

    let mut marked_path = PathBuf::new();
    marked_path.push(&data_path);
    marked_path.push(Path::new(file));

    marked_path
}

fn mark_entry(cd: &PathBuf, key: usize) -> String {
    format!("{},{}", key, cd.to_str().expect("Valid string"))
}
