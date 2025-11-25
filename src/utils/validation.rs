use regex::Regex;

pub fn validate_phone_number(phone_number: &str) -> bool {
    // Remove leading 0 if present
    let normalized = phone_number.strip_prefix('0').unwrap_or(phone_number);

    // Match only digits (no country code)
    let phone_regex = Regex::new(r"^\d{9,14}$").unwrap();
    phone_regex.is_match(normalized)
}
