use rand::Rng;

const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub struct StringGenerator {
    prefix: String,
}

impl StringGenerator {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    pub fn generate(&self, length: usize) -> String {
        let mut rng = rand::thread_rng();
        let suffix_len = length.saturating_sub(self.prefix.len());

        let suffix: String = (0..suffix_len)
            .map(|_| {
                let idx = rng.gen_range(0..ALPHANUMERIC.len());
                ALPHANUMERIC[idx] as char
            })
            .collect();

        format!("{}{}", self.prefix, suffix)
    }

    pub fn generate_hex(&self, length: usize) -> String {
        let mut rng = rand::thread_rng();
        let hex_chars = b"0123456789ABCDEF";
        let suffix_len = length.saturating_sub(self.prefix.len());

        let suffix: String = (0..suffix_len)
            .map(|_| {
                let idx = rng.gen_range(0..hex_chars.len());
                hex_chars[idx] as char
            })
            .collect();

        format!("{}{}", self.prefix, suffix)
    }
}

impl Default for StringGenerator {
    fn default() -> Self {
        Self::new("RT".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_length() {
        let gen = StringGenerator::new("RT".to_string());
        let s = gen.generate(12);
        assert_eq!(s.len(), 12);
        assert!(s.starts_with("RT"));
    }

    #[test]
    fn test_generate_alphanumeric() {
        let gen = StringGenerator::new("".to_string());
        let s = gen.generate(100);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }
}
