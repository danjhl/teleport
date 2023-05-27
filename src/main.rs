mod data;

use std::env;
use std::path::PathBuf;

use crate::data::repo::*;

static HELP: &str = r#"
Usage: [ARGS] [FLAGS]

[dir]            Mark directory uses current directory if no dir argument
[dir] -b [key]   Mark as bookmark with key
-ls              List marked directories
-h, --help       Show usage
-g [key]         Get marked directory
-rm [key]        Remove bookmark with key
-clr, --clear    Remove all marks
"#;

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
    let flags: Vec<&String> = args.iter().filter(|a| a.starts_with('-')).collect();
    let cmd_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();

    mark(&cmd_args[1..], flags, repo)
}

fn ls(repo: impl MarksRepository) -> String {
    let marks = repo.get_marks();
    let bookmarks = repo.get_bookmarks();

    let mut out = String::new();
    out.push_str("\n# Marks\n\n");
    for (key, value) in marks {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 5));
    }
    out.push('\n');

    out.push_str("# Bookmarks\n\n");
    for (key, value) in bookmarks {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 5));
    }
    out.push('\n');
    out
}

fn mark(
    args: &[&String],
    flags: Vec<&String>,
    repo: impl MarksRepository,
) -> Result<String, String> {
    if flags.len() > 1 {
        return Err("Cannot mix flags".to_string());
    }

    if args.is_empty() && flags.is_empty() {
        return mark_dir(None, repo);
    }

    if flags.is_empty() {
        if args.len() == 1 {
            return mark_dir(args.first().map(|s| s.to_owned()), repo);
        } else {
            return Err("Too many argurments".to_string());
        }
    }

    let flag = flags.first().unwrap().to_string();

    if flag == "-ls" {
        return Ok(ls(repo));
    }

    if flag == "-h" || flag == "--help" {
        return Ok(HELP.to_string());
    }

    if flag == "-g" {
        if args.len() == 1 {
            return get(args.first().map(|s| s.to_owned()), repo);
        } else if args.is_empty() {
            return Err("Get command requires key argument\n".to_string());
        } else {
            return Err("Too many arguments".to_string());
        }
    }

    if flag == "-b" {
        if args.len() == 2 {
            return bookmark(
                args.get(1).map(|s| s.to_owned()),
                args.first().map(|s| s.to_owned()),
                repo,
            );
        } else if args.len() == 1 {
            return bookmark(args.first().map(|s| s.to_owned()), None, repo);
        } else {
            return Err("Wrong number of arguments".to_string());
        }
    }

    if flag == "-rm" {
        if args.len() != 1 {
            return Err("Wrong number of arguments".to_string());
        } else {
            return remove_bookmark(args.first().map(|s| s.to_owned()), repo);
        }
    }

    if flag == "-clr" || flag == "--clear" {
        if !args.is_empty() {
            return Err("Wrong number of arguments".to_string());
        } else {
            repo.clear_marks();
            return Ok("Cleared marks\n".to_string());
        }
    }

    Err("Unkown command for mark command".to_string())
}

fn mark_dir(arg: Option<&String>, repo: impl MarksRepository) -> Result<String, String> {
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

    match bookmarks.get(key) {
        Some(v) => Ok(v.to_string()),
        None => match marks.get(key) {
            Some(v) => Ok(v.to_string()),
            None => Ok("".to_string()),
        },
    }
}

fn bookmark(
    key: Option<&String>,
    path: Option<&String>,
    repo: impl MarksRepository,
) -> Result<String, String> {
    if key.is_none() {
        return Err("bookmark needs a key argument".to_string());
    }

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

fn remove_bookmark(key: Option<&String>, repo: impl MarksRepository) -> Result<String, String> {
    if key.is_none() {
        return Err("remove requires key argument".to_string());
    }

    let key_uw = key.unwrap();

    let mut bookmarks = repo.get_bookmarks();
    let removed = bookmarks.remove(key_uw);
    repo.store_bookmarks(bookmarks);
    if removed.is_some() {
        return Ok(format!("Removed {}\n", key_uw));
    } else {
        return Err(format!("No bookmark named {} found", key_uw));
    }
}

#[cfg(test)]
mod test {
    use crate::run_cmd;
    use crate::MarksRepository;
    use crate::HELP;
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

        fn clear_marks(&self) {}
    }

    fn mock_repo() -> impl MarksRepository {
        MockRepo {}
    }

    #[test]
    fn should_print_help() {
        let result = run_cmd(vec!["bin".to_string(), "-h".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HELP.to_string());
    }

    #[test]
    fn should_fail_with_unknown_flag() {
        let result = run_cmd(vec!["bin".to_string(), "-unknown".to_string()], mock_repo());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unkown command for mark command");
    }

    #[test]
    fn should_list_marks() {
        let result = run_cmd(vec!["bin".to_string(), "-ls".to_string()], mock_repo());

        let out = concat!(
            "\n# Marks\n\n",
            "0     : /dir\n",
            "1     : /dir/two\n",
            "\n# Bookmarks\n\n",
            "a     : /dir\n",
            "b     : /dir/two\n\n"
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), out);
    }

    #[test]
    fn should_add_mark() {
        let result = run_cmd(vec!["bin".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Marked as 5\n");
    }

    #[test]
    fn should_add_mark_with_path() {
        let result = run_cmd(vec!["bin".to_string(), "dir".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Marked as 5\n");
    }

    #[test]
    fn should_get_mark() {
        let result = run_cmd(
            vec!["bin".to_string(), "-g".to_string(), "0".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/dir");
    }

    #[test]
    fn should_fail_to_get_mark_without_key_arg() {
        let result = run_cmd(vec!["bin".to_string(), "-g".to_string()], mock_repo());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Get command requires key argument\n");
    }

    #[test]
    fn should_add_bookmark_for_current_dir() {
        let result = run_cmd(
            vec!["bin".to_string(), "-b".to_string(), "cd".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Bookmarked as cd\n");
    }

    #[test]
    fn should_remove_bookmark() {
        let result = run_cmd(
            vec!["bin".to_string(), "-rm".to_string(), "b".to_string()],
            mock_repo(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Removed b\n");
    }

    #[test]
    fn should_clear_marks() {
        let result = run_cmd(vec!["bin".to_string(), "-clr".to_string()], mock_repo());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cleared marks\n");
    }
}
