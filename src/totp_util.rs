use std::time::{SystemTime, UNIX_EPOCH};

/// Generate TOTP code from secret and return (code, seconds_remaining)
pub fn generate_totp(secret: &str) -> Result<(String, u64), Box<dyn std::error::Error>> {
    // Remove any spaces and convert to uppercase
    let clean_secret = secret.replace(' ', "").to_uppercase();
    
    // Decode base32 secret
    let key = match base32::decode(base32::Alphabet::RFC4648 { padding: false }, &clean_secret) {
        Some(k) => k,
        None => {
            // Try with padding
            match base32::decode(base32::Alphabet::RFC4648 { padding: true }, &clean_secret) {
                Some(k) => k,
                None => return Err("Failed to decode base32 secret".into()),
            }
        }
    };
    
    // Get current Unix timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    // Calculate time step (30 seconds)
    let time_step = 30u64;
    let steps = now / time_step;
    let remaining = time_step - (now % time_step);
    
    // Generate TOTP
    let totp = totp_lite::totp_custom::<totp_lite::Sha1>(time_step, 6, &key, steps);
    
    Ok((totp, remaining))
}

