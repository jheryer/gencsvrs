use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::error::Error;
use std::fs;
type TestResult = Result<(), Box<dyn Error>>;

const NAME: &str = "gencsv";
fn run(args: &[&str], expected_file: &str) -> TestResult {
    let expected = fs::read_to_string(expected_file)?;
    Command::cargo_bin(NAME)?
        .args(args)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

#[test]
fn test_help() -> TestResult {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(NAME)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("Usage"));
    }
    Ok(())
}

#[test]
fn test_default_output_no_parameters() -> TestResult {
    run(&[], "tests/expected/csv-default-output.txt")
}

#[test]
fn test_default_output_with_3_rows() -> TestResult {
    run(&["-r", "3"], "tests/expected/csv-default-r3-output.txt")
}

#[test]
fn test_default_output_with_20_rows_and_pipe_delimiter() -> TestResult {
    run(
        &["-r", "20", "-d", "|"],
        "tests/expected/csv-default-r20-pipe-output.txt",
    )
}
