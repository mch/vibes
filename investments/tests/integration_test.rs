use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_end_to_end_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test CSV file
    let csv_content = r#"As of Date,2025-05-24 14:39:25
Account,BANK NAME - ACCOUNT NUMBER
Cash,1000.00
Investments,9000.00
Total Value,10000.00
,
Symbol,Market,Description,Quantity,Average Cost,Price,Book Cost,Market Value,Unrealized $,Unrealized %,% of Positions,Loan Value,Change Today $,Change Today %,Bid,Bid Lots,Ask,Ask Lots,Volume,Day Low,Day High,52-wk Low,52-wk High
ABC123,,FUND1,100.0,50.0,60.0,5000.0,6000.0,1000.0,20.0,60.0,,,,,,,,,,,,
ABC456,,FUND2,50.0,60.0,60.0,3000.0,3000.0,0.0,0.0,30.0,,,,,,,,,,,,
"#;

    let input_file = temp_path.join("test-portfolio.csv");
    fs::write(&input_file, csv_content).unwrap();

    // Create test config file
    let config_content = r#"[funds]
ABC123 = 50.0
ABC456 = 30.0
ABC789 = 20.0
"#;

    let config_file = temp_path.join("config.toml");
    fs::write(&config_file, config_content).unwrap();

    // Run the program
    let output = Command::new("cargo")
        .args(&["run", "--", "--input", input_file.to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute program");

    assert!(
        output.status.success(),
        "Program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output file was created
    let expected_output = temp_path.join("test-portfolio-orders.csv");
    assert!(expected_output.exists(), "Output file was not created");

    // Read and verify output
    let output_content = fs::read_to_string(&expected_output).unwrap();
    let lines: Vec<&str> = output_content.lines().collect();

    assert_eq!(lines[0], "Symbol,Action,Amount");

    // Total portfolio: 10000
    // Target ABC123: 5000 (current: 6000) -> SELL 1000
    // Target ABC456: 3000 (current: 3000) -> no change
    // Target ABC789: 2000 (current: 0) -> BUY 2000

    assert!(
        lines.len() >= 2,
        "Expected at least 2 lines (header + orders)"
    );

    // Find the orders (order may vary)
    let orders: Vec<&str> = lines[1..].to_vec();
    let order_content = orders.join("\n");

    assert!(
        order_content.contains("ABC123,SELL,1000.00") || order_content.contains("ABC123,SELL,1000")
    );
    assert!(
        order_content.contains("ABC789,BUY,2000.00") || order_content.contains("ABC789,BUY,2000")
    );
}

#[test]
fn test_custom_config_and_output() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test CSV file
    let csv_content = r#"As of Date,2025-05-24 14:39:25
Account,BANK NAME - ACCOUNT NUMBER
Cash,0.00
Investments,10000.00
Total Value,10000.00
,
Symbol,Market,Description,Quantity,Average Cost,Price,Book Cost,Market Value,Unrealized $,Unrealized %,% of Positions,Loan Value,Change Today $,Change Today %,Bid,Bid Lots,Ask,Ask Lots,Volume,Day Low,Day High,52-wk Low,52-wk High
XYZ111,,FUND_X,100.0,100.0,100.0,10000.0,10000.0,0.0,0.0,100.0,,,,,,,,,,,,
"#;

    let input_file = temp_path.join("custom-input.csv");
    fs::write(&input_file, csv_content).unwrap();

    // Create custom config file
    let config_content = r#"[funds]
XYZ111 = 80.0
XYZ222 = 20.0
"#;

    let custom_config = temp_path.join("custom-config.toml");
    fs::write(&custom_config, config_content).unwrap();

    let custom_output = temp_path.join("custom-output.csv");

    // Run the program with custom config and output
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--input",
            input_file.to_str().unwrap(),
            "--config",
            custom_config.to_str().unwrap(),
            "--output",
            custom_output.to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute program");

    assert!(
        output.status.success(),
        "Program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that custom output file was created
    assert!(custom_output.exists(), "Custom output file was not created");

    // Verify content
    let output_content = fs::read_to_string(&custom_output).unwrap();
    let lines: Vec<&str> = output_content.lines().collect();

    assert_eq!(lines[0], "Symbol,Action,Amount");

    // Total: 10000
    // Target XYZ111: 8000 (current: 10000) -> SELL 2000
    // Target XYZ222: 2000 (current: 0) -> BUY 2000

    let order_content = lines[1..].join("\n");
    assert!(
        order_content.contains("XYZ111,SELL,2000.00") || order_content.contains("XYZ111,SELL,2000")
    );
    assert!(
        order_content.contains("XYZ222,BUY,2000.00") || order_content.contains("XYZ222,BUY,2000")
    );
}

#[test]
fn test_missing_config_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test CSV file but no config file
    let csv_content = r#"As of Date,2025-05-24 14:39:25
Account,BANK NAME - ACCOUNT NUMBER
Cash,1000.00
Investments,9000.00
Total Value,10000.00
,
Symbol,Market,Description,Quantity,Average Cost,Price,Book Cost,Market Value,Unrealized $,Unrealized %,% of Positions,Loan Value,Change Today $,Change Today %,Bid,Bid Lots,Ask,Ask Lots,Volume,Day Low,Day High,52-wk Low,52-wk High
ABC123,,FUND1,100.0,50.0,60.0,5000.0,6000.0,1000.0,20.0,60.0,,,,,,,,,,,,
"#;

    let input_file = temp_path.join("test-no-config.csv");
    fs::write(&input_file, csv_content).unwrap();

    // Run the program without config file
    let output = Command::new("cargo")
        .args(&["run", "--", "--input", input_file.to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute program");

    assert!(
        !output.status.success(),
        "Program should have failed due to missing config"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Config file not found") || stderr.contains("config.toml"));
}
