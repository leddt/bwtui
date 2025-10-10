use std::time::{SystemTime, UNIX_EPOCH};

/// Generate TOTP code from secret and return (code, seconds_remaining)
pub fn generate_totp(secret: &str) -> Result<(String, u64), Box<dyn std::error::Error>> {
    generate_totp_with_time(secret, None)
}

/// Generate TOTP code with optional custom timestamp (for testing)
pub fn generate_totp_with_time(secret: &str, timestamp: Option<u64>) -> Result<(String, u64), Box<dyn std::error::Error>> {
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
    
    // Get current Unix timestamp or use provided one
    let now = match timestamp {
        Some(ts) => ts,
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs(),
    };
    
    // Calculate time step (30 seconds)
    let time_step = 30u64;
    let remaining = time_step - (now % time_step);
    
    // Generate TOTP using the timestamp directly
    // The totp_custom function handles the step calculation internally
    let totp = totp_lite::totp_custom::<totp_lite::Sha1>(time_step, 6, &key, now);
    
    Ok((totp, remaining))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_changes_across_time_steps() {
        // Test with a known TOTP secret (base32 encoded "12345678901234567890")
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        
        // Generate TOTP at timestamp 0 (step 0)
        let (code1, remaining1) = generate_totp_with_time(secret, Some(0)).unwrap();
        assert_eq!(remaining1, 30);
        
        // Generate TOTP at timestamp 29 (still step 0)
        let (code2, remaining2) = generate_totp_with_time(secret, Some(29)).unwrap();
        assert_eq!(remaining2, 1);
        assert_eq!(code1, code2, "Code should be same within same 30-second window");
        
        // Generate TOTP at timestamp 30 (step 1)
        let (code3, remaining3) = generate_totp_with_time(secret, Some(30)).unwrap();
        assert_eq!(remaining3, 30);
        assert_ne!(code1, code3, "Code should change after 30 seconds");
        
        // Generate TOTP at timestamp 60 (step 2)
        let (code4, _) = generate_totp_with_time(secret, Some(60)).unwrap();
        assert_ne!(code3, code4, "Code should change again after another 30 seconds");
        assert_ne!(code1, code4, "Code at step 2 should differ from step 0");
    }
}

