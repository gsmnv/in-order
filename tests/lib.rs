extern crate in_order;

use std::io::fs::PathExtensions;
use in_order::config::{Config, Do, Undo};
use std::io::{File, Command};

fn run_command(cmd: &str, args: &[&str]) {
    let p = Command::new(cmd).args(args).spawn();

    match p {
        Err(error) => fail!(error),
        Ok(mut p)  => match p.wait() {
            Err(error) => fail!(error),
            Ok(_)      => ()
        }
    };
}

fn save_config(path: &str) {
    let args = vec![path, "/tmp/do.toml"];

    run_command("cp", args.as_slice())
}

fn restore_config(path: &str) {
    let args = vec!["/tmp/do.toml", path];

    run_command("mv", args.as_slice())
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
    let config_0 =
        "command = \"sh\"\nroot = \"tests/sequence\"\n[special]\n[special.3]\ncommand = \"special\"\ncurrent_action = 0\n";
    let config_3 =
        "command = \"sh\"\nroot = \"tests/sequence\"\n[special]\n[special.3]\ncommand = \"special\"\ncurrent_action = 3\n";
    let path = "tests/sequence/do.toml";
    let mut config = Config::read(Some(path.to_string())).unwrap();

    save_config(path);

    config.perform(Do);

    assert!(Path::new("/tmp/in-order-test-file-1").exists());
    assert!(Path::new("/tmp/in-order-test-file-2").exists());
    assert_eq!(read(path).as_slice(), config_3);

    config.perform(Undo);

    assert!(!Path::new("/tmp/in-order-test-file-1").exists());
    assert!(!Path::new("/tmp/in-order-test-file-2").exists());
    assert_eq!(read(path).as_slice(), config_0);

    restore_config(path);
}
