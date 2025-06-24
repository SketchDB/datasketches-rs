use std::fs;
use std::path::Path;

// Add the local dsrs dependency to your Cargo.toml:
// dsrs = { path = "../datasketches-rs" }
use dsrs::{KllFloatSketch, KllDoubleSketch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== KLL Sketch Demo ===");
    
    // Demo 1: Float sketch with simulated sensor data
    println!("\n1. Creating KLL Float Sketch for temperature sensor data...");
    let mut temp_sketch = KllFloatSketch::with_k(100); // Smaller k for demo
    
    // Simulate temperature readings over time (in Celsius)
    let temperatures = generate_temperature_data(500);
    for temp in &temperatures {
        temp_sketch.update(*temp);
    }
    
    println!("Temperature data analysis:");
    println!("  Total readings: {}", temp_sketch.get_n());
    println!("  Min temperature: {:.2}°C", temp_sketch.get_min_value());
    println!("  Max temperature: {:.2}°C", temp_sketch.get_max_value());
    println!("  K parameter: {}", temp_sketch.get_k());
    println!("  Number retained: {}", temp_sketch.get_num_retained());
    println!("  Is estimation mode: {}", temp_sketch.is_estimation_mode());
    
    // Standard quantiles for comparison
    let quantiles = [0.25, 0.5, 0.75, 0.9, 0.99];
    println!("  Quantiles:");
    for q in &quantiles {
        let value = temp_sketch.get_quantile(*q);
        println!("    {:.0}th percentile: {:.2}°C", q * 100.0, value);
    }
    
    // Rank queries for comparison
    let test_temps = [15.0, 20.0, 25.0, 30.0, 35.0];
    println!("  Rank queries:");
    for temp in &test_temps {
        let rank = temp_sketch.get_rank(*temp);
        println!("    Rank of {:.0}°C: {:.4} ({:.1}%)", temp, rank, rank * 100.0);
    }
    
    // Demo 2: Double sketch with financial data
    println!("\n2. Creating KLL Double Sketch for stock price data...");
    let mut price_sketch = KllDoubleSketch::new();
    
    // Simulate stock prices
    let prices = generate_stock_prices(1000);
    for price in &prices {
        price_sketch.update(*price);
    }
    
    println!("Stock price analysis:");
    println!("  Total price points: {}", price_sketch.get_n());
    println!("  Min price: ${:.2}", price_sketch.get_min_value());
    println!("  Max price: ${:.2}", price_sketch.get_max_value());
    println!("  Median price: ${:.2}", price_sketch.get_quantile(0.5));
    
    // Percentile analysis
    let percentiles = [0.25, 0.5, 0.75, 0.9, 0.95, 0.99];
    println!("  Price percentiles:");
    for p in &percentiles {
        let price = price_sketch.get_quantile(*p);
        println!("    {:.0}%: ${:.2}", p * 100.0, price);
    }
    
    // Demo 3: Sketch merging
    println!("\n3. Demonstrating sketch merging...");
    let mut sketch1 = KllFloatSketch::new();
    let mut sketch2 = KllFloatSketch::new();
    
    // Add different ranges to each sketch
    for i in 1..=250 {
        sketch1.update(i as f32);
    }
    
    for i in 251..=500 {
        sketch2.update(i as f32);
    }
    
    println!("Before merge:");
    println!("  Sketch1: {} items, median: {:.1}", sketch1.get_n(), sketch1.get_quantile(0.5));
    println!("  Sketch2: {} items, median: {:.1}", sketch2.get_n(), sketch2.get_quantile(0.5));
    
    // Merge sketch2 into sketch1
    sketch1.merge(&sketch2);
    
    println!("After merge:");
    println!("  Combined: {} items, median: {:.1}", sketch1.get_n(), sketch1.get_quantile(0.5));
    
    // Demo 4: Serialization to native format
    println!("\n4. Serializing sketches...");
    
    // Native serialization
    let native_bytes = temp_sketch.serialize();
    let native_size = native_bytes.as_ref().len();
    println!("  Native serialization: {} bytes", native_size);
    
    // MessagePack serialization
    let msgpack_bytes = temp_sketch.to_msgpack()?;
    let msgpack_size = msgpack_bytes.len();
    println!("  MessagePack serialization: {} bytes", msgpack_size);
    println!("  Overhead: {:.1}%", (msgpack_size as f64 / native_size as f64 - 1.0) * 100.0);
    
    // Save MessagePack data
    let output_path = "temperature_sketch.msgpack";
    fs::write(output_path, &msgpack_bytes)?;
    println!("  Saved temperature data to: {}", output_path);
    
    // Save stock price data too
    let stock_msgpack = price_sketch.to_msgpack()?;
    fs::write("stock_prices.msgpack", &stock_msgpack)?;
    println!("  Saved stock price data to: stock_prices.msgpack");
    
    // Demo 5: Deserialization and verification
    println!("\n5. Testing deserialization...");
    
    let loaded_data = fs::read(output_path)?;
    let recovered_sketch = KllFloatSketch::from_msgpack(&loaded_data)?;
    
    println!("Verification:");
    println!("  Original items: {}", temp_sketch.get_n());
    println!("  Recovered items: {}", recovered_sketch.get_n());
    println!("  Original median: {:.2}", temp_sketch.get_quantile(0.5));
    println!("  Recovered median: {:.2}", recovered_sketch.get_quantile(0.5));
    
    let median_diff = (temp_sketch.get_quantile(0.5) - recovered_sketch.get_quantile(0.5)).abs();
    println!("  Median difference: {:.6}", median_diff);
    
    if median_diff < 0.001 {
        println!("  ✓ Serialization/deserialization successful!");
    } else {
        println!("  ⚠ Warning: Median difference detected");
    }
    
    // Demo 6: Rank queries
    println!("\n6. Rank analysis on temperature data...");
    let test_temps = [15.0, 20.0, 25.0, 30.0, 35.0];
    for temp in &test_temps {
        let rank = temp_sketch.get_rank(*temp);
        println!("  {:.0}°C is at rank {:.3} ({:.1}% of readings are below this)", 
                 temp, rank, rank * 100.0);
    }
    
    println!("\n=== Demo completed successfully! ===");
    println!("\nFiles created:");
    println!("  - temperature_sketch.msgpack (temperature sensor data)");
    println!("  - stock_prices.msgpack (stock price data)");
    println!("\nThese can be read by Python using the read_kll_sketch.py script.");
    
    Ok(())
}

/// Generate realistic temperature data with seasonal variation
fn generate_temperature_data(count: usize) -> Vec<f32> {
    let mut temps = Vec::with_capacity(count);
    
    for i in 0..count {
        // Base temperature with seasonal variation
        let day_of_year = (i % 365) as f32;
        let seasonal = 10.0 * (2.0 * std::f32::consts::PI * day_of_year / 365.0).cos();
        
        // Daily variation
        let hour = (i % 24) as f32;
        let daily = 5.0 * (2.0 * std::f32::consts::PI * hour / 24.0 - std::f32::consts::PI / 2.0).sin();
        
        // Random noise
        let noise = (i as f32 * 17.0).sin() * 2.0;
        
        let temp = 20.0 + seasonal + daily + noise; // Base 20°C
        temps.push(temp);
    }
    
    temps
}

/// Generate realistic stock price data with trends and volatility
fn generate_stock_prices(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 100.0; // Starting price
    
    for i in 0..count {
        // Add trend (slight upward bias)
        let trend = 0.001;
        
        // Add volatility (random walk)
        let volatility = (i as f64 * 47.0).sin() * 0.02;
        
        // Add market cycles
        let cycle = 0.005 * (2.0 * std::f64::consts::PI * i as f64 / 50.0).sin();
        
        // Update price
        price *= 1.0 + trend + volatility + cycle;
        
        // Ensure price stays positive
        price = price.max(1.0);
        
        prices.push(price);
    }
    
    prices
}