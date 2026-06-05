use miden_protocol::{Felt, Word};
use miden_standards::account::access::Ownable2Step;
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
const MULTISIG_APPROVER_PUBLIC_KEYS: &str =
    "miden::standards::auth::multisig::approver_public_keys";
const MULTISIG_EXECUTED_TRANSACTIONS: &str =
    "miden::standards::auth::multisig::executed_transactions";

// Auth — NetworkAccount
const NETWORK_ACCOUNT_ALLOWED_NOTE_SCRIPTS: &str =
    "miden::standards::auth::network_account::allowed_note_scripts";

// Access — Ownable2Step
const OWNABLE2STEP_OWNER_CONFIG: &str = "miden::standards::access::ownable2step::owner_config";

// MintPolicyManager
const MINT_POLICY_MANAGER_ACTIVE_POLICY_PROC_ROOT: &str =
    "miden::standards::mint_policy_manager::active_policy_proc_root";
const MINT_POLICY_MANAGER_POLICY_AUTHORITY: &str =
    "miden::standards::mint_policy_manager::policy_authority";

// Metadata — FungibleFaucet
const FUNGIBLE_NAME_CHUNK_0: &str = "miden::standards::metadata::fungible_faucet::name_chunk_0";
const FUNGIBLE_NAME_CHUNK_1: &str = "miden::standards::metadata::fungible_faucet::name_chunk_1";

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
const STORAGE_SCHEMA_COMMITMENT: &str = "miden::standards::metadata::storage_schema::commitment";

// ================================================================================================
// PUBLIC API
// ================================================================================================

/// Decode a map entry value given the parent `slot_name`.
///
/// - For `allowed_policy_proc_roots`: Word `[1,0,0,0]` means `true` (allowed),
///   `[0,0,0,0]` means `false`.
/// - For all other maps: falls back to the same slot-level decoder so known
///   slot schemas are reused; unknown values become `raw_word`.
pub fn decode_map_value(slot_name: &str, value_bytes: &[u8]) -> Option<Value> {
    if value_bytes.len() != 32 {
        return None;
    }
    let felts = parse_felts(value_bytes);

    let result = match slot_name {
        // MintPolicyManager — boolean flag
        "miden::standards::mint_policy_manager::allowed_policy_proc_roots" => {
            let enabled = felts[0] != 0;
            let display = if enabled { "true" } else { "false" };
            json!({
                "type": "bool",
                "value": enabled,
                "display_value": display
            })
        }

        // Multisig — approver public keys: value is a raw public-key Word
        "miden::standards::auth::multisig::approver_public_keys" => decode_as_hex(value_bytes),

        // Multisig — approver scheme IDs: [scheme_id, 0, 0, 0]
        "miden::standards::auth::multisig::approver_schemes" => decode_as_auth_scheme(felts[0]),

        // Multisig — executed transactions: both key and value are native Words
        "miden::standards::auth::multisig::executed_transactions" => decode_as_hex(value_bytes),

        // Multisig — procedure thresholds: [threshold, 0, 0, 0]
        "miden::standards::auth::multisig::procedure_thresholds" => decode_as_number(felts[0]),

        // SingleSigAcl — trigger procedure roots: [position_index, 0, 0, 0]
        "miden::standards::auth::singlesig_acl::trigger_procedure_roots" => {
            decode_as_number(felts[0])
        }

        _ => decode_as_raw_word(felts),
    };
    Some(result)
}

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
        | NETWORK_ACCOUNT_ALLOWED_NOTE_SCRIPTS
        | MINT_POLICY_MANAGER_ACTIVE_POLICY_PROC_ROOT => decode_as_hex(word_bytes),

        // ── Ownable2Step owner config ──
        OWNABLE2STEP_OWNER_CONFIG => decode_as_owner_config(felts),

        // ── Auth scheme ──
        SINGLESIG_SCHEME | SINGLESIG_ACL_SCHEME | GUARDIAN_SCHEME => {
            decode_as_auth_scheme(felts[0])
        }

        // ── Mint policy authority (0=AuthControlled, 1=OwnerControlled) ──
        MINT_POLICY_MANAGER_POLICY_AUTHORITY => decode_as_mint_policy_authority(felts[0]),

        // ── Multisig threshold config: [threshold, num_approvers, 0, 0] ──
        MULTISIG_THRESHOLD_CONFIG => decode_as_threshold_config(felts),

        // ── String chunks (UTF-8 packed into Felts) ──
        FUNGIBLE_NAME_CHUNK_0 | FUNGIBLE_NAME_CHUNK_1 | SINGLESIG_ACL_CONFIG => {
            decode_as_string_chunk(felts)
        }

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

/// Decode multisig threshold_config: Word layout is [threshold, num_approvers, 0, 0].
fn decode_as_threshold_config(felts: [u64; 4]) -> Value {
    let threshold = felts[0];
    let num_approvers = felts[1];
    json!({
        "type": "threshold_config",
        "value": {
            "threshold": threshold,
            "num_approvers": num_approvers
        },
        "display_value": format!("{}/{}", threshold, num_approvers)
    })
}

/// Decode a MintPolicyAuthority value (0 = AuthControlled, 1 = OwnerControlled).
fn decode_as_mint_policy_authority(raw: u64) -> Value {
    let (id, name) = match raw as u8 {
        0 => (0u8, "AuthControlled"),
        1 => (1u8, "OwnerControlled"),
        other => {
            return json!({
                "type": "mint_policy_authority",
                "value": other,
                "display_value": format!("Unknown({})", other)
            });
        }
    };
    json!({
        "type": "mint_policy_authority",
        "value": id,
        "display_value": name
    })
}

/// Decode `owner_config` using [`Ownable2Step::try_from_word`].
///
/// Storage layout: `[owner_suffix, owner_prefix, nominated_suffix, nominated_prefix]`
fn decode_as_owner_config(felts: [u64; 4]) -> Value {
    let word = Word::new([
        Felt::new_unchecked(felts[0]),
        Felt::new_unchecked(felts[1]),
        Felt::new_unchecked(felts[2]),
        Felt::new_unchecked(felts[3]),
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
    // Only the first 7 bytes in each felt belong to the payload. The 8th byte is
    // a length hint/padding byte and can introduce embedded NULs if preserved.
    let mut raw_bytes: Vec<u8> = Vec::with_capacity(32);
    for felt in felts {
        let bytes = felt.to_le_bytes();
        raw_bytes.extend_from_slice(&bytes[..7]);
    }
    raw_bytes.retain(|byte| *byte != 0);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn felt_from_payload(bytes: &[u8], marker: u8) -> u64 {
        let mut felt_bytes = [0u8; 8];
        felt_bytes[..bytes.len()].copy_from_slice(bytes);
        felt_bytes[7] = marker;
        u64::from_le_bytes(felt_bytes)
    }

    #[test]
    fn decode_string_chunk_ignores_length_hint_bytes() {
        let decoded = decode_as_string_chunk([
            felt_from_payload(b"ABCDEFG", 1),
            felt_from_payload(b"HIJKLMN", 2),
            felt_from_payload(b"", 0),
            felt_from_payload(b"", 0),
        ]);

        assert_eq!(decoded["value"], "ABCDEFGHIJKLMN");
        assert_eq!(decoded["display_value"], "ABCDEFGHIJKLMN");
    }

    #[test]
    fn decode_slot_for_string_chunks_has_no_embedded_nuls() {
        let mut word_bytes = Vec::new();
        word_bytes.extend_from_slice(&felt_from_payload(b"Token", 1).to_le_bytes());
        word_bytes.extend_from_slice(&felt_from_payload(b" Name", 2).to_le_bytes());
        word_bytes.extend_from_slice(&felt_from_payload(b"", 0).to_le_bytes());
        word_bytes.extend_from_slice(&felt_from_payload(b"", 0).to_le_bytes());

        let decoded = decode_slot(FUNGIBLE_NAME_CHUNK_0, &word_bytes).expect("decoded payload");

        assert_eq!(decoded["value"], "Token Name");
        assert_eq!(decoded["display_value"], "Token Name");
    }
}
