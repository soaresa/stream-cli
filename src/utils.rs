use num_format::{Locale, ToFormattedString};

/// Converts a token amount from microns and formats it for printing with a given denomination.
pub fn format_token_amount_with_denom(amount_microns: u64, denomination: &str) -> String {
    // Convert microns to full tokens (divide by 1_000_000)
    let full_token_amount = amount_microns as f64 / 1_000_000.0;
    
    // Split the integer and fractional parts
    let integer_part = (full_token_amount.floor() as u64).to_formatted_string(&Locale::en);
    let formatted_fraction = format!("{:.6}", full_token_amount.fract());
    let fractional_part = formatted_fraction.split('.').nth(1).unwrap_or("000000");
    
    // Format the amount with thousands separator and return as string with denomination
    format!("{} {}.{}", denomination, integer_part, fractional_part)
}

