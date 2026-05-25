use assert_cmd::Command;
use predicates::prelude::*;
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
fn test_parquet_without_file_target_fails() -> TestResult {
    // --parquet without --file-target should now exit non-zero with a clear error.
    Command::cargo_bin(NAME)?
        .args(["-s", "a:INT,b:STRING", "-r", "3", "-p"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--file-target"));
    Ok(())
}

#[test]
fn test_int_rng_without_modifier_does_not_panic() -> TestResult {
    // Previously panicked. Now should succeed (falls back to default range)
    // and emit a CSV header row.
    Command::cargo_bin(NAME)?
        .args(["-s", "id:INT_RNG,name:STRING", "-r", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("id,name"));
    Ok(())
}

#[test]
fn test_delete_negative_range_succeeds() -> TestResult {
    // Negative-range delete (e.g. "-2-2") was unsupported by the parser.
    // Use `--delete-target=...` to keep clap from interpreting the leading
    // dash as a short flag.
    Command::cargo_bin(NAME)?
        .args([
            "-s",
            "id:INT_INC,name:STRING",
            "-r",
            "5",
            "--delete-target=-2-2",
        ])
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_bad_schema_returns_error() -> TestResult {
    Command::cargo_bin(NAME)?
        .args(["-s", "totally_invalid_garbage", "-r", "3"])
        .assert()
        .failure();
    Ok(())
}
