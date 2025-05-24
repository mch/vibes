use anyhow::Result;
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct Holding {
    pub symbol: String,
    pub market_value: Decimal,
}

pub fn parse_csv(path: &PathBuf) -> Result<(Decimal, Vec<Holding>)> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut holdings = Vec::new();
    let mut cash = Decimal::ZERO;
    let mut in_data_section = false;
    let mut symbol_index = None;
    let mut market_value_index = None;

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();

        // Look for cash entry in header section
        if fields.len() >= 2 && fields[0].trim() == "Cash" {
            if let Ok(value) = Decimal::from_str(fields[1].trim()) {
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
                        if let Ok(market_value) = Decimal::from_str(market_value_str) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv() {
        let csv_content = r#"As of Date,2025-05-24 14:39:25
Account,BANK NAME - ACCOUNT NUMBER
Cash,5600.43
Investments,59361.28
Total Value,64961.71
,
Symbol,Market,Description,Quantity,Average Cost,Price,Book Cost,Market Value,Unrealized $,Unrealized %,% of Positions,Loan Value,Change Today $,Change Today %,Bid,Bid Lots,Ask,Ask Lots,Volume,Day Low,Day High,52-wk Low,52-wk High
ABC123,,FUND1,123.456,31.789,56.43,3924.54,6966.62,3042.08,77.51,10.72,,,,,,,,,,,,
ABC456,,FUND2,1234.678,34.232,15.343,42265.50,18943.66,-23321.83,-55.18,29.16,,,,,,,,,,,,
ABC789,,FUND3,1031.324,13.456,32.435,13877.50,33450.99,19573.50,141.04,51.49,,,,,,,,,,,,
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let (cash, holdings) = parse_csv(&temp_file.path().to_path_buf()).unwrap();
        
        assert_eq!(cash, Decimal::from_str("5600.43").unwrap());
        assert_eq!(holdings.len(), 3);
        assert_eq!(holdings[0].symbol, "ABC123");
        assert_eq!(holdings[0].market_value, Decimal::from_str("6966.62").unwrap());
        assert_eq!(holdings[1].symbol, "ABC456");
        assert_eq!(holdings[1].market_value, Decimal::from_str("18943.66").unwrap());
        assert_eq!(holdings[2].symbol, "ABC789");
        assert_eq!(holdings[2].market_value, Decimal::from_str("33450.99").unwrap());
    }
}