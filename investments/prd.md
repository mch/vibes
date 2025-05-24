# Portfolio Rebalancing Tool - Product Requirements Document

## Overview
A simple command line Rust program that reads bank CSV data and generates buy/sell orders to rebalance a mutual fund portfolio according to target allocations.

## Functional Requirements

### Input Files
1. **Bank CSV File**: Contains current portfolio holdings
   - Cash balance
   - Mutual fund holdings with:
     - Fund identifier/name
     - Book value
     - Market value
     - Other standard bank CSV fields

2. **Configuration File (TOML)**: Defines target portfolio allocation
   - Mutual fund identifiers
   - Target percentage allocation for each fund
   - Example format:
     ```toml
     [funds]
     VTSAX = 60.0
     VTIAX = 30.0
     VBTLX = 10.0
     ```

### Core Functionality
- Read and parse bank CSV file
- Load TOML configuration file with target allocations
- Calculate current portfolio percentages
- Determine buy/sell orders needed to reach target allocations
- Generate output CSV with rebalancing instructions

### Output
**CSV File** containing:
- Mutual fund name/identifier
- Action: "BUY" or "SELL"
- Amount (dollar value)

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
