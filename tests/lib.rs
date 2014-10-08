extern crate in_order;

use std::io::fs::PathExtensions;
use in_order::config::{Config, Do, Undo};
use std::io::{File, Command, TempDir};


static CONFIG_0: &'static str =
r##"command = "sh"
root = "tests/sequence"
[special]
[special.3]
command = "special"
current_action = 0
"##;

static CONFIG_3: &'static str =
r##"command = "sh"
root = "tests/sequence"
[special]
[special.3]
command = "special"
current_action = 3
"##;

fn run_command(cmd: &str, args: &[&str]) {
    Command::new(cmd).args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn save_config() -> TempDir {
    let path = Path::new("tests/sequence/do.toml");
    let dir = TempDir::new_in(&Path::new("/tmp"), "in-order-test").unwrap();
    {
        let args = vec![path.as_str().unwrap(), dir.path().as_str().unwrap()];
        run_command("cp", args.as_slice());
    }

    dir
}

fn read(path: &str) -> String {
    File::open(&Path::new(path)).read_to_string().unwrap()
}


#[test]
fn read_config() {
    let path = Some("tests/sequence/do.toml".to_string());
    let config = Config::read(path).unwrap();
    let action = config.actions[0].clone();

    assert_eq!(action.command, "sh".to_string());
    assert_eq!(action.name, "1-first".to_string());
    assert_eq!(action.do_file.unwrap().filename_str().unwrap(), "do.sh");
    assert_eq!(action.undo_file.unwrap().filename_str().unwrap(), "undo.sh")
}

#[test]
fn special_command() {
    let path = Some("tests/sequence/do.toml".to_string());
    let config = Config::read(path).unwrap();
    let action = config.actions.last().unwrap().clone();

    assert_eq!(action.command, "special".to_string());
}

#[test]
fn read_config_without_path_and_without_config_in_current_dir() {
    let config = Config::read(None);

    assert!(config.is_err())
}

#[test]
fn perform_do_undo() {
    let tmp_config = save_config();
    let path = tmp_config.path().join("do.toml");
    let path = path.as_str().unwrap();

    let mut config = Config::read(Some(path.to_string())).unwrap();

    config.perform(Do);

    assert!(Path::new("/tmp/in-order-test-file-1").exists());
    assert!(Path::new("/tmp/in-order-test-file-2").exists());
    assert_eq!(read(path).as_slice(), CONFIG_3);

    config.perform(Undo);

    assert!(!Path::new("/tmp/in-order-test-file-1").exists());
    assert!(!Path::new("/tmp/in-order-test-file-2").exists());
    assert_eq!(read(path).as_slice(), CONFIG_0);
}
