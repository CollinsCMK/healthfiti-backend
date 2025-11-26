use regex::Regex;

pub fn validate_phone_number(phone_number: &str) -> bool {
    // Remove leading 0 if present
    let normalized = phone_number.strip_prefix('0').unwrap_or(phone_number);

    // Match only digits (no country code)
    let phone_regex = Regex::new(r"^\d{9,14}$").unwrap();
    phone_regex.is_match(normalized)
}

pub fn validate_password(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    if !has_uppercase {
        return false;
    }

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    if !has_lowercase {
        return false;
    }

    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }

    let has_special = password.chars().any(|c| "@$!%*?&".contains(c));
    if !has_special {
        return false;
    }

    let valid_chars_regex = Regex::new(r"^[A-Za-z\d@$!%*?&]+$").unwrap();
    if !valid_chars_regex.is_match(password) {
        return false;
    }

    true
}
