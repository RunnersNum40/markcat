use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use predicates::str::{contains, is_match};
use std::fs;

fn mk_basic_tree() -> assert_fs::TempDir {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("a.rs").write_str("fn main() {}\n").unwrap();
    tmp.child("README.md").write_str("# readme\n").unwrap();
    tmp.child("LICENSE").write_str("license text\n").unwrap();
    tmp.child("script").write_str("echo hi\n").unwrap();
    tmp
}

#[test]
fn positional_and_flag_path() {
    let tmp = mk_basic_tree();
    let dir = tmp.path().to_str().unwrap();

    Command::cargo_bin("markcat")
        .unwrap()
        .arg(dir)
        .assert()
        .success()
        .stdout(contains("a.rs").and(contains("README.md")));

    Command::cargo_bin("markcat")
        .unwrap()
        .args(["-p", dir])
        .assert()
        .success()
        .stdout(contains("a.rs").and(contains("README.md")));
}

#[test]
fn whitelist_ext_filename_and_noext() {
    let tmp = mk_basic_tree();
    let dir = tmp.path().to_str().unwrap();

    Command::cargo_bin("markcat")
        .unwrap()
        .args(["-w", "rs,LICENSE,noext", dir])
        .assert()
        .success()
        .stdout(contains("a.rs"))
        .stdout(contains("LICENSE"))
        .stdout(contains("script"))
        .stdout(is_match("README\\.md").unwrap().not());
}

#[test]
fn blacklist_ext_and_noext() {
    let tmp = mk_basic_tree();
    let dir = tmp.path().to_str().unwrap();

    Command::cargo_bin("markcat")
        .unwrap()
        .args(["-b", "md,noext", dir])
        .assert()
        .success()
        .stdout(contains("a.rs"))
        .stdout(is_match("README\\.md").unwrap().not())
        .stdout(contains("script").not());
}

#[test]
fn gitignore_respected_unless_ignored() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child(".gitignore").write_str("*.log\n").unwrap();
    tmp.child("app.log").write_str("x\n").unwrap();
    tmp.child("keep.txt").write_str("y\n").unwrap();
    let dir = tmp.path().to_str().unwrap();

    Command::cargo_bin("markcat")
        .unwrap()
        .arg(dir)
        .assert()
        .success()
        .stdout(contains("keep.txt"))
        .stdout(contains("app.log").not());

    Command::cargo_bin("markcat")
        .unwrap()
        .args(["-i", dir])
        .assert()
        .success()
        .stdout(contains("app.log"));
}

#[test]
fn output_to_file() {
    let tmp = mk_basic_tree();
    let out = tmp.child("out.md");
    let dir = tmp.path().to_str().unwrap();

    Command::cargo_bin("markcat")
        .unwrap()
        .args(["-o", out.path().to_str().unwrap(), dir])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let written = fs::read_to_string(out.path()).unwrap();
    assert!(written.contains("a.rs"));
    assert!(written.contains("README.md"));
}

#[test]
fn trim_flag_trims_contents() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("X.txt").write_str("  hi  \n").unwrap();
    let dir = tmp.path().to_str().unwrap();

    let raw = Command::cargo_bin("markcat")
        .unwrap()
        .arg(dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let raw_s = String::from_utf8(raw).unwrap();
    assert!(raw_s.contains("  hi  "));

    let trimmed = Command::cargo_bin("markcat")
        .unwrap()
        .args(["-t", dir])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let trimmed_s = String::from_utf8(trimmed).unwrap();
    assert!(trimmed_s.contains("\nhi\n"));
    assert!(!trimmed_s.contains("  hi  "));
}
