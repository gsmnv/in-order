extern crate toml;
extern crate collections;

use std::io;
use std::io::{File, Open, ReadWrite};
use std::io::fs::PathExtensions;
use std::collections::treemap::TreeMap;
use self::action::Action;

pub mod action;


#[deriving(PartialEq, Show)]
pub enum Direction {
    Do,
    Undo
}

pub struct Config {
    pub actions: Vec<Action>,
    pub current_action: uint,
    path: Path
}

impl Config {
    pub fn read(path: Option<String>) -> Result<Config, &'static str> {
        let path = try!(decide_config_path(&path));

        let toml = match File::open(&path).read_to_string() {
            Err(error) => return Err(error.desc),
            Ok(file)   => from_str(file.as_slice()).unwrap()
        };

        let root = try!(lookup_root(&toml));

        let current_action = try!(lookup_current_action(&toml));

        let command = try!(lookup_command(&toml));

        let special = try!(lookup_special(&toml));

        let actions = try!(Action::find_actions(&root, &command, &special));

        Ok(Config {
            actions: actions,
            current_action: current_action,
            path: path
        })
    }

    pub fn set_current_action(&mut self, current_action: uint) -> Result<(), &'static str> {
        let mut exists = false;
        let ca_string = format!("current_action = {}", current_action);

        let mut file = match File::open_mode(&self.path, Open, ReadWrite) {
            Ok(file)   => file,
            Err(error) => return Err(error.desc)
        };

        let mut config = file.read_to_string().unwrap().as_slice().lines().map(|line| {
            if is_current_action(line) {
                exists = true;
                ca_string.as_slice()
            } else {
                line
            }
        }).collect::<Vec<&str>>().connect("\n");

        config.push('\n');

        if !exists {
            config.push_str(ca_string.as_slice());
            config.push('\n');
        }

        match file.seek(0, io::SeekSet) {
            Err(error) => Err(error.desc),
            Ok(_)      => match file.write(config.as_bytes()) {
                Err(error) => Err(error.desc),
                Ok(_)      => { self.current_action = current_action; Ok(()) }
            }
        }
    }

    pub fn perform(&mut self, direction: Direction) {
        let actions_count = self.actions.len();
        let mut current_action = self.current_action;

        if current_action > actions_count {
            println!("'current_action' is invalid");
            return
        } else if (direction == Do && current_action == actions_count) || (direction == Undo && current_action == 0) {
            println!("Nothing to do here");
            return
        }

        {
            // Borrow actions mutably for this scope
            let ref mut actions = self.actions;

            let not_yet_performed_actions = match direction {
                Do   => actions.slice_from_mut(current_action),
                Undo => {
                    let actions = actions.slice_mut(0, current_action);

                    actions.reverse();
                    actions
                }
            };

            for action in not_yet_performed_actions.iter() {
                println!("Performing '{}' of {}", direction, action.name);

                let process = match direction {
                    Do   => action.do_command(),
                    Undo => action.undo_command()
                };

                match process {
                    Err(error) => {
                        println!("{}", error);
                        break;
                    },
                    Ok(output) => {
                        println!("{}", output);

                        match direction {
                            Do   => current_action += 1,
                            Undo => current_action -= 1
                        }
                    }
                }
            }
        }

        match self.set_current_action(current_action) {
            Err(error) => println!("{}", error),
            Ok(_)      => ()
        }

    }
}

fn lookup_root(toml: &toml::Value) -> Result<Path, &'static str> {
    let default_root = ".";

    match toml.lookup("root") {
        None        => Ok(Path::new(default_root)),
        Some(value) => match value.as_str() {
            Some(value) => Ok(Path::new(value)),
            None        => Err("'root' is invalid")
        }
    }
}

fn lookup_current_action(toml: &toml::Value) -> Result<uint, &'static str> {
    match toml.lookup("current_action") {
        None        => Ok(0),
        Some(value) => match value.as_integer() {
            Some(value) => Ok(value.to_uint().unwrap()),
            None        => Err("'current_action' is invalid")
        }
    }
}

fn lookup_command(toml: &toml::Value) -> Result<String, &'static str> {
    match toml.lookup("command") {
        None        => Err("'command' is required option"),
        Some(value) => match value.as_str() {
            Some(value) => Ok(value.to_string()),
            None        => Err("'commad' is invalid")
        }
    }
}

fn lookup_special(toml: &toml::Value) -> Result<TreeMap<String, toml::Value>, &'static str> {
    match toml.lookup("special") {
        None        => Ok(TreeMap::new()),
        Some(value) => match value.as_table() {
            Some(value) => Ok(value.clone()),
            None        => Err("'special' is invalid")
        }
    }
}

fn decide_config_path(path: &Option<String>) -> Result<Path, &'static str> {
    match path {
        &Some(ref path) => Ok(Path::new(path.clone())),
        &None           => {
            let path = Path::new("./in-order.toml");

            if path.exists() {
                Ok(path)
            } else {
                Err("Can't find config file in current directory")
            }
        }
    }
}

fn is_current_action(string: &str) -> bool {
    let string: String = string.chars().filter(|c| c != &' ').collect();
    string.as_slice().starts_with("current_action")
}
