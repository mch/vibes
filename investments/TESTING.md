# Testing Guide

This document outlines the testing approaches available for the Portfolio Rebalancing Tool.

## Testing Options

### 1. Unit Tests

Run unit tests for individual functions:

```bash
cargo test
```

The unit tests cover:
- **`test_load_config`**: Verifies TOML configuration parsing
- **`csv_parser::tests::test_parse_csv`**: Tests CSV parsing with mixed header/data format (in csv_parser module)
- **`test_calculate_orders`**: Tests rebalancing calculation logic
- **`test_calculate_orders_with_sells`**: Tests scenarios requiring sell orders
- **`test_calculate_orders_ignores_small_differences`**: Verifies $1 threshold behavior
- **`test_write_orders`**: Tests CSV output generation
- **`test_decimal_precision_benefits`**: Demonstrates precise money calculations using rust_decimal
- **`test_determine_config_path`**: Tests config file path resolution logic

### 2. Integration Tests

Run end-to-end workflow tests:

```bash
cargo test --test integration_test
```

The integration tests cover:
- **`test_end_to_end_workflow`**: Full program execution with default config/output paths
- **`test_custom_config_and_output`**: Custom config and output file paths
- **`test_missing_config_file_error`**: Error handling for missing config files

### 3. Manual Testing with Examples

Test with provided example files:

```bash
# Basic usage (uses examples/config.toml automatically)
cargo run -- --input examples/test-portfolio.csv

# Custom config file
cargo run -- --input examples/test-portfolio.csv --config custom-config.toml

# Custom output file
cargo run -- --input examples/test-portfolio.csv --output my-orders.csv

# All custom
cargo run -- --input examples/test-portfolio.csv --config custom-config.toml --output my-orders.csv
```

### 4. Test Data

#### Example Input CSV (`examples/test-portfolio.csv`)
- Portfolio value: $50,000 ($2,500 cash + $47,500 investments)
- Holdings: VTSAX, VTIAX, VBTLX with realistic values

#### Example Config (`examples/config.toml`)
- Target allocation: 60% VTSAX, 30% VTIAX, 10% VBTLX
- **Note**: Uses string values (e.g., `"60.0"`) for precise decimal parsing

#### Expected Results
The example should generate rebalancing orders to move from current allocation to target allocation.

## Creating Your Own Test Data

### Test CSV Format
Your CSV must include:
1. Header section with `Cash,<amount>` row
2. Empty row separator
3. Column headers starting with `Symbol,Market,Description...`
4. Data rows with fund information

### Test Config Format
```toml
[funds]
SYMBOL1 = "percentage1"
SYMBOL2 = "percentage2"
# ... percentages should sum to 100
# Note: Use string values for precise decimal parsing
```

## Running All Tests

```bash
# Run all unit and integration tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_calculate_orders

# Run with verbose output
cargo test --verbose
```

## Test Coverage

The tests verify:
- **CSV parsing** (in `csv_parser` module):
  - Mixed header/data format handling
  - Cash extraction from header rows
  - Fund data extraction from data rows
- **Core application logic**:
  - TOML configuration loading
  - Config file path resolution and defaults
  - Rebalancing calculation accuracy with precise decimal arithmetic
  - Order generation (BUY/SELL decisions)
  - Minimum order threshold ($1)
- **File I/O operations**:
  - CSV reading and writing
  - Error handling for missing files
- **Command line processing**:
  - Argument parsing and validation
  - Default file path behavior
- **Decimal precision**:
  - No floating-point errors
  - Exact money calculations and formatting

## Debugging Tests

If tests fail:
1. Check that example files exist in `examples/` directory
2. Verify CSV format matches expected structure
3. Ensure config file has valid TOML syntax with string values for percentages
4. Run with `--nocapture` to see debug output
5. Check file permissions for read/write access
6. Verify decimal parsing - ensure percentage values are quoted strings in TOML

## Adding New Tests

To add new test cases:
1. **Unit tests for main logic**: Add to the `tests` module in `src/main.rs`
2. **CSV parsing tests**: Add to the `tests` module in `src/csv_parser.rs`
3. **Integration tests**: Add to `tests/integration_test.rs`
4. **Example data**: Create new files in `examples/` directory
5. **Documentation**: Update this file and relevant docs

## Code Organization

The codebase is organized into modules:
- **`src/main.rs`**: Main application logic, CLI, and core functions
- **`src/csv_parser.rs`**: CSV parsing functionality and related tests
- **`tests/integration_test.rs`**: End-to-end workflow tests
- **`examples/`**: Sample data files for manual testing
