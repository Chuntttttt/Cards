use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from cards-cli to workspace root
    path
}

#[test]
fn test_cli_generates_pdf_from_static_cards() {
    let root = workspace_root();
    let output_path = root.join("test_output.pdf");

    // Clean up any previous test output
    let _ = fs::remove_file(&output_path);

    // Run the CLI tool (now in cards-cli package)
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(&[
            "run",
            "-p",
            "cards-cli",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            "test_output.pdf",
            "--sides",
            "3",
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(status.success(), "CLI tool should execute successfully");

    // Verify the PDF was created
    assert!(output_path.exists(), "Output PDF should be created");

    // Verify the file is not empty
    let metadata = fs::metadata(&output_path).expect("Failed to read PDF metadata");
    assert!(metadata.len() > 0, "Output PDF should not be empty");

    // Clean up
    fs::remove_file(&output_path).expect("Failed to clean up test output");
}

#[test]
fn test_cli_with_different_grid_sizes() {
    let root = workspace_root();
    let output_path = root.join("test_output_5x5.pdf");

    // Clean up any previous test output
    let _ = fs::remove_file(&output_path);

    // Run the CLI tool with 5x5 grid
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(&[
            "run",
            "-p",
            "cards-cli",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            "test_output_5x5.pdf",
            "--sides",
            "5",
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(
        status.success(),
        "CLI tool should execute successfully with 5x5 grid"
    );

    // Verify the PDF was created
    assert!(output_path.exists(), "Output PDF should be created");

    // Clean up
    fs::remove_file(&output_path).expect("Failed to clean up test output");
}

#[test]
fn test_cli_verbose_flag() {
    let root = workspace_root();
    let output_path = root.join("test_output_verbose.pdf");

    // Clean up any previous test output
    let _ = fs::remove_file(&output_path);

    // Run the CLI tool with verbose flag
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(&[
            "run",
            "-p",
            "cards-cli",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            "test_output_verbose.pdf",
            "--sides",
            "3",
            "--verbose",
        ])
        .output()
        .expect("Failed to execute cargo run");

    assert!(
        output.status.success(),
        "CLI tool should execute successfully with verbose flag"
    );

    // Verbose mode should produce some output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    assert!(
        combined_output.contains("Added image") || combined_output.contains("PDF saved to"),
        "Verbose output should contain progress messages"
    );

    // Clean up
    fs::remove_file(&output_path).expect("Failed to clean up test output");
}

#[test]
fn test_library_api_directly() {
    use cards_core::CardWriter;

    let root = workspace_root();
    let cards_path = root.join("static/cards").to_string_lossy().to_string();
    let output_path = root.join("test_output_lib_api.pdf");

    let writer = CardWriter::new(cards_path, 3).unwrap();

    // Clean up any previous test output
    let _ = fs::remove_file(&output_path);

    let result = writer.create_pdf(output_path.to_str().unwrap());
    assert!(result.is_ok(), "Library API should create PDF successfully");

    // Verify the PDF was created
    assert!(output_path.exists(), "Output PDF should be created");

    // Clean up
    fs::remove_file(&output_path).expect("Failed to clean up test output");
}

#[test]
fn test_library_api_bytes_output() {
    use cards_core::CardWriter;

    let root = workspace_root();
    let cards_path = root.join("static/cards").to_string_lossy().to_string();

    let writer = CardWriter::new(cards_path, 3).unwrap();

    let result = writer.generate_pdf_bytes();
    assert!(
        result.is_ok(),
        "Library API should generate PDF bytes successfully"
    );

    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "PDF bytes should not be empty");
    assert_eq!(&bytes[0..5], b"%PDF-", "Should be a valid PDF");
}
