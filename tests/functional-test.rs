use std::env;
use std::process::Command;

#[test]
fn test_first() {
    let mut path = env::current_dir().unwrap();
    path.push("tests");
    path.push("functional-test.bash");
    let status = Command::new("bash").args([path.to_str().unwrap()]).status();
    assert!(status.is_ok());
    if status.is_ok() {
        assert!(status.unwrap().success());
    }
}
