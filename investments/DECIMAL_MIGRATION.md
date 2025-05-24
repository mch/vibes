# Decimal Migration Guide

## Why We Migrated from f64 to rust_decimal

### Problems with f64 for Money

1. **Floating-Point Precision Errors**
   ```rust
   // f64 cannot represent 0.1 exactly
   let price = 0.1f64;
   let tax = 0.2f64;
   let total = price + tax;
   assert_ne!(total, 0.3f64); // This assertion passes! 0.1 + 0.2 ≠ 0.3
   ```

2. **Accumulating Errors**
   ```rust
   // Adding pennies with f64
   let mut total = 0.0f64;
   for _ in 0..1000 {
       total += 0.01; // Add 1 cent 1000 times
   }
   assert_ne!(total, 10.0); // Result is 9.999999999999831, not 10.0!
   ```

3. **Inconsistent Comparisons**
   ```rust
   let amount1 = 1000.01f64;
   let amount2 = 1000.02f64;
   let difference = amount2 - amount1;
   // difference might not exactly equal 0.01 due to precision
   ```

### Benefits of rust_decimal

1. **Exact Decimal Representation**
   ```rust
   use rust_decimal::Decimal;
   use std::str::FromStr;
   
   let price = Decimal::from_str("0.1").unwrap();
   let tax = Decimal::from_str("0.2").unwrap();
   let total = price + tax;
   assert_eq!(total, Decimal::from_str("0.3").unwrap()); // Exact!
   ```

2. **No Accumulation Errors**
   ```rust
   let mut total = Decimal::ZERO;
   for _ in 0..1000 {
       total += Decimal::from_str("0.01").unwrap();
   }
   assert_eq!(total, Decimal::from(10)); // Exactly $10.00
   ```

3. **Precise Financial Calculations**
   ```rust
   let portfolio_value = Decimal::from_str("100000.00").unwrap();
   let target_percent = Decimal::from_str("33.33").unwrap();
   let target_value = portfolio_value * (target_percent / Decimal::from(100));
   // Result is exactly $33,330.00, not $33,329.999999...
   ```

## Migration Changes

### 1. Dependencies
```toml
[dependencies]
rust_decimal = { version = "1.35", features = ["serde"] }
```

### 2. Type Changes
```rust
// Before
struct Holding {
    symbol: String,
    market_value: f64,
}

// After
struct Holding {
    symbol: String,
    market_value: Decimal,
}
```

### 3. Parsing Changes
```rust
// Before
let value: f64 = value_str.parse()?;

// After
let value: Decimal = Decimal::from_str(value_str)?;
```

### 4. Arithmetic Changes
```rust
// Before
let target_value = total_value * (target_percent / 100.0);
let difference = target_value - current_value;
if difference.abs() > 1.0 {
    // ...
}

// After
let target_value = total_value * (target_percent / Decimal::from(100));
let difference = target_value - current_value;
if difference.abs() > Decimal::ONE {
    // ...
}
```

### 5. Output Formatting
```rust
// Before
format!("{:.2}", amount)

// After
amount.round_dp(2).to_string()
```

### 6. Configuration Changes
TOML files now use string values for precise parsing:
```toml
# Before
[funds]
VTSAX = 60.0
VTIAX = 30.0

# After
[funds]
VTSAX = "60.0"
VTIAX = "30.0"
```

## Performance Considerations

- **rust_decimal** is slower than f64 for arithmetic operations
- For financial applications, **correctness > speed**
- The performance difference is negligible for portfolio rebalancing use case
- Memory usage is slightly higher (16 bytes vs 8 bytes per value)

## Best Practices

1. **Always use string parsing** for decimal values from external sources
2. **Use Decimal constants** like `Decimal::ZERO`, `Decimal::ONE`
3. **Round appropriately** for display: `.round_dp(2)` for currency
4. **Validate input ranges** to prevent overflow
5. **Use meaningful variable names** to clarify precision requirements

## Testing Decimal Precision

The `test_decimal_precision_benefits` test demonstrates:
- Exact percentage calculations
- Precise arithmetic without accumulation errors
- Correct handling of small differences
- Proper formatting preservation
- Division without inappropriate precision loss

This migration ensures our portfolio rebalancing calculations are financially accurate and reliable.