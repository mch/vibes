# Portfolio Rebalancing Tool - Product Requirements Document

## Overview
A simple command line Rust program that reads bank CSV data and generates buy/sell orders to rebalance a mutual fund portfolio according to target allocations.

## Functional Requirements

### Input Files
1. **Bank CSV File**: Contains current portfolio holdings in a specific format:
   - Header rows with account information and totals:
     - As of Date (timestamp)
     - Account name/number
     - Cash balance
     - Total investments value
     - Total portfolio value
   - Investment detail rows with columns:
     - Symbol (fund identifier)
     - Description (fund name)
     - Quantity (shares held)
     - Book Cost (total cost basis)
     - Market Value (current value)
     - Other fields (Price, Average Cost, Unrealized gains, etc.)

2. **Configuration File (TOML)**: Defines target portfolio allocation
   - Mutual fund identifiers
   - Target percentage allocation for each fund
   - Example format:
     ```toml
     [funds]
     ABC123 = 60.0
     ABC456 = 30.0
     ABC789 = 10.0
     ```

### Core Functionality
- Read and parse bank CSV file with mixed header/data format
- Extract cash balance from header section
- Parse investment holdings from data rows (Symbol, Market Value columns)
- Load TOML configuration file with target allocations
- Calculate current portfolio percentages based on market values
- Determine buy/sell orders needed to reach target allocations (considering available cash)
- Generate output CSV with rebalancing instructions

### Output
**CSV File** containing:
- Symbol (mutual fund identifier)
- Action: "BUY" or "SELL"
- Amount (dollar value)

### Processing Notes
- CSV has mixed format: header rows with summary data, followed by investment detail rows
- Investment rows start after empty row and column headers
- Cash balance is extracted from "Cash,<amount>" row
- Fund symbols are in the "Symbol" column, market values in "Market Value" column
- Empty Symbol fields should be ignored (header/summary rows)

### Command Line Interface
```
cargo run -- --input <bank_csv_file> --config <config_file> --output <output_csv_file>
```

## Technical Requirements
- Language: Rust
- CSV parsing: `csv` crate
- TOML parsing: `toml` and `serde` crates
- Command line argument parsing: `clap` crate

## Constraints
- Keep implementation simple - no advanced features
- Focus on core rebalancing logic only
- Minimal error handling (basic validation)

## Out of Scope
- Historical data analysis
- Performance tracking
- Tax optimization
- Multiple account support
- GUI interface
- Real-time market data integration
