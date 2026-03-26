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
