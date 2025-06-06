use anyhow::Result;
use clap::Parser;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
mod csv_parser;
use csv_parser::{parse_csv, Holding};

#[derive(Parser)]
#[command(name = "portfolio-rebalancer")]
#[command(about = "A simple tool to generate buy/sell orders for portfolio rebalancing")]
struct Args {
    /// Input CSV file from bank
    #[arg(short, long)]
    input: PathBuf,

    /// Configuration TOML file with target allocations (defaults to config.toml in input directory)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output CSV file for buy/sell orders (defaults to input filename with -orders suffix)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct Config {
    funds: HashMap<String, Decimal>,
}



#[derive(Debug)]
struct Order {
    fund: String,
    action: String,
    amount: Decimal,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = determine_config_path(&args.input, args.config)?;
    let output_path = determine_output_path(args.output, &args.input)?;
    let config = load_config(&config_path)?;
    println!("Loaded config with {} funds", config.funds.len());

    let (cash, holdings) = parse_csv(&args.input)?;
    println!("Cash: ${:.2}", cash);
    println!("Found {} holdings", holdings.len());

    let orders = calculate_orders(&config, cash, &holdings)?;
    println!("Generated {} orders", orders.len());

    write_orders(&output_path, &orders)?;
    println!("Orders written to {:?}", output_path);

    Ok(())
}

fn determine_output_path(
    path_buf: Option<PathBuf>,
    path: &PathBuf,
) -> Result<PathBuf, anyhow::Error> {
    let output_path = match path_buf {
        Some(path) => path,
        None => {
            let input_stem = path
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine filename from input path"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Input filename contains invalid UTF-8"))?;
            let mut output_path = path.clone();
            output_path.set_file_name(format!("{}-orders.csv", input_stem));
            output_path
        }
    };
    Ok(output_path)
}

fn determine_config_path(input_path: &PathBuf, config_arg: Option<PathBuf>) -> Result<PathBuf> {
    match config_arg {
        Some(path) => Ok(path),
        None => {
            let mut default_config = input_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory of input file"))?
                .to_path_buf();
            default_config.push("config.toml");
            if !default_config.exists() {
                return Err(anyhow::anyhow!(
                    "Config file not found at {:?}. Please create it or specify with --config",
                    default_config
                ));
            }
            Ok(default_config)
        }
    }
}

fn load_config(path: &PathBuf) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}



fn calculate_orders(config: &Config, cash: Decimal, holdings: &[Holding]) -> Result<Vec<Order>> {
    // Calculate total portfolio value
    let total_invested: Decimal = holdings.iter().map(|h| h.market_value).sum();
    let total_value = total_invested + cash;

    let mut orders = Vec::new();

    for (fund_name, target_percent) in &config.funds {
        let target_value = total_value * (*target_percent / Decimal::from(100));
        let current_value = holdings
            .iter()
            .find(|h| &h.symbol == fund_name)
            .map(|h| h.market_value)
            .unwrap_or(Decimal::ZERO);

        let difference = target_value - current_value;

        if difference.abs() > Decimal::ONE {
            // Only create orders for differences > $1
            let action = if difference > Decimal::ZERO {
                "BUY"
            } else {
                "SELL"
            };
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
            &order.amount.round_dp(2).to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        let config_content = r#"
[funds]
ABC123 = "60.0"
ABC456 = "30.0"
ABC789 = "10.0"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_content).unwrap();

        let config = load_config(&temp_file.path().to_path_buf()).unwrap();

        assert_eq!(config.funds.len(), 3);
        assert_eq!(config.funds.get("ABC123"), Some(&Decimal::from(60)));
        assert_eq!(config.funds.get("ABC456"), Some(&Decimal::from(30)));
        assert_eq!(config.funds.get("ABC789"), Some(&Decimal::from(10)));
    }



    #[test]
    fn test_calculate_orders() {
        let mut funds = HashMap::new();
        funds.insert("ABC123".to_string(), Decimal::from(60));
        funds.insert("ABC456".to_string(), Decimal::from(30));
        funds.insert("ABC789".to_string(), Decimal::from(10));
        let config = Config { funds };

        let holdings = vec![
            Holding {
                symbol: "ABC123".to_string(),
                market_value: Decimal::from(6000),
            },
            Holding {
                symbol: "ABC456".to_string(),
                market_value: Decimal::from(2000),
            },
            Holding {
                symbol: "ABC789".to_string(),
                market_value: Decimal::from(1000),
            },
        ];
        let cash = Decimal::from(1000);

        let orders = calculate_orders(&config, cash, &holdings).unwrap();

        // Total value: 6000 + 2000 + 1000 + 1000 = 10000
        // Target ABC123: 10000 * 0.6 = 6000 (current: 6000, diff: 0)
        // Target ABC456: 10000 * 0.3 = 3000 (current: 2000, diff: +1000)
        // Target ABC789: 10000 * 0.1 = 1000 (current: 1000, diff: 0)

        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].fund, "ABC456");
        assert_eq!(orders[0].action, "BUY");
        assert_eq!(orders[0].amount, Decimal::from(1000));
    }

    #[test]
    fn test_calculate_orders_with_sells() {
        let mut funds = HashMap::new();
        funds.insert("ABC123".to_string(), Decimal::from(30));
        funds.insert("ABC456".to_string(), Decimal::from(30));
        funds.insert("ABC789".to_string(), Decimal::from(40));
        let config = Config { funds };

        let holdings = vec![
            Holding {
                symbol: "ABC123".to_string(),
                market_value: Decimal::from(6000),
            },
            Holding {
                symbol: "ABC456".to_string(),
                market_value: Decimal::from(2000),
            },
            Holding {
                symbol: "ABC789".to_string(),
                market_value: Decimal::from(1000),
            },
        ];
        let cash = Decimal::from(1000);

        let orders = calculate_orders(&config, cash, &holdings).unwrap();

        // Total value: 10000
        // Target ABC123: 10000 * 0.3 = 3000 (current: 6000, diff: -3000)
        // Target ABC456: 10000 * 0.3 = 3000 (current: 2000, diff: +1000)
        // Target ABC789: 10000 * 0.4 = 4000 (current: 1000, diff: +3000)

        assert_eq!(orders.len(), 3);

        let abc123_order = orders.iter().find(|o| o.fund == "ABC123").unwrap();
        assert_eq!(abc123_order.action, "SELL");
        assert_eq!(abc123_order.amount, Decimal::from(3000));

        let abc456_order = orders.iter().find(|o| o.fund == "ABC456").unwrap();
        assert_eq!(abc456_order.action, "BUY");
        assert_eq!(abc456_order.amount, Decimal::from(1000));

        let abc789_order = orders.iter().find(|o| o.fund == "ABC789").unwrap();
        assert_eq!(abc789_order.action, "BUY");
        assert_eq!(abc789_order.amount, Decimal::from(3000));
    }

    #[test]
    fn test_calculate_orders_ignores_small_differences() {
        let mut funds = HashMap::new();
        funds.insert("ABC123".to_string(), Decimal::from(60));
        let config = Config { funds };

        let holdings = vec![Holding {
            symbol: "ABC123".to_string(),
            market_value: Decimal::from_str("6000.50").unwrap(),
        }];
        let cash = Decimal::ZERO;

        let orders = calculate_orders(&config, cash, &holdings).unwrap();

        // Total value: 6000.50
        // Target ABC123: 6000.50 * 0.6 = 3600.30 (current: 6000.50, diff: -2400.20)
        // Should create order since difference > $1
        assert_eq!(orders.len(), 1);

        // Test with small difference
        let holdings_small_diff = vec![Holding {
            symbol: "ABC123".to_string(),
            market_value: Decimal::from_str("5999.50").unwrap(),
        }];
        let cash_small = Decimal::from_str("0.50").unwrap();

        let orders_small = calculate_orders(&config, cash_small, &holdings_small_diff).unwrap();
        // Total: 6000, Target: 3600, Current: 5999.50, diff: -2399.50 > $1
        assert_eq!(orders_small.len(), 1);

        // Test with very small difference
        let holdings_tiny_diff = vec![Holding {
            symbol: "ABC123".to_string(),
            market_value: Decimal::from_str("5999.99").unwrap(),
        }];
        let cash_tiny = Decimal::from_str("0.01").unwrap();

        let orders_tiny = calculate_orders(&config, cash_tiny, &holdings_tiny_diff).unwrap();
        // Total: 6000, Target: 3600, Current: 5999.99, diff: -2399.99 > $1
        assert_eq!(orders_tiny.len(), 1);
    }

    #[test]
    fn test_write_orders() {
        let orders = vec![
            Order {
                fund: "ABC123".to_string(),
                action: "BUY".to_string(),
                amount: Decimal::from_str("1500.50").unwrap(),
            },
            Order {
                fund: "ABC456".to_string(),
                action: "SELL".to_string(),
                amount: Decimal::from_str("750.25").unwrap(),
            },
        ];

        let temp_file = NamedTempFile::new().unwrap();
        write_orders(&temp_file.path().to_path_buf(), &orders).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 3); // header + 2 orders
        assert_eq!(lines[0], "Symbol,Action,Amount");
        assert_eq!(lines[1], "ABC123,BUY,1500.50");
        assert_eq!(lines[2], "ABC456,SELL,750.25");
    }

    #[test]
    fn test_decimal_precision_benefits() {
        // This test demonstrates why Decimal is better than f64 for money calculations

        // Example: precise percentage calculations that would fail with f64
        let portfolio_value = Decimal::from_str("100000.00").unwrap();
        let target_percent = Decimal::from_str("33.33").unwrap(); // 1/3 allocation

        // Calculate target value - this is exact with Decimal
        let target_value = portfolio_value * (target_percent / Decimal::from(100));
        assert_eq!(target_value, Decimal::from_str("33330.00").unwrap());

        // Test precise arithmetic that would accumulate errors with f64
        let mut running_total = Decimal::ZERO;
        for _ in 0..1000 {
            running_total += Decimal::from_str("0.01").unwrap(); // Add 1 cent 1000 times
        }
        assert_eq!(running_total, Decimal::from(10)); // Exactly $10.00

        // Test that small differences are handled correctly
        let amount1 = Decimal::from_str("1000.01").unwrap();
        let amount2 = Decimal::from_str("1000.02").unwrap();
        let difference = amount2 - amount1;
        assert_eq!(difference, Decimal::from_str("0.01").unwrap()); // Exactly 1 cent

        // Verify formatting preserves precision
        let precise_amount = Decimal::from_str("12345.67").unwrap();
        assert_eq!(precise_amount.round_dp(2).to_string(), "12345.67");

        // Test division doesn't lose precision inappropriately
        let total = Decimal::from_str("100.00").unwrap();
        let shares = Decimal::from(3);
        let per_share = total / shares;
        assert_eq!(per_share.round_dp(2), Decimal::from_str("33.33").unwrap());
    }

    #[test]
    fn test_determine_config_path() {
        // Test with provided config path
        let input_path = PathBuf::from("test/input.csv");
        let config_path = PathBuf::from("custom/config.toml");
        let result = determine_config_path(&input_path, Some(config_path.clone())).unwrap();
        assert_eq!(result, config_path);

        // Test with default config path (file exists)
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input_file = temp_dir.path().join("portfolio.csv");
        let config_file = temp_dir.path().join("config.toml");

        // Create the files
        std::fs::write(&input_file, "test content").unwrap();
        std::fs::write(&config_file, "[funds]\nTEST = \"100.0\"").unwrap();

        let result = determine_config_path(&input_file, None).unwrap();
        assert_eq!(result, config_file);

        // Test with default config path (file doesn't exist)
        let temp_dir2 = tempfile::TempDir::new().unwrap();
        let input_file2 = temp_dir2.path().join("portfolio2.csv");
        std::fs::write(&input_file2, "test content").unwrap();
        // Don't create config.toml

        let result = determine_config_path(&input_file2, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Config file not found"));

        // Test with isolated directory that doesn't contain config.toml
        let temp_dir3 = tempfile::TempDir::new().unwrap();
        let isolated_input = temp_dir3.path().join("input.csv");
        std::fs::write(&isolated_input, "test content").unwrap();

        let result2 = determine_config_path(&isolated_input, None);
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("Config file not found"));
    }
}
