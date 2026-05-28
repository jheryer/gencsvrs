use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
type TestResult = Result<(), Box<dyn Error>>;

const NAME: &str = "synthtab";
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

#[test]
fn test_er_subcommand_listed_in_help() -> TestResult {
    Command::cargo_bin(NAME)?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("er"));
    Ok(())
}

#[test]
fn test_er_generates_csv_files_for_valid_fixture() -> TestResult {
    let out_dir = std::env::temp_dir().join("synthtab_cli_er_fixture_test");
    let _ = fs::remove_dir_all(&out_dir);
    Command::cargo_bin(NAME)?
        .args([
            "er",
            "tests/fixtures/er/car_person.mmd",
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("wrote"))
        .stderr(predicate::str::contains("PERSON"))
        .stderr(predicate::str::contains("CAR"));
    assert!(out_dir.join("PERSON.csv").exists(), "PERSON.csv missing");
    assert!(out_dir.join("CAR.csv").exists(), "CAR.csv missing");
    let _ = fs::remove_dir_all(&out_dir);
    Ok(())
}

#[test]
fn test_er_generates_junction_for_many_to_many() -> TestResult {
    let out_dir = std::env::temp_dir().join("synthtab_cli_er_mn_test");
    let _ = fs::remove_dir_all(&out_dir);
    Command::cargo_bin(NAME)?
        .args([
            "er",
            "tests/fixtures/er/student_course_mn.mmd",
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("STUDENT"))
        .stderr(predicate::str::contains("COURSE"))
        .stderr(predicate::str::contains("STUDENT_COURSE"));
    assert!(out_dir.join("STUDENT.csv").exists(), "STUDENT.csv missing");
    assert!(out_dir.join("COURSE.csv").exists(), "COURSE.csv missing");
    assert!(
        out_dir.join("STUDENT_COURSE.csv").exists(),
        "STUDENT_COURSE.csv missing"
    );
    let _ = fs::remove_dir_all(&out_dir);
    Ok(())
}

#[test]
fn test_er_rejects_unsupported_glyph_with_line_number() -> TestResult {
    Command::cargo_bin(NAME)?
        .args(["er", "tests/fixtures/er/invalid_glyph.mmd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("non-identifying"))
        .stderr(predicate::str::contains("line 8"));
    Ok(())
}

#[test]
fn test_er_missing_file_returns_error() -> TestResult {
    Command::cargo_bin(NAME)?
        .args(["er", "/nonexistent/diagram.mmd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read"));
    Ok(())
}

#[test]
fn test_flat_mode_still_works_after_subcommand_refactor() -> TestResult {
    // Regression guard: adding the `er` subcommand must not break the
    // existing flag-based mode.
    Command::cargo_bin(NAME)?
        .args(["-s", "id:INT_INC,name:VALUE", "-r", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("id,name"));
    Ok(())
}

#[test]
fn test_target_postgres_writes_ddl_file() -> TestResult {
    let data = std::env::temp_dir().join("synthtab_ddl_test_users.csv");
    let ddl = std::env::temp_dir().join("synthtab_ddl_test_users.ddl.postgres.sql");
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&ddl);
    Command::cargo_bin(NAME)?
        .args([
            "-s",
            "id:INT_INC,name:STRING,joined:DATE",
            "-r",
            "3",
            "-c",
            "-f",
            data.to_str().unwrap(),
            "--target",
            "postgres",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("ddl.postgres.sql"));
    assert!(data.exists(), "data file missing");
    assert!(ddl.exists(), "DDL file missing");
    let golden = fs::read_to_string("tests/fixtures/dialects/users.ddl.postgres.sql")?;
    let actual = fs::read_to_string(&ddl)?;
    assert_eq!(actual, golden, "DDL content mismatch");
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&ddl);
    Ok(())
}

#[test]
fn test_target_without_file_target_is_rejected() -> TestResult {
    Command::cargo_bin(NAME)?
        .args(["-s", "id:INT_INC", "-r", "3", "--target", "mysql"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--target requires --file-target"));
    Ok(())
}

#[test]
fn test_target_postgres_writes_load_file() -> TestResult {
    let data = std::env::temp_dir().join("synthtab_load_test_users.csv");
    let load = std::env::temp_dir().join("synthtab_load_test_users.load.postgres.sql");
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&load);
    Command::cargo_bin(NAME)?
        .args([
            "-s",
            "id:INT_INC,name:STRING",
            "-r",
            "2",
            "-c",
            "-f",
            data.to_str().unwrap(),
            "--target",
            "postgres",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("load.postgres.sql"));
    assert!(load.exists(), "load file missing");
    let content = fs::read_to_string(&load)?;
    assert!(
        content.contains("\\copy"),
        "expected postgres \\copy: {content}"
    );
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&load);
    Ok(())
}

#[test]
fn test_no_load_suppresses_load_file() -> TestResult {
    let data = std::env::temp_dir().join("synthtab_no_load_test.csv");
    let load = std::env::temp_dir().join("synthtab_no_load_test.load.mysql.sql");
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&load);
    Command::cargo_bin(NAME)?
        .args([
            "-s",
            "id:INT_INC",
            "-r",
            "2",
            "-c",
            "-f",
            data.to_str().unwrap(),
            "--target",
            "mysql",
            "--no-load",
        ])
        .assert()
        .success();
    assert!(!load.exists(), "load file should not exist with --no-load");
    let _ = fs::remove_file(&data);
    Ok(())
}

#[test]
fn test_er_target_postgres_writes_ddl_and_load() -> TestResult {
    let out_dir = std::env::temp_dir().join("synthtab_cli_er_target_test");
    let _ = fs::remove_dir_all(&out_dir);
    Command::cargo_bin(NAME)?
        .args([
            "er",
            "tests/fixtures/er/car_person.mmd",
            "--out",
            out_dir.to_str().unwrap(),
            "--target",
            "postgres",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("schema.ddl.postgres.sql"))
        .stderr(predicate::str::contains("load.postgres.sql"));
    assert!(
        out_dir.join("schema.ddl.postgres.sql").exists(),
        "DDL missing"
    );
    let ddl = fs::read_to_string(out_dir.join("schema.ddl.postgres.sql"))?;
    assert!(
        ddl.contains("CREATE TABLE"),
        "DDL should have CREATE TABLE: {ddl}"
    );
    assert!(
        ddl.contains("FOREIGN KEY"),
        "DDL should have FK constraints: {ddl}"
    );
    let _ = fs::remove_dir_all(&out_dir);
    Ok(())
}

#[test]
fn test_no_ddl_suppresses_ddl_file() -> TestResult {
    let data = std::env::temp_dir().join("synthtab_no_ddl_test.csv");
    let ddl = std::env::temp_dir().join("synthtab_no_ddl_test.ddl.mysql.sql");
    let _ = fs::remove_file(&data);
    let _ = fs::remove_file(&ddl);
    Command::cargo_bin(NAME)?
        .args([
            "-s",
            "id:INT_INC",
            "-r",
            "2",
            "-c",
            "-f",
            data.to_str().unwrap(),
            "--target",
            "mysql",
            "--no-ddl",
        ])
        .assert()
        .success();
    assert!(data.exists(), "data file missing");
    assert!(!ddl.exists(), "DDL file should not exist with --no-ddl");
    let _ = fs::remove_file(&data);
    Ok(())
}
