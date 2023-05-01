use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

static DATA_PATH: &str = ".cache/teleport/";
static MARKED: &str = "marked";
static BOOKMARKED: &str = "bookmarked";

static HELP: &str = r#"
Usage: tp [CMD] [ARGS]

help            Show usage
ls              List marked directories
m               Mark current directory
m [dir]         Mark directory 
g [key]         Get marked directory
bm [key] [dir]  Bookmark directory
"#;

static ERR_NO_CMD: &str = "No command given. Run 'tp help' for more.";

struct Repository {}

trait MarksRepository {
    fn get_marks(&self) -> BTreeMap<String, String>;
    fn get_bookmarks(&self) -> BTreeMap<String, String>;
    fn add_mark(&self, path: Option<String>) -> Result<usize, String>;
    fn store_bookmarks(&self, bookmarks: BTreeMap<String, String>);
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
    return format!("{},{}", key, cd.to_str().expect("Valid string"));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let repo = Repository {};
    let result = run_cmd(args, repo);

    match result {
        Ok(out) => print!("{}", out),
        Err(msg) => eprintln!("{}", msg),
    }
}

fn run_cmd(args: Vec<String>, repo: impl MarksRepository) -> Result<String, String> {
    let cmd = match args.get(1) {
        Some(s) => s,
        None => return Err(ERR_NO_CMD.to_string()),
    };

    if cmd == "help" {
        return Ok(HELP.to_string());
    }
    if cmd == "ls" {
        return Ok(ls(repo));
    }

    let arg = args.get(2);

    if cmd == "m" {
        return mark(arg, repo);
    }

    if cmd == "g" {
        return get(arg, repo);
    }

    if cmd == "bm" {
        return bookmark(arg, args.get(3), repo);
    }

    Err(format!("Unknown command '{}'", cmd))
}

fn ls(repo: impl MarksRepository) -> String {
    let marks = repo.get_marks();
    let bookmarks = repo.get_bookmarks();

    let mut out = String::new();
    out.push_str("\n# Marks\n\n");
    for (key, value) in marks {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 2));
    }
    out.push_str("\n");

    out.push_str("# Bookmarks\n\n");
    for (key, value) in bookmarks {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 2));
    }
    out.push_str("\n");
    out
}

fn mark(arg: Option<&String>, repo: impl MarksRepository) -> Result<String, String> {
    repo.add_mark(arg.map(|it| it.to_string()))
        .map(|key| format!("Marked as {}\n", key))
}

fn get(arg: Option<&String>, repo: impl MarksRepository) -> Result<String, String> {
    if arg.is_none() {
        return Err("Get command requires key argument\n".to_string());
    }
    let key = arg.unwrap();
    let marks = repo.get_marks();
    let bookmarks = repo.get_bookmarks();

    let mut value_o = bookmarks.get(key);
    if (value_o.is_some()) {
        return Ok(value_o.unwrap().to_string());
    }

    let value_o = marks.get(key);
    if (value_o.is_some()) {
        return Ok(value_o.unwrap().to_string());
    }
    return Ok("".to_string());
}

fn bookmark(
    key: Option<&String>,
    path: Option<&String>,
    repo: impl MarksRepository,
) -> Result<String, String> {
    if key.is_none() {
        return Err("bookmark needs a key argument".to_string());
    }

    // todo duplicated
    let dir = env::current_dir().expect("Current directory");

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

    let key_uw = key.unwrap();
    let mut bookmarks = repo.get_bookmarks();
    bookmarks.insert(key_uw.to_string(), value.to_string_lossy().to_string());
    repo.store_bookmarks(bookmarks);

    Ok(format!("Bookmarked as {}\n", key_uw.to_string()))
}

#[cfg(test)]
mod test {
    use crate::run_cmd;
    use crate::MarksRepository;
    use crate::{ERR_NO_CMD, HELP};
    use std::collections::BTreeMap;

    struct MockRepo {}

    impl MarksRepository for MockRepo {
        fn get_marks(&self) -> BTreeMap<String, String> {
            let mut map = BTreeMap::new();
            map.insert("0".to_string(), "/dir".to_string());
            map.insert("1".to_string(), "/dir/two".to_string());
            map
        }

        fn add_mark(&self, path: Option<String>) -> Result<usize, String> {
            Ok(5)
        }

        fn get_bookmarks(&self) -> BTreeMap<String, String> {
            let mut map = BTreeMap::new();
            map.insert("a".to_string(), "/dir".to_string());
            map.insert("b".to_string(), "/dir/two".to_string());
            map
        }

        fn store_bookmarks(&self, bookmarks: BTreeMap<String, String>) {}
    }

    fn mock_repo() -> impl MarksRepository {
        MockRepo {}
    }

    #[test]
    fn should_fail_without_cmd() {
        let result = run_cmd(vec!["tp".to_string()], mock_repo());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ERR_NO_CMD.to_string());
    }

    #[test]
    fn should_print_help() {
        let result = run_cmd(vec!["tp".to_string(), "help".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HELP.to_string());
    }

    #[test]
    fn should_fail_with_unknown_cmd() {
        let result = run_cmd(vec!["tp".to_string(), "unknown".to_string()], mock_repo());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown command 'unknown'");
    }

    #[test]
    fn should_list_marks() {
        let result = run_cmd(vec!["tp".to_string(), "ls".to_string()], mock_repo());

        let out = concat!(
            "\n# Marks\n\n",
            "0  : /dir\n",
            "1  : /dir/two\n",
            "\n# Bookmarks\n\n",
            "a  : /dir\n",
            "b  : /dir/two\n\n"
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), out);
    }

    #[test]
    fn should_add_mark() {
        let result = run_cmd(vec!["tp".to_string(), "m".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Marked as 5\n");
    }

    #[test]
    fn should_add_mark_with_path() {
        let result = run_cmd(
            vec!["tp".to_string(), "m".to_string(), "dir".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Marked as 5\n");
    }

    #[test]
    fn should_get_mark() {
        let result = run_cmd(
            vec!["tp".to_string(), "g".to_string(), "0".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/dir");
    }

    #[test]
    fn should_fail_to_get_mark_without_key_arg() {
        let result = run_cmd(vec!["tp".to_string(), "g".to_string()], mock_repo());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Get command requires key argument\n");
    }

    #[test]
    fn should_add_bookmark_for_current_dir() {
        let result = run_cmd(
            vec!["tp".to_string(), "bm".to_string(), "cd".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Bookmarked as cd\n");
    }
}
