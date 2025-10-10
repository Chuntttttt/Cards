use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_cli_generates_pdf_from_static_cards() {
    let output_path = "test_output.pdf";

    // Clean up any previous test output
    let _ = fs::remove_file(output_path);

    // Run the CLI tool
    let status = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            output_path,
            "--sides",
            "3",
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(status.success(), "CLI tool should execute successfully");

    // Verify the PDF was created
    assert!(
        Path::new(output_path).exists(),
        "Output PDF should be created"
    );

    // Verify the file is not empty
    let metadata = fs::metadata(output_path).expect("Failed to read PDF metadata");
    assert!(metadata.len() > 0, "Output PDF should not be empty");

    // Clean up
    fs::remove_file(output_path).expect("Failed to clean up test output");
}

#[test]
fn test_cli_with_different_grid_sizes() {
    let output_path = "test_output_5x5.pdf";

    // Clean up any previous test output
    let _ = fs::remove_file(output_path);

    // Run the CLI tool with 5x5 grid
    let status = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            output_path,
            "--sides",
            "5",
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(status.success(), "CLI tool should execute successfully with 5x5 grid");

    // Verify the PDF was created
    assert!(
        Path::new(output_path).exists(),
        "Output PDF should be created"
    );

    // Clean up
    fs::remove_file(output_path).expect("Failed to clean up test output");
}

#[test]
fn test_cli_verbose_flag() {
    let output_path = "test_output_verbose.pdf";

    // Clean up any previous test output
    let _ = fs::remove_file(output_path);

    // Run the CLI tool with verbose flag
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--cards-path",
            "static/cards",
            "--output",
            output_path,
            "--sides",
            "3",
            "--verbose",
        ])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success(), "CLI tool should execute successfully with verbose flag");

    // Verbose mode should produce some output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    assert!(
        combined_output.contains("Added image") || combined_output.contains("PDF saved to"),
        "Verbose output should contain progress messages"
    );

    // Clean up
    fs::remove_file(output_path).expect("Failed to clean up test output");
}
