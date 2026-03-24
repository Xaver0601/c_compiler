use std::fs;
use std::process::Command;

// Run with 'cargo test -- --nocapture' to also print warnings

// End-To-End test if return value is correct.
#[test]
fn test_valid_programs_e2e() {
  let valid_dir = "tests/fixtures/valid";
  let subdirs = ["format", "unary", "binary"];
  let mut skipped_files = Vec::new();

  for dir in subdirs {
    let set_dir = valid_dir.to_owned() + "/" + dir;

    // Read all files in the subdirectory
    for entry in fs::read_dir(set_dir).expect("File not found!") {
      let path = entry.unwrap().path();

      if path
        .extension()
        .expect("Test file has no extension (add '.c')")
        == "c"
      {
        // 1. Run compiler
        let compile_status = Command::new("cargo")
          .args(["run", "--bin", "compiler", path.to_str().unwrap()])
          .status()
          .unwrap();
        assert!(compile_status.success(), "Failed to compile: {:?}", path);

        // 2. Run the generated binary
        let run_status = Command::new("./temp/temp.out").status().unwrap();

        // 3. Check the exit code (matching file name)    // ### IMPORTANT ###: Return values in Linux are 8-bit unsigned!
        let expected_exit_code = path
          .file_prefix()
          .unwrap()
          .to_str()
          .unwrap()
          .split("_")
          .last()
          .unwrap()
          .parse::<i32>()
          .expect("End-To-End test file must end with an integer");
        assert_eq!(
          run_status.code().unwrap(),
          expected_exit_code,
          "Assertion failed: {:?}", // ':?' uses Debug trait instead of Display trait
          path
        );
      } else {
        skipped_files.push(path);
      }
    }
  }
  if !skipped_files.is_empty() {  // TODO: print this warning at the very end of all tests
    eprintln!("\nWarning: The following non '.c' files were skipped:");
    for skipped in skipped_files {
      eprintln!("  - {}", skipped.display());
    }
  }
}

// Invalid programs should make the compiler throw an error.
#[test]
fn test_invalid_programs_e2e() {
  let invalid_dir = "tests/fixtures/invalid";
  let subdirs = ["format", "unary", "binary"];
  let mut skipped_files = Vec::new();

  for dir in subdirs {
    let set_dir = invalid_dir.to_owned() + "/" + dir;

    // Read all files in the subdirectory
    for entry in fs::read_dir(set_dir).expect("File not found!") {
      let path = entry.unwrap().path();

      if path
        .extension()
        .expect("Test file has no extension (add '.c')")
        == "c"
      {
        // 1. Run compiler
        let compile_status = Command::new("cargo")
          .args(["run", "--bin", "compiler", path.to_str().unwrap()])
          .status()
          .unwrap();
        assert!(
          !compile_status.success(),
          "Invalid file got compiled {:?}",
          path
        );
      } else {
        skipped_files.push(path);
      }
    }
  }
  if !skipped_files.is_empty() {
    eprintln!("\nWarning: The following non '.c' files were skipped:");
    for skipped in skipped_files {
      eprintln!("  - {}", skipped.display());
    }
  }
}
