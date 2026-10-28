use crate::config::CONFIG;
use miden_protocol::{
    account::AccountId,
    address::{Address, AddressId},
};

pub fn convert_option_u8_to_bigdecimal(opt: Option<u8>) -> Option<sqlx::types::BigDecimal> {
    opt.map(sqlx::types::BigDecimal::from)
}

pub fn convert_option_u64_to_bigdecimal(opt: Option<u64>) -> Option<sqlx::types::BigDecimal> {
    opt.map(sqlx::types::BigDecimal::from)
}

pub fn account_id_to_bech32(account_id: &AccountId) -> String {
    let address_id = AddressId::from(*account_id);
    let address: Address = Address::new(address_id);
    address.encode(CONFIG.miden_network.clone())
}

/// Upper bound for stored MAST text (account code, note scripts). Pretty-printing expands
/// shared MAST nodes, so the text can be thousands of times larger than the code itself.
pub const MAX_RENDERED_CODE_BYTES: usize = 1024 * 1024;

/// Renders `value` into at most `max_bytes` bytes, stopping the render once the limit is hit
/// and appending a marker comment when the output was truncated.
pub fn render_bounded(value: &impl std::fmt::Display, max_bytes: usize) -> String {
    use std::fmt::Write;

    struct BoundedWriter {
        out: String,
        max_bytes: usize,
    }

    impl Write for BoundedWriter {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            let remaining = self.max_bytes - self.out.len();
            if s.len() <= remaining {
                self.out.push_str(s);
                return Ok(());
            }
            self.out.push_str(&s[..s.floor_char_boundary(remaining)]);
            Err(std::fmt::Error)
        }
    }

    let mut writer = BoundedWriter {
        out: String::new(),
        max_bytes,
    };
    if write!(writer, "{}", value).is_err() {
        writer
            .out
            .push_str(&format!("\n# ... truncated at {} bytes", max_bytes));
    }
    writer.out
}

#[cfg(test)]
mod tests {
    use super::render_bounded;

    #[test]
    fn render_bounded_keeps_short_output() {
        assert_eq!(render_bounded(&"begin end", 64), "begin end");
        assert_eq!(render_bounded(&"begin end", 9), "begin end");
    }

    #[test]
    fn render_bounded_truncates_on_char_boundary() {
        let rendered = render_bounded(&"ab€cd", 4);
        assert_eq!(rendered, "ab\n# ... truncated at 4 bytes");
    }
}
