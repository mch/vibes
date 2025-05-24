# Portfolio Rebalancer

A simple Rust command line tool that reads bank CSV data and generates buy/sell orders to rebalance a mutual fund portfolio according to target allocations.

## Usage

```bash
cargo run -- --input sample_input.csv --config config.toml --output orders.csv
