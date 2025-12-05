pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .trim()
        .replace(' ', "-")
        .replace('.', "")
        .replace(',', "")
        .replace('/', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}
