use cards_core::CardWriter;

#[test]
fn test_cardwriter_dimensions() {
    let _writer = CardWriter::new("test_path".to_string(), 3);

    // Can't access private fields, but we can test the behavior
    // by generating a PDF and checking it doesn't panic
    // This is a basic smoke test
}

#[test]
fn test_cardwriter_different_grid_sizes() {
    let _writer_5x5 = CardWriter::new("test_path".to_string(), 5);

    // Basic instantiation test - ensures constructor works with different sizes
    // Real validation would require accessing private fields or integration tests
}

#[test]
fn test_images_from_path_sorting() {
    // This test requires actual file access, so we'll use the static directory
    let _writer = CardWriter::new("../static".to_string(), 3);

    // The private images_from_path method can't be tested directly
    // This will be covered by integration tests
}

#[test]
fn test_generate_pdf_bytes_returns_data() {
    let writer = CardWriter::new("../static/cards".to_string(), 3);

    // Test that generate_pdf_bytes returns data
    let result = writer.generate_pdf_bytes();
    assert!(result.is_ok(), "Should generate PDF bytes successfully");

    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "PDF bytes should not be empty");

    // Basic PDF signature check (PDFs start with %PDF-)
    assert_eq!(&bytes[0..5], b"%PDF-", "Should be a valid PDF");
}

#[test]
fn test_create_pdf_writes_file() {
    let writer = CardWriter::new("../static/cards".to_string(), 3);
    let output_path = "../test_output_lib.pdf";

    // Clean up any previous test output
    let _ = std::fs::remove_file(output_path);

    let result = writer.create_pdf(output_path);
    assert!(result.is_ok(), "Should create PDF file successfully");

    // Verify file exists
    assert!(
        std::path::Path::new(output_path).exists(),
        "PDF file should exist"
    );

    // Clean up
    std::fs::remove_file(output_path).expect("Failed to clean up test output");
}

#[test]
fn test_error_directory_not_found() {
    let writer = CardWriter::new("nonexistent_directory".to_string(), 3);

    let result = writer.generate_pdf_bytes();
    assert!(result.is_err(), "Should fail with nonexistent directory");

    let err = result.unwrap_err();
    assert!(matches!(err, cards_core::CardsError::DirectoryNotFound(_)));
}

#[test]
fn test_error_display() {
    use cards_core::CardsError;

    let err = CardsError::DirectoryNotFound("test/path".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Directory not found"));
    assert!(display.contains("test/path"));

    let err = CardsError::ImageDecodeError("test error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Image decode error"));
    assert!(display.contains("test error"));
}

#[test]
fn test_error_from_io_error() {
    use cards_core::CardsError;
    use std::io;

    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let cards_err: CardsError = io_err.into();

    assert!(matches!(cards_err, CardsError::IoError(_)));
}
