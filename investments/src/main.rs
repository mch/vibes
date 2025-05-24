use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "portfolio-rebalancer")]
#[command(about = "A simple tool to generate buy/sell orders for portfolio rebalancing")]
struct Args {
    /// Input CSV file from bank
    #[arg(short, long)]
    input: PathBuf,

    /// Configuration TOML file with target allocations
    #[arg(short, long)]
    config: PathBuf,

    /// Output CSV file for buy/sell orders
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    funds: HashMap<String, f64>,
}

#[derive(Debug)]
struct Holding {
    symbol: String,
    market_value: f64,
}

#[derive(Debug)]
struct Order {
    fund: String,
    action: String,
    amount: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = load_config(&args.config)?;
    println!("Loaded config with {} funds", config.funds.len());

    // Parse CSV input
    let (cash, holdings) = parse_csv(&args.input)?;
    println!("Cash: ${:.2}", cash);
    println!("Found {} holdings", holdings.len());

    // Calculate orders
    let orders = calculate_orders(&config, cash, &holdings)?;
    println!("Generated {} orders", orders.len());

    // Write output CSV
    write_orders(&args.output, &orders)?;
    println!("Orders written to {:?}", args.output);

    Ok(())
}

fn load_config(path: &PathBuf) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn parse_csv(path: &PathBuf) -> Result<(f64, Vec<Holding>)> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut holdings = Vec::new();
    let mut cash = 0.0;
    let mut in_data_section = false;
    let mut symbol_index = None;
    let mut market_value_index = None;

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        
        // Look for cash entry in header section
        if fields.len() >= 2 && fields[0].trim() == "Cash" {
            if let Ok(value) = fields[1].trim().parse::<f64>() {
                cash = value;
            }
            continue;
        }

        // Check if this is the column header row
        if fields.len() > 10 && fields[0].trim() == "Symbol" {
            // Find column indices
            for (i, field) in fields.iter().enumerate() {
                match field.trim() {
                    "Symbol" => symbol_index = Some(i),
                    "Market Value" => market_value_index = Some(i),
                    _ => {}
                }
            }
            in_data_section = true;
            continue;
        }

        // Parse data rows if we're in the data section
        if in_data_section && fields.len() > 10 {
            if let (Some(sym_idx), Some(mv_idx)) = (symbol_index, market_value_index) {
                if sym_idx < fields.len() && mv_idx < fields.len() {
                    let symbol = fields[sym_idx].trim();
                    let market_value_str = fields[mv_idx].trim();
                    
                    // Skip empty symbols and parse market value
                    if !symbol.is_empty() && !market_value_str.is_empty() {
                        if let Ok(market_value) = market_value_str.parse::<f64>() {
                            holdings.push(Holding {
                                symbol: symbol.to_string(),
                                market_value,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok((cash, holdings))
}

fn calculate_orders(config: &Config, cash: f64, holdings: &[Holding]) -> Result<Vec<Order>> {
    // Calculate total portfolio value
    let total_invested: f64 = holdings.iter().map(|h| h.market_value).sum();
    let total_value = total_invested + cash;

    let mut orders = Vec::new();

    for (fund_name, target_percent) in &config.funds {
        let target_value = total_value * (target_percent / 100.0);
        let current_value = holdings
            .iter()
            .find(|h| &h.symbol == fund_name)
            .map(|h| h.market_value)
            .unwrap_or(0.0);

        let difference = target_value - current_value;

        if difference.abs() > 1.0 { // Only create orders for differences > $1
            let action = if difference > 0.0 { "BUY" } else { "SELL" };
            orders.push(Order {
                fund: fund_name.clone(),
                action: action.to_string(),
                amount: difference.abs(),
            });
        }
    }

    Ok(orders)
}

fn write_orders(path: &PathBuf, orders: &[Order]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;

    // Write header
    writer.write_record(&["Symbol", "Action", "Amount"])?;

    // Write orders
    for order in orders {
        writer.write_record(&[
            &order.fund,
            &order.action,
            &format!("{:.2}", order.amount),
        ])?;
    }

    writer.flush()?;
    Ok(())
}
