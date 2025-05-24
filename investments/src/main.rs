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

#[derive(Debug, Deserialize)]
struct Holding {
    #[serde(rename = "Fund")]
    fund: String,
    #[serde(rename = "Market Value")]
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
    let mut reader = csv::Reader::from_path(path)?;
    let mut holdings = Vec::new();
    let mut cash = 0.0;

    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;

        // Look for cash entry
        if let Some(fund) = record.get("Fund") {
            if fund.to_lowercase().contains("cash") {
                if let Some(value_str) = record.get("Market Value") {
                    cash = value_str.parse().unwrap_or(0.0);
                }
                continue;
            }
        }

        // Parse holding
        if let (Some(fund), Some(market_value_str)) = (record.get("Fund"), record.get("Market Value")) {
            if let Ok(market_value) = market_value_str.parse::<f64>() {
                holdings.push(Holding {
                    fund: fund.clone(),
                    market_value,
                });
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
            .find(|h| &h.fund == fund_name)
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
    writer.write_record(&["Fund", "Action", "Amount"])?;

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
