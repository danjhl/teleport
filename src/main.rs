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
    #Usage: tp [CMD] [ARGS]

    ls              List marked directories\n
"#;

static ERR_NO_CMD: &'static str = "No command given. Run 'tp help' for more.";

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = run_cmd(args);

    match result {
        Ok(out) => print!("{}", out),
        Err(msg) => eprintln!("{}", msg)
    }
}

fn run_cmd(args: Vec<String>) -> Result<String, String> {
    let first = match args.first() {
        Some(s) => s,
        None => return Err(ERR_NO_CMD.to_string())
    };

    if first == "help" {
        return Ok(HELP.to_string())
    }
    
    Err(format!("Unknown command '{}'", first))
}


#[test]
fn should_fail_without_cmd() {   
    let result = run_cmd(vec![]);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ERR_NO_CMD.to_string());
}

#[test]
fn should_print_help() {
    let result = run_cmd(vec!["help".to_string()]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HELP.to_string());
}

#[test]
fn should_fail_with_unknown_cmd() {
    let result = run_cmd(vec!["unknown".to_string()]);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unknown command 'unknown'");
}