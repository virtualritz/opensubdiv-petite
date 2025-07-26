use std::fs;
use std::path::{Path, PathBuf};
use std::env;

/// Check if tests should update expected results
pub fn should_update_expected() -> bool {
    // Check for UPDATE_EXPECTED environment variable
    if env::var("UPDATE_EXPECTED").is_ok() {
        return true;
    }
    
    // Check for --update or -u flag in test arguments
    let args: Vec<String> = env::args().collect();
    args.iter().any(|arg| arg == "--update" || arg == "-u")
}

/// Get the path to the expected results directory
pub fn expected_results_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("expected_results")
}

/// Get the path to the test output directory (in target)
pub fn test_output_dir() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test_output");
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&path).expect("Failed to create test output directory");
    path
}

/// Compare or update a test result file
pub fn assert_file_matches(actual_path: &Path, expected_filename: &str) {
    let expected_path = expected_results_dir().join(expected_filename);
    
    if should_update_expected() {
        // Update mode: copy actual to expected
        fs::copy(actual_path, &expected_path)
            .unwrap_or_else(|_| panic!("Failed to update expected file: {expected_filename}"));
        println!("Updated expected file: {expected_filename}");
    } else {
        // Compare mode
        assert!(
            expected_path.exists(),
            "Expected file does not exist: {}. Run with UPDATE_EXPECTED=1 or --update to create it.",
            expected_path.display()
        );
        
        let actual_content = fs::read_to_string(actual_path)
            .unwrap_or_else(|_| panic!("Failed to read actual file: {}", actual_path.display()));
        let expected_content = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("Failed to read expected file: {}", expected_path.display()));
        
        // For STEP files, normalize the timestamp line before comparison
        let normalize_step = |content: &str| -> String {
            if expected_filename.ends_with(".step") {
                content.lines()
                    .map(|line| {
                        if line.starts_with("FILE_NAME(") {
                            // Replace timestamp with placeholder
                            // Format: FILE_NAME('filename', 'timestamp', ...)
                            let mut in_quotes = false;
                            let mut quote_count = 0;
                            let mut result = String::new();
                            let mut chars = line.chars();
                            
                            while let Some(ch) = chars.next() {
                                if ch == '\'' {
                                    in_quotes = !in_quotes;
                                    if !in_quotes {
                                        quote_count += 1;
                                    }
                                }
                                
                                result.push(ch);
                                
                                // After the second closing quote (end of timestamp), replace content
                                if quote_count == 2 && !in_quotes {
                                    // Find the previous quote and replace the timestamp
                                    let timestamp_end = result.len() - 1;
                                    if let Some(timestamp_start) = result[..timestamp_end].rfind('\'') {
                                        result.replace_range((timestamp_start + 1)..timestamp_end, "TIMESTAMP_PLACEHOLDER");
                                    }
                                    // Add the rest of the line
                                    result.push_str(&chars.collect::<String>());
                                    break;
                                }
                            }
                            
                            result
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                content.to_string()
            }
        };
        
        let normalized_actual = normalize_step(&actual_content);
        let normalized_expected = normalize_step(&expected_content);
        
        assert_eq!(
            normalized_actual,
            normalized_expected,
            "File content mismatch for {expected_filename}. Run with UPDATE_EXPECTED=1 or --update to update expected results."
        );
    }
}

/// Compare or update test result with in-memory content
pub fn assert_content_matches(actual_content: &str, expected_filename: &str) {
    let expected_path = expected_results_dir().join(expected_filename);
    
    if should_update_expected() {
        // Update mode: write content to expected file
        fs::write(&expected_path, actual_content)
            .unwrap_or_else(|_| panic!("Failed to update expected file: {expected_filename}"));
        println!("Updated expected file: {expected_filename}");
    } else {
        // Compare mode
        assert!(
            expected_path.exists(),
            "Expected file does not exist: {}. Run with UPDATE_EXPECTED=1 or --update to create it.",
            expected_path.display()
        );
        
        let expected_content = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("Failed to read expected file: {}", expected_path.display()));
        
        assert_eq!(
            actual_content,
            expected_content,
            "Content mismatch for {expected_filename}. Run with UPDATE_EXPECTED=1 or --update to update expected results."
        );
    }
}

/// Helper to create a test-specific output path
pub fn test_output_path(filename: &str) -> PathBuf {
    test_output_dir().join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_directories_exist() {
        assert!(expected_results_dir().exists());
        assert!(test_output_dir().exists());
    }
}