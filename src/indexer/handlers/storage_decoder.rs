use serde_json::{json, Value};

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
    "miden::standards::metadata::fungible_faucet::token_metadata";

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
        | OWNABLE2STEP_OWNER_CONFIG
        | STORAGE_SCHEMA_COMMITMENT
        | NETWORK_ACCOUNT_ALLOWED_NOTE_SCRIPTS => decode_as_hex(word_bytes),

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

/// Decode a Word as token metadata: `[token_supply, max_supply, decimals, symbol_u32]`.
///
/// Token symbol is stored as a packed ASCII/UTF-8 string in a single u64 field element.
fn decode_as_token_metadata(felts: [u64; 4]) -> Value {
    let token_supply = felts[0];
    let max_supply = felts[1];
    let decimals = felts[2] as u8;
    let symbol_raw = felts[3];

    // Symbol is stored as up to 6 ASCII bytes packed into a u64 (big-endian per Miden convention).
    let symbol_str = decode_token_symbol(symbol_raw);

    let display = format!(
        "supply={}/{} decimals={} symbol={}",
        token_supply, max_supply, decimals, symbol_str
    );

    json!({
        "type": "token_metadata",
        "value": {
            "token_supply": token_supply,
            "max_supply": max_supply,
            "decimals": decimals,
            "symbol": symbol_str
        },
        "display_value": display
    })
}

/// Decode the token symbol packed in a single Felt u64.
///
/// Miden stores the symbol as a right-aligned, null-padded ASCII string in the lower bytes
/// of a u64 (big-endian byte order for the string bytes).
fn decode_token_symbol(raw: u64) -> String {
    // The raw u64 has ASCII bytes in its significant bytes (big-endian).
    let bytes = raw.to_be_bytes();
    let s: String = bytes
        .iter()
        .filter(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    if s.is_empty() {
        format!("0x{:x}", raw)
    } else {
        s
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

/// Fallback: encode as a raw word of four Felt u64 values.
fn decode_as_raw_word(felts: [u64; 4]) -> Value {
    let display = format!("[{}, {}, {}, {}]", felts[0], felts[1], felts[2], felts[3]);
    json!({
        "type": "raw_word",
        "value": [felts[0], felts[1], felts[2], felts[3]],
        "display_value": display
    })
}
