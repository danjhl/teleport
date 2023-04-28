use home;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::prelude::*;

static DATA_PATH: &'static str = ".cache/teleport/";
static MARKED: &'static str = "marked";
static BOOKMARKED: &'static str = "bookmarked";

static HELP: &'static str = r#"
Usage: tp [CMD] [ARGS]

help            Show usage
ls              List marked directories
"#;

static ERR_NO_CMD: &'static str = "No command given. Run 'tp help' for more.";

struct Repository {}

trait MarksRespository {
    fn get_marks(&self) -> HashMap<String, String>;
}

impl MarksRespository for Repository {
    fn get_marks(&self) -> HashMap<String, String> {
        let home_dir = home::home_dir().expect("Unable to get home directory");

        let mut data_path = PathBuf::new();
        data_path.push(home_dir.as_path());
        data_path.push(DATA_PATH);

        let mut marked_path = PathBuf::new();
        marked_path.push(&data_path);
        marked_path.push(Path::new(MARKED));

        let file = OpenOptions::new()
            .read(true)
            .open(marked_path)
            .unwrap();

        let lines = io::BufReader::new(file).lines();
        let mut map = HashMap::new();

        for line in lines {
            let split_o = line.unwrap();
            let mut split = split_o.split(",");
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
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let repo = Repository {};
    let result = run_cmd(args, repo);

    match result {
        Ok(out) => print!("{}", out),
        Err(msg) => eprintln!("{}", msg)
    }
}

fn run_cmd(args: Vec<String>, repo: impl MarksRespository) -> Result<String, String> {
    let cmd = match args.get(1) {
        Some(s) => s,
        None => return Err(ERR_NO_CMD.to_string())
    };

    if cmd == "help" {
        return Ok(HELP.to_string())
    }
    if cmd == "ls" {
        return Ok(ls(repo))
    }
 
    Err(format!("Unknown command '{}'", cmd))
}

fn ls(repo: impl MarksRespository) -> String {
    let marks = repo.get_marks();
    let mut out = String::new();
    out.push_str("\n# Marks\n\n");
    for (key, value) in marks {
        out.push_str(&format!("{:width$} : {}\n", key, value, width = 2));
    }
    out.push_str("\n");
    out
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::MarksRespository;
    use crate::run_cmd;
    use crate::{ERR_NO_CMD, HELP};

    struct MockRepo {}

    impl MarksRespository for MockRepo {
        fn get_marks(&self) -> HashMap<String, String> {
            let mut map = HashMap::new();
            map.insert("0".to_string(), "/dir".to_string());
            map
        }
    }

    fn mock_repo() -> impl MarksRespository {
        let mock_repo = MockRepo {};
        mock_repo
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
            "0  : /dir\n\n"
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), out);
    }
    
}