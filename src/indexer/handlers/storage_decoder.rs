use miden_protocol::{Felt, Word};
use miden_standards::account::{access::Ownable2Step, faucets::TokenMetadata};
use serde_json::{json, Value};

use crate::utils::format::account_id_to_bech32;

// ================================================================================================
// SLOT NAME CONSTANTS
// ================================================================================================

// Auth — SingleSig
const SINGLESIG_PUB_KEY: &str = "miden::standards::auth::singlesig::pub_key";
const SINGLESIG_SCHEME: &str = "miden::standards::auth::singlesig::scheme";

// Auth — SingleSigAcl
const SINGLESIG_ACL_PUB_KEY: &str = "miden::standards::auth::singlesig_acl::pub_key";
const SINGLESIG_ACL_SCHEME: &str = "miden::standards::auth::singlesig_acl::scheme";
const SINGLESIG_ACL_CONFIG: &str = "miden::standards::auth::singlesig_acl::config";

// Auth — Guardian (GuardedMultisig)
const GUARDIAN_PUB_KEY: &str = "miden::standards::auth::guardian::pub_key";
const GUARDIAN_SCHEME: &str = "miden::standards::auth::guardian::scheme";

// Auth — Multisig
const MULTISIG_THRESHOLD_CONFIG: &str = "miden::standards::auth::multisig::threshold_config";
const MULTISIG_APPROVER_PUBLIC_KEYS: &str = "miden::standards::auth::multisig::approver_public_keys";
const MULTISIG_EXECUTED_TRANSACTIONS: &str =
    "miden::standards::auth::multisig::executed_transactions";

// Auth — NetworkAccount
const NETWORK_ACCOUNT_ALLOWED_NOTE_SCRIPTS: &str =
    "miden::standards::auth::network_account::allowed_note_scripts";

// Access — Ownable2Step
const OWNABLE2STEP_OWNER_CONFIG: &str =
    "miden::standards::access::ownable2step::owner_config";

// Metadata — FungibleFaucet
const FUNGIBLE_TOKEN_METADATA: &str =
    "miden::standards::fungible_faucets::metadata";

const FUNGIBLE_NAME_CHUNK_0: &str =
    "miden::standards::metadata::fungible_faucet::name_chunk_0";
const FUNGIBLE_NAME_CHUNK_1: &str =
    "miden::standards::metadata::fungible_faucet::name_chunk_1";

const FUNGIBLE_DESCRIPTION_SLOTS: [&str; 7] = [
    "miden::standards::metadata::fungible_faucet::description_0",
    "miden::standards::metadata::fungible_faucet::description_1",
    "miden::standards::metadata::fungible_faucet::description_2",
    "miden::standards::metadata::fungible_faucet::description_3",
    "miden::standards::metadata::fungible_faucet::description_4",
    "miden::standards::metadata::fungible_faucet::description_5",
    "miden::standards::metadata::fungible_faucet::description_6",
];

const FUNGIBLE_LOGO_URI_SLOTS: [&str; 7] = [
    "miden::standards::metadata::fungible_faucet::logo_uri_0",
    "miden::standards::metadata::fungible_faucet::logo_uri_1",
    "miden::standards::metadata::fungible_faucet::logo_uri_2",
    "miden::standards::metadata::fungible_faucet::logo_uri_3",
    "miden::standards::metadata::fungible_faucet::logo_uri_4",
    "miden::standards::metadata::fungible_faucet::logo_uri_5",
    "miden::standards::metadata::fungible_faucet::logo_uri_6",
];

const FUNGIBLE_EXTERNAL_LINK_SLOTS: [&str; 7] = [
    "miden::standards::metadata::fungible_faucet::external_link_0",
    "miden::standards::metadata::fungible_faucet::external_link_1",
    "miden::standards::metadata::fungible_faucet::external_link_2",
    "miden::standards::metadata::fungible_faucet::external_link_3",
    "miden::standards::metadata::fungible_faucet::external_link_4",
    "miden::standards::metadata::fungible_faucet::external_link_5",
    "miden::standards::metadata::fungible_faucet::external_link_6",
];

// Metadata — StorageSchema
const STORAGE_SCHEMA_COMMITMENT: &str =
    "miden::standards::metadata::storage_schema::commitment";

// ================================================================================================
// PUBLIC API
// ================================================================================================

/// Decode a named storage slot's raw 32-byte Word into a structured JSON payload.
///
/// Returns `None` only if `word_bytes` is not exactly 32 bytes.
/// Otherwise always returns a JSON object with at least `{"type": "...", "display_value": "..."}`.
pub fn decode_slot(slot_name: &str, word_bytes: &[u8]) -> Option<Value> {
    if word_bytes.len() != 32 {
        return None;
    }
    let felts = parse_felts(word_bytes);

    let result = match slot_name {
        // ── Public key / hex ──
        SINGLESIG_PUB_KEY
        | SINGLESIG_ACL_PUB_KEY
        | GUARDIAN_PUB_KEY
        | MULTISIG_APPROVER_PUBLIC_KEYS
        | MULTISIG_EXECUTED_TRANSACTIONS
        | STORAGE_SCHEMA_COMMITMENT
        | NETWORK_ACCOUNT_ALLOWED_NOTE_SCRIPTS => decode_as_hex(word_bytes),

        // ── Ownable2Step owner config ──
        OWNABLE2STEP_OWNER_CONFIG => decode_as_owner_config(felts),

        // ── Auth scheme ──
        SINGLESIG_SCHEME | SINGLESIG_ACL_SCHEME | GUARDIAN_SCHEME => {
            decode_as_auth_scheme(felts[0])
        }

        // ── Numeric (u32/u64 in Word[0]) ──
        MULTISIG_THRESHOLD_CONFIG => decode_as_number(felts[0]),

        // ── Token metadata: [token_supply, max_supply, decimals, symbol_u32] ──
        FUNGIBLE_TOKEN_METADATA => decode_as_token_metadata(felts),

        // ── String chunks (UTF-8 packed into Felts) ──
        FUNGIBLE_NAME_CHUNK_0
        | FUNGIBLE_NAME_CHUNK_1
        | SINGLESIG_ACL_CONFIG => decode_as_string_chunk(felts),

        _ => {
            // Check dynamic arrays (descriptions, logo URIs, external links)
            if FUNGIBLE_DESCRIPTION_SLOTS.contains(&slot_name)
                || FUNGIBLE_LOGO_URI_SLOTS.contains(&slot_name)
                || FUNGIBLE_EXTERNAL_LINK_SLOTS.contains(&slot_name)
            {
                decode_as_string_chunk(felts)
            } else {
                decode_as_raw_word(felts)
            }
        }
    };

    Some(result)
}

// ================================================================================================
// HELPERS
// ================================================================================================

/// Parse 32 bytes into four u64 Felt values (little-endian).
fn parse_felts(bytes: &[u8]) -> [u64; 4] {
    let mut felts = [0u64; 4];
    for (i, chunk) in bytes.chunks(8).enumerate() {
        if chunk.len() == 8 {
            felts[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
    }
    felts
}

/// Hex-encode the entire 32-byte Word.
fn decode_as_hex(bytes: &[u8]) -> Value {
    let hex_body: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let hex = format!("0x{}", hex_body);
    json!({
        "type": "hex",
        "value": hex,
        "display_value": hex
    })
}

/// Map a Felt u64 to an auth scheme name.
fn decode_as_auth_scheme(raw: u64) -> Value {
    // AuthScheme: Falcon512Poseidon2 = 2, EcdsaK256Keccak = 1
    let (id, name) = match raw as u8 {
        1 => (1u8, "EcdsaK256Keccak"),
        2 => (2u8, "Falcon512Poseidon2"),
        other => {
            return json!({
                "type": "auth_scheme",
                "value": other,
                "display_value": format!("Unknown({})", other)
            });
        }
    };
    json!({
        "type": "auth_scheme",
        "value": id,
        "display_value": name
    })
}

/// Decode a u64 as a human-readable number.
fn decode_as_number(raw: u64) -> Value {
    let s = raw.to_string();
    json!({
        "type": "number",
        "value": raw,
        "display_value": s
    })
}

/// Decode a Word as token metadata using [`TokenMetadata::try_from`].
///
/// Storage layout: `[token_supply, max_supply, decimals, token_symbol]`
fn decode_as_token_metadata(felts: [u64; 4]) -> Value {
    let word = Word::new([
        Felt::new(felts[0]),
        Felt::new(felts[1]),
        Felt::new(felts[2]),
        Felt::new(felts[3]),
    ]);

    match TokenMetadata::try_from(word) {
        Ok(meta) => {
            let token_supply = meta.token_supply().as_canonical_u64();
            let max_supply = meta.max_supply().as_canonical_u64();
            let decimals = meta.decimals();
            let symbol = meta.symbol().to_string();
            let display = format!(
                "supply={}/{} decimals={} symbol={}",
                token_supply, max_supply, decimals, symbol
            );
            json!({
                "type": "token_metadata",
                "value": {
                    "token_supply": token_supply,
                    "max_supply": max_supply,
                    "decimals": decimals,
                    "symbol": symbol
                },
                "display_value": display
            })
        }
        Err(_) => decode_as_raw_word(felts),
    }
}

/// Decode `owner_config` using [`Ownable2Step::try_from_word`].
///
/// Storage layout: `[owner_suffix, owner_prefix, nominated_suffix, nominated_prefix]`
fn decode_as_owner_config(felts: [u64; 4]) -> Value {
    let word = Word::new([
        Felt::new(felts[0]),
        Felt::new(felts[1]),
        Felt::new(felts[2]),
        Felt::new(felts[3]),
    ]);

    match Ownable2Step::try_from_word(word) {
        Ok(ownership) => {
            let owner = ownership
                .owner()
                .map(|id| account_id_to_bech32(&id))
                .unwrap_or_default();
            let nominated = ownership
                .nominated_owner()
                .map(|id| account_id_to_bech32(&id))
                .unwrap_or_default();
            let display = if nominated.is_empty() {
                format!("owner={}", owner)
            } else {
                format!("owner={} nominated={}", owner, nominated)
            };
            json!({
                "type": "owner_config",
                "value": {
                    "owner": owner,
                    "nominated_owner": if nominated.is_empty() { None::<String> } else { Some(nominated) }
                },
                "display_value": display
            })
        }
        Err(_) => decode_as_hex(
            &felts
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>(),
        ),
    }
}

/// Decode a Word as a UTF-8 string chunk packed across 4 Felts.
///
/// Each Felt contains up to 7 UTF-8 bytes (the 8th byte is a length hint or padding).
fn decode_as_string_chunk(felts: [u64; 4]) -> Value {
    // Each u64 Felt stores bytes as little-endian; strip null bytes to get the string.
    let mut raw_bytes: Vec<u8> = Vec::with_capacity(32);
    for felt in felts {
        let bytes = felt.to_le_bytes();
        raw_bytes.extend_from_slice(&bytes);
    }
    // Strip trailing null bytes.
    while raw_bytes.last() == Some(&0) {
        raw_bytes.pop();
    }

    let decoded = String::from_utf8_lossy(&raw_bytes).to_string();
    let display = decoded.clone();
    json!({
        "type": "string",
        "value": decoded,
        "display_value": display
    })
}

/// Fallback: encode as a raw word of four Felt hex values.
fn decode_as_raw_word(felts: [u64; 4]) -> Value {
    let hex = felts.map(|f| format!("0x{:016x}", f));
    let display = format!("[{}, {}, {}, {}]", hex[0], hex[1], hex[2], hex[3]);
    json!({
        "type": "raw_word",
        "value": [hex[0], hex[1], hex[2], hex[3]],
        "display_value": display
    })
}
