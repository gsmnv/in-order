extern crate toml;
extern crate collections;

use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::{Process, Command};
use std::collections::treemap::TreeMap;

#[deriving(Clone)]
pub struct Action {
    pub name: String,
    pub command: String,
    pub do_file: Option<Path>,
    pub undo_file: Option<Path>
}

impl Action {
    pub fn do_command(&self) -> Result<String, String> {
        execute_command(&self.do_file, &self.command, "No 'do' file")
    }

    pub fn undo_command(&self) -> Result<String, String> {
        execute_command(&self.undo_file, &self.command, "No 'undo' file")
    }

    pub fn find_actions(root: &Path, cmd: &String, special: &TreeMap<String, toml::Value>) -> Result<Vec<Action>, &'static str> {
        let dirs = try!(select_files(root, |_, path| valid_action_dir(path)));
        let mut actions: Vec<Action> = Vec::with_capacity(dirs.len());

        for dir in dirs.iter() {
            let do_file = try!(select_files(dir, |name, _| name.starts_with("do"))).into_iter().next();

            let undo_file = try!(select_files(dir, |name, _| name.starts_with("undo"))).into_iter().next();

            let name = dir.filename_str().unwrap().to_string();

            let command = match special.find(&number(&name).to_string()) {
                None          => cmd.clone(),
                Some(v) => match v.lookup("command").map_or(None, |c| c.as_str()) {
                    Some(value) => value.to_string(),
                    None        => return Err("'special' is invalid")
                },
            };

            actions.push(Action {
                name: name,
                command: command,
                do_file: do_file,
                undo_file: undo_file
            });
        }

        actions.sort_by(|a, b| number(&a.name).cmp(&number(&b.name)));

        Ok(actions)
    }
}

fn number(name: &String) -> uint {
    let mut result = 0;

    for c in name.as_slice().chars() {
        if c.is_digit() {
            result += c.to_digit(10).unwrap();
        } else {
            break;
        }
    }

    result
}


fn handle_output(process: Process) -> Result<String, String> {
    match process.wait_with_output() {
        Err(error) => Err(error.desc.to_string()),
        Ok(exit) => {
            if exit.status.success() {
                Ok(exit.output.into_ascii().into_string())
            } else {
                Err(exit.error.into_ascii().into_string())
            }
        }
    }
}

fn execute_command(file: &Option<Path>, cmd: &String, error: &'static str) -> Result<String, String> {
    let path = match file {
        &Some(ref path) => path,
        &None           => return Ok(error.to_string())
    };

    let command = parse_command(cmd.as_slice(), path);
    let process = Command::new(command[0])
        .cwd(&path.dir_path())
        .args(command.tail())
        .spawn();

    match process {
        Ok(p)      => handle_output(p),
        Err(error) => Err(error.desc.to_string())
    }
}

fn parse_command<'a>(cmd: &'a str, path: &'a Path) -> Vec<&'a str> {
    let mut words: Vec<&str> = cmd.words().collect();

    words.push(path.filename_str().unwrap());
    words
}

fn valid_action_dir(path: &Path) -> bool {
    let name = path.filename_str().unwrap_or("");
    let valid_name = name.graphemes(true).next()
        .map_or(false, |s| s.char_at(0).is_digit());

    valid_name && path.is_dir()
}

fn select_files(dir: &Path, p: |&str, &Path| -> bool) -> Result<Vec<Path>, &'static str> {
    let paths = match fs::readdir(dir) {
        Ok(paths)  => paths,
        Err(error) => return Err(error.desc)
    };

    let result = paths.iter().filter_map(|path| {
        let name = path.filename_str().unwrap_or("");

        if p(name, path) {
            Some(path.clone())
        } else {
            None
        }
    }).collect();

    Ok(result)
}
