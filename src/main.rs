use home;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

static DATA_PATH: &'static str = ".cache/teleport/";
static MARKED: &'static str = "marked";
static BOOKMARKED: &'static str = "bookmarked";

fn main() {
    let args: Vec<String> = env::args().collect();

    let home_dir = home::home_dir().expect("Unable to get home directory");

    let mut data_path = PathBuf::new();
    data_path.push(home_dir.as_path());
    data_path.push(DATA_PATH);
    
    if !data_path.exists() {
        fs::create_dir_all(&data_path).expect(&format!("Could not create marked dir: {}", &data_path.to_str().unwrap()));
    }

    let mut marked_path = PathBuf::new();
    marked_path.push(&data_path);
    marked_path.push(Path::new(MARKED));

    create_file_if_not_exists(&marked_path);

    let mut bookmarked_path = PathBuf::new();
    bookmarked_path.push(&data_path);
    bookmarked_path.push(Path::new(BOOKMARKED));

    create_file_if_not_exists(&bookmarked_path);

    
    if args.len() == 1 {
        return;
    }

    
    let op = match args.get(1) {
        None => {
            print!("fileop needs an operation see --help\n");
            return;
        }
        Some(s) => s,
    };

    if op == "help" {
        print_help();
    }

    if op == "ls" {
        let marked = read_marked_dirs(&marked_path);
        let bookmarked = read_marked_dirs(&bookmarked_path);
        print!("{}", list_marks(marked, bookmarked));
    }

    if op == "m" {
        mark_current_dir();
    }
    
}

fn create_file_if_not_exists(path: &PathBuf) {
    if !path.exists() {
        File::create(&path).expect(&format!("could not create file: {}", &path.to_str().unwrap()));
    }
}


fn list_marks(marked: HashMap<String, String>, bookmarked: HashMap<String, String>) -> String {
    let mut out = String::new();
    out.push_str("\n# Marked\n\n");
    for (key, value) in marked {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 2));
    }
    out.push_str("\n# Bookmarked\n\n");
    for (key, value) in bookmarked {
        out.push_str(&format!("{:width$} : {}\n\n", key, value, width = 2));
    }

    out
}

fn read_marked_dirs(marked: &PathBuf) -> HashMap<String, String> {
    let file = File::open(marked.as_path()).unwrap();
    let lines = io::BufReader::new(file).lines();
    let mut map = HashMap::new();

    for line in lines {
        let split_o = line.unwrap();
        let mut split = split_o.split(",");
        let key_o = split.next();
        let dir_o = split.next();
        let key = key_o.unwrap();
        let dir = dir_o.unwrap();

        map.insert(String::from(key), String::from(dir));
    }

    map
}

fn mark_current_dir() {
    todo!()
}

fn print_help() {
    print!("Move between marked directories\n\n");
    print!("Usage: tp [CMD] [ARGS]\n\n");
    print!("No cmd              Show marked directories to move to\n");
    // print!("m                   Mark current directory\n");
    // print!("m <dir>             Mark <dir>\n");
    // print!("bm                  Bookmark current directory");
    // print!("bm <dir>            Bookmark <dir>");
    // print!("c                   Clear marked directories\n");
    // print!("rb <dir>            Remove bookmark for <dir>\n");
}



#[test]
fn list_marks_test() {
    // GIVEN
    let mut marks = HashMap::new();
    marks.insert("1".to_string(), "/some/dir".to_string());

    let mut bookmarks = HashMap::new();
    bookmarks.insert("m".to_string(), "/some/dir".to_string());

    // WHEN
    let result = list_marks(marks, bookmarks);

    // THEN
    assert_eq!(result, "\n# Marked\n\n1  : /some/dir\n\n# Bookmarked\n\nm  : /some/dir\n\n");
}

