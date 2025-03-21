extern crate dotenv_linter;

use assert_cmd::Command;
use std::fs::File;
use std::io::Write;
use tempfile::{tempdir, tempdir_in};

mod common;

#[test]
fn checks_current_dir() {
    let current_dir = tempdir().unwrap();
    let file_path = current_dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}:1 The FOO key should be with a value or have an equal sign\n",
            file_path.file_name().unwrap().to_str().unwrap()
        ));

    drop(file);
    current_dir.close().unwrap();
}

#[test]
fn checks_current_dir_with_dot_arg() {
    let current_dir = tempdir().unwrap();
    let file_path = current_dir.path().join("test.env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "foo=").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .arg(".")
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}:1 The foo key should be in uppercase\n",
            file_path.file_name().unwrap().to_str().unwrap()
        ));

    drop(file);
    current_dir.close().unwrap();
}

#[test]
fn checks_one_specific_path() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let dir1 = tempdir_in(&current_dir).unwrap();
    let file_path2 = dir1.path().join(".env.test");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, "1FOO=").unwrap();

    let relative_path = common::relative_path(current_dir.path(), dir1.path());

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .arg(dir1.path())
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}/{}:1 Invalid leading character detected\n",
            relative_path.to_str().unwrap(),
            file_path2.file_name().unwrap().to_str().unwrap()
        ));

    drop(file2);
    drop(file1);
    dir1.close().unwrap();
    current_dir.close().unwrap();
}

#[test]
fn checks_two_specific_paths() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let dir1 = tempdir_in(&current_dir).unwrap();
    let file_path2 = dir1.path().join(".env");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, " FOO=").unwrap();

    let dir2 = tempdir_in(&dir1).unwrap();
    let file_path3 = dir2.path().join(".env");
    let mut file3 = File::create(&file_path3).unwrap();
    writeln!(file3, " FOO=").unwrap();

    let relative_path1 = common::relative_path(current_dir.path(), dir1.path());
    let relative_path2 = common::relative_path(dir1.path(), dir2.path());

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&[dir1.path(), dir2.path()])
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}/{}:1 Invalid leading character detected\n{}/{}/{}:1 Invalid leading character detected\n",
            relative_path1.to_str().unwrap(),
            file_path2.file_name().unwrap().to_str().unwrap(),
            relative_path1.to_str().unwrap(),
            relative_path2.to_str().unwrap(),
            file_path3.file_name().unwrap().to_str().unwrap(),
        ));

    drop(file3);
    drop(file2);
    drop(file1);
    dir2.close().unwrap();
    dir1.close().unwrap();
    current_dir.close().unwrap();
}

#[test]
fn checks_one_specific_file() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let file_path2 = current_dir.path().join("test-env-file");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, "FOO =").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .arg(&file_path2)
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}:1 The line has spaces around equal sign\n",
            file_path2.file_name().unwrap().to_str().unwrap()
        ));

    drop(file2);
    drop(file1);
    current_dir.close().unwrap();
}

#[test]
fn checks_two_specific_files() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let file_path2 = current_dir.path().join("test-env-file");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, "FOO =").unwrap();

    let dir1 = tempdir_in(&current_dir).unwrap();
    let file_path3 = dir1.path().join("another_test_file");
    let mut file3 = File::create(&file_path3).unwrap();
    writeln!(file3, "FOO=BAR\nFOO=BAR").unwrap();

    let relative_path = common::relative_path(current_dir.path(), dir1.path());

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&[&file_path2, &file_path3])
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}/{}:2 The FOO key is duplicated\n{}:1 The line has spaces around equal sign\n",
            relative_path.to_str().unwrap(),
            file_path3.file_name().unwrap().to_str().unwrap(),
            file_path2.file_name().unwrap().to_str().unwrap(),
        ));

    drop(file3);
    drop(file2);
    drop(file1);
    dir1.close().unwrap();
    current_dir.close().unwrap();
}

#[test]
fn checks_one_specific_file_and_one_path() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let file_path2 = current_dir.path().join("test.env.test");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, "FOO=BAR\nBAR=FOO").unwrap();

    let dir1 = tempdir_in(&current_dir).unwrap();
    let file_path3 = dir1.path().join("test.env");
    let mut file3 = File::create(&file_path3).unwrap();
    writeln!(file3, "FOO=BAR\nFOO=BAR").unwrap();

    let relative_path = common::relative_path(current_dir.path(), dir1.path());

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&[&file_path2, dir1.path()])
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}/{}:2 The FOO key is duplicated\n{}:2 UnorderedKey: The BAR key should go before the FOO key\n",
            relative_path.to_str().unwrap(),
            file_path3.file_name().unwrap().to_str().unwrap(),
            file_path2.file_name().unwrap().to_str().unwrap(),
        ));

    drop(file3);
    drop(file2);
    drop(file1);
    dir1.close().unwrap();
    current_dir.close().unwrap();
}

#[test]
fn checks_one_specific_file_twice() {
    let current_dir = tempdir().unwrap();
    let file_path1 = current_dir.path().join(".env");
    let mut file1 = File::create(file_path1).unwrap();
    writeln!(file1, "foo=").unwrap();

    let file_path2 = current_dir.path().join("test-env-file");
    let mut file2 = File::create(&file_path2).unwrap();
    writeln!(file2, "1FOO=").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&[&file_path2, &file_path2])
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}:1 Invalid leading character detected\n",
            file_path2.file_name().unwrap().to_str().unwrap()
        ));

    drop(file2);
    drop(file1);
    current_dir.close().unwrap();
}

#[test]
fn exits_with_0_on_no_errors() {
    let current_dir = tempdir().unwrap();
    let file_path = current_dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir).assert().success();

    drop(file);
    current_dir.close().unwrap();
}

#[test]
fn exclude_one_file() {
    let current_dir = tempdir().unwrap();
    let file_path = current_dir.path().join(".env");
    let file_path_str = file_path.to_str().unwrap();
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, " FOO=").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&["--exclude", file_path_str])
        .assert()
        .success();

    drop(file);
    current_dir.close().unwrap();
}

#[test]
fn exclude_two_files() {
    let current_dir = tempdir().unwrap();
    let file_1_path = current_dir.path().join(".env");
    let file_1_path_str = file_1_path.to_str().unwrap();
    let mut file_1 = File::create(&file_1_path).unwrap();
    writeln!(file_1, " FOO=").unwrap();

    let file_2_path = current_dir.path().join(".local.env");
    let file_2_path_str = file_2_path.to_str().unwrap();
    let mut file_2 = File::create(&file_2_path).unwrap();
    writeln!(file_2, " BAR=").unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&["-e", file_1_path_str, "-e", file_2_path_str])
        .assert()
        .success();

    drop(file_1);
    drop(file_2);
    current_dir.close().unwrap();
}

#[test]
fn exclude_one_file_check_one_file() {
    let current_dir = tempdir().unwrap();
    let file_to_check_path = current_dir.path().join(".env");
    let mut file_to_check = File::create(&file_to_check_path).unwrap();
    writeln!(file_to_check, " FOO=").unwrap();

    let file_to_exclude_path = current_dir.path().join(".exclude-me.env");
    let mut file_to_exclude = File::create(&file_to_exclude_path).unwrap();
    writeln!(file_to_exclude, " BAR=").unwrap();
    let file_to_exclude_str = file_to_exclude_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&current_dir)
        .args(&["--exclude", file_to_exclude_str])
        .assert()
        .failure()
        .code(1)
        .stdout(format!(
            "{}:1 Invalid leading character detected\n",
            file_to_check_path.file_name().unwrap().to_str().unwrap()
        ));

    drop(file_to_exclude);
    drop(file_to_check);
    current_dir.close().unwrap();
}
