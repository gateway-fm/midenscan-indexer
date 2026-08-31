use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::{
    account::{Account, AccountPatch, AccountUpdateDetails},
    asset::{Asset, AssetComposition, TokenSymbol},
    crypto::utils::Serializable,
    PrettyPrint, Word,
};
use miden_standards::account::faucets::FungibleFaucet;
use std::collections::HashMap;

use super::storage_decoder;

pub async fn account_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_accounts: Vec<db::models::DatabaseAccount> = Vec::new();
    let mut database_account_updates: Vec<db::models::DatabaseAccountUpdate> = Vec::new();
    let mut database_account_vault_assets_changes: HashMap<
        String,
        db::models::DatabaseAccountVaultAsset,
    > = HashMap::new();
    let mut database_account_storage_slot_changes: HashMap<
        String,
        db::models::DatabaseAccountStorageSlot,
    > = HashMap::new();
    let mut database_account_storage_slot_map_changes: HashMap<
        String,
        db::models::DatabaseAccountStorageSlotMap,
    > = HashMap::new();
    let mut database_fungible_faucet_accounts: Vec<db::models::DatabaseFungibleFaucetAccount> =
        Vec::new();

    for (block_updated_account_index_usize, updated_account) in
        block.body().updated_accounts().iter().enumerate()
    {
        // DEVNOTE: don't expect for there to be that many updated accounts in a single block
        let block_updated_account_index = u32::try_from(block_updated_account_index_usize)?;
        let account_bech = utils::format::account_id_to_bech32(&updated_account.account_id());
        let account_id = updated_account.account_id().to_bytes().to_vec();
        let account_id_prefix = updated_account.account_id().prefix().to_bytes().to_vec();

        let account_update_id =
            format!("{}_{}", account_bech, block.header().commitment().to_hex());

        let mut database_account = db::models::DatabaseAccount {
            account_bech: account_bech.clone(),
            account_id: account_id.clone(),
            account_id_prefix: account_id_prefix.clone(),

            account_type: Some(db::models::DatabaseMidenAccountType::Private),
            code: None,
            code_procedure_roots: None,
            code_size: 0,

            deployed_at_block_number: block.header().block_num().as_u32(),
            deployed_at_timestamp: block.header().timestamp(),
            deployed_at_updated_account_index: block_updated_account_index,
            // {deployed_at_block_number}_{deployed_at_updated_account_index}_0
            deployed_at_internal_time: utils::internal_time::get_internal_time(
                block.header().block_num().as_u32(),
                block_updated_account_index,
                0,
            ),
        };
        let mut database_account_update = db::models::DatabaseAccountUpdate {
            account_update_id: account_update_id.clone(),
            account_bech: account_bech.clone(),
            final_state_commitment: updated_account.final_state_commitment().as_bytes().to_vec(),
            nonce_delta: None,

            block_number: block.header().block_num().as_u32(),
            timestamp: block.header().timestamp(),
            updated_account_index: block_updated_account_index,
            // {block_number}_{updated_account_index}_{}
            internal_time: utils::internal_time::get_internal_time(
                block.header().block_num().as_u32(),
                block_updated_account_index,
                0,
            ),
        };

        match updated_account.details() {
            AccountUpdateDetails::Public(account_patch) => {
                if let Some(code) = account_patch.code() {
                    // new account
                    let account = Account::try_from(account_patch).ok();

                    database_account.account_type =
                        Some(db::models::DatabaseMidenAccountType::from(
                            account_patch.id().account_type(),
                        ));

                    database_account.code = Some(format!("{}", PrettyPrint::render(code)));
                    database_account.code_size = code.get_size_hint() as u64;

                    let mut account_code_procedure_roots: Vec<String> = Vec::new();
                    for account_code_procedure in code.procedures() {
                        account_code_procedure_roots
                            .push(account_code_procedure.mast_root().to_hex());
                    }
                    database_account.code_procedure_roots = Some(account_code_procedure_roots);

                    let fungible_faucet_row = account
                        .and_then(|acc| FungibleFaucet::try_from(acc).ok())
                        .map(
                            |fungible_faucet| db::models::DatabaseFungibleFaucetAccount {
                                account_bech: account_bech.clone(),
                                faucet_id_prefix: account_id_prefix.clone(),
                                symbol: Some(fungible_faucet.symbol().to_string()),
                                decimals: Some(fungible_faucet.decimals()),
                                max_supply: Some(fungible_faucet.max_supply().as_u64()),
                            },
                        )
                        .or_else(|| {
                            fallback_fungible_faucet_account(
                                account_patch,
                                &account_bech,
                                &account_id_prefix,
                            )
                        });
                    if let Some(db_acc) = fungible_faucet_row {
                        database_fungible_faucet_accounts.push(db_acc);
                    }
                }

                // The patch carries the final nonce of the account rather than a delta.
                database_account_update.nonce_delta = account_patch
                    .final_nonce()
                    .map(|nonce| nonce.as_canonical_u64());

                // Vault patches carry absolute asset values: updated assets hold the
                // new total amount, removed assets are zeroed out.
                let account_patch_vault = account_patch.vault();
                for asset in account_patch_vault.updated_assets() {
                    let faucet_id_prefix_formatted = asset.faucet_id().prefix().to_bytes().to_vec();
                    let (asset_id_hex, amount) = match asset {
                        Asset::Fungible(fungible) => (
                            fungible.faucet_id().prefix().to_hex(),
                            i64::try_from(u64::from(fungible.amount()))?,
                        ),
                        // Non-fungible assets are unique, one row per asset id.
                        Asset::NonFungible(_) => (asset.id().to_word().to_hex(), 1),
                    };
                    database_account_vault_assets_changes.insert(
                        format!("{}_{}", account_bech, asset_id_hex),
                        db::models::DatabaseAccountVaultAsset {
                            account_vault_asset_id: format!("{}_{}", account_bech, asset_id_hex),
                            account_bech: account_bech.clone(),
                            faucet_id_prefix: faucet_id_prefix_formatted,
                            amount,
                        },
                    );
                }
                for asset_id in account_patch_vault.removed_asset_ids() {
                    let asset_id_hex = if asset_id.composition() == AssetComposition::Fungible {
                        asset_id.faucet_id().prefix().to_hex()
                    } else {
                        asset_id.to_word().to_hex()
                    };
                    database_account_vault_assets_changes.insert(
                        format!("{}_{}", account_bech, asset_id_hex),
                        db::models::DatabaseAccountVaultAsset {
                            account_vault_asset_id: format!("{}_{}", account_bech, asset_id_hex),
                            account_bech: account_bech.clone(),
                            faucet_id_prefix: asset_id.faucet_id().prefix().to_bytes().to_vec(),
                            amount: 0,
                        },
                    );
                }
                for (slot_index, value_patch) in account_patch.storage().values() {
                    let slot_name = slot_index.as_str().to_string();
                    let slot_id_hex = slot_index.id().to_string();
                    let account_storage_slot_id = format!("{}_{}", account_bech, slot_id_hex);
                    // A removed slot has no value; store an empty word.
                    let value_bytes = value_patch.value().unwrap_or_else(Word::empty).to_bytes();
                    let decoded_payload = storage_decoder::decode_slot(&slot_name, &value_bytes);
                    let database_update_account_storage_slot =
                        db::models::DatabaseAccountStorageSlot {
                            account_storage_slot_id: account_storage_slot_id.clone(),
                            account_bech: account_bech.clone(),
                            slot_index: slot_name.clone(),
                            value: value_bytes,
                            // DEVNOTE: this is defaulted to Value, and will be updated
                            // when indexing storage changes if there any.
                            account_storage_slot_type:
                                db::models::DatabaseAccountStorageSlotType::Value,
                            last_updated_at_block_number: block.header().block_num().as_u32(),
                            last_updated_at_account_update_id: account_update_id.clone(),
                            decoded_payload,
                        };
                    database_account_storage_slot_changes.insert(
                        account_storage_slot_id,
                        database_update_account_storage_slot,
                    );
                }
                for (slot_index, storage_map_patch) in account_patch.storage().maps() {
                    let slot_name = slot_index.as_str().to_string();
                    let slot_id_hex = slot_index.id().to_string();
                    let account_storage_slot_id = format!("{}_{}", account_bech, slot_id_hex);
                    if let Some(database_account_storage_slot) =
                        database_account_storage_slot_changes.get_mut(&account_storage_slot_id)
                    {
                        database_account_storage_slot.account_storage_slot_type =
                            db::models::DatabaseAccountStorageSlotType::Map;
                    } else {
                        // Map slots never appear in values() — create the parent slot row here.
                        // The map root/commitment is not available from the delta alone, so we
                        // use zero bytes as a placeholder; the actual data lives in
                        // account_storage_slot_map.
                        database_account_storage_slot_changes.insert(
                            account_storage_slot_id.clone(),
                            db::models::DatabaseAccountStorageSlot {
                                account_storage_slot_id: account_storage_slot_id.clone(),
                                account_bech: account_bech.clone(),
                                slot_index: slot_name.clone(),
                                value: vec![0u8; 32],
                                account_storage_slot_type:
                                    db::models::DatabaseAccountStorageSlotType::Map,
                                last_updated_at_block_number: block.header().block_num().as_u32(),
                                last_updated_at_account_update_id: account_update_id.clone(),
                                decoded_payload: None,
                            },
                        );
                    }

                    // A removed map has no entries to record.
                    let storage_map_entries = storage_map_patch
                        .entries()
                        .map(|entries| entries.as_map().iter())
                        .into_iter()
                        .flatten();
                    for (storage_slot_map_key, storage_slot_map_value) in storage_map_entries {
                        let storage_slot_map_key_word = Word::from(*storage_slot_map_key);
                        let account_storage_slot_map_id = format!(
                            "{}_{}_{}",
                            account_bech,
                            slot_id_hex,
                            storage_slot_map_key_word.to_hex(),
                        );
                        let database_account_storage_slot_map =
                            db::models::DatabaseAccountStorageSlotMap {
                                account_storage_slot_map_id: account_storage_slot_map_id.clone(),
                                account_bech: account_bech.clone(),
                                slot_index: slot_name.clone(),
                                key: storage_slot_map_key_word.as_bytes().to_vec(),
                                value: storage_slot_map_value.to_bytes(),
                                last_updated_at_block_number: block.header().block_num().as_u32(),
                                last_updated_at_account_update_id: account_update_id.clone(),
                                decoded_payload: storage_decoder::decode_map_value(
                                    &slot_name,
                                    &storage_slot_map_value.to_bytes(),
                                ),
                            };
                        database_account_storage_slot_map_changes.insert(
                            account_storage_slot_map_id,
                            database_account_storage_slot_map,
                        );
                    }
                }
            }
            AccountUpdateDetails::Private => {}
        }
        database_accounts.push(database_account);
        database_account_updates.push(database_account_update);
    }
    db::account::insert_or_ignore_accounts(db_tx, database_accounts).await?;
    db::account_update::insert_account_updates(db_tx, database_account_updates).await?;
    db::account_vault_asset::insert_or_set_account_vault_assets(
        db_tx,
        database_account_vault_assets_changes
            .into_values()
            .collect(),
    )
    .await?;
    db::account_storage_slot::insert_or_merge_account_storage_slots(
        db_tx,
        database_account_storage_slot_changes
            .values()
            .cloned()
            .collect(),
    )
    .await?;
    db::account_storage_slot_map::insert_or_merge_account_storage_slot_maps(
        db_tx,
        database_account_storage_slot_map_changes
            .values()
            .cloned()
            .collect(),
    )
    .await?;
    db::fungible_faucet_account::insert_fungible_faucet_accounts(
        db_tx,
        database_fungible_faucet_accounts,
    )
    .await?;

    Ok(())
}

/// Recovers fungible faucet metadata for faucet accounts whose code does not match the
/// current standard `FungibleFaucet` component build, e.g. the agglayer bridge faucets,
/// which were compiled from an older miden-standards revision and therefore fail the
/// MAST-root interface check inside `FungibleFaucet::try_from`. The `token_config` slot
/// name and layout are stable across builds, so its presence identifies a fungible
/// faucet and its word can be decoded directly.
fn fallback_fungible_faucet_account(
    account_patch: &AccountPatch,
    account_bech: &str,
    account_id_prefix: &[u8],
) -> Option<db::models::DatabaseFungibleFaucetAccount> {
    let word = account_patch
        .storage()
        .values()
        .find(|(slot_name, _)| *slot_name == FungibleFaucet::token_config_slot())
        .and_then(|(_, value_patch)| value_patch.value())?;
    let (symbol, decimals, max_supply) = decode_token_config(word)?;
    Some(db::models::DatabaseFungibleFaucetAccount {
        account_bech: account_bech.to_string(),
        faucet_id_prefix: account_id_prefix.to_vec(),
        symbol: Some(symbol),
        decimals: Some(decimals),
        max_supply: Some(max_supply),
    })
}

/// Decodes the `token_config` word `[token_supply, max_supply, decimals, symbol]` into
/// `(symbol, decimals, max_supply)`. Returns `None` if the symbol felt or decimals are
/// not valid, in which case no faucet metadata row is written.
fn decode_token_config(word: Word) -> Option<(String, u8, u64)> {
    let [_token_supply, max_supply, decimals, symbol] = *word;
    let symbol = TokenSymbol::try_from(symbol).ok()?;
    let decimals = u8::try_from(decimals.as_canonical_u64()).ok()?;
    Some((symbol.to_string(), decimals, max_supply.as_canonical_u64()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use miden_protocol::Felt;

    #[test]
    fn decode_token_config_for_agglayer_bridged_eth_faucet() {
        // Actual on-chain token_config word of the agglayer-bridged ETH faucet
        // mtst1aqu8zjdwvcgkeug5a67kpwmnsymvmkg0 (Miden testnet, deployed at block
        // 182748), which FungibleFaucet::try_from rejects due to its older code build:
        // 0xf3aa40000000000000000080ffffff7f08000000000000008545010000000000
        let word = Word::new([
            Felt::new(4_238_067).unwrap(),          // token_supply
            Felt::new(0x7fffffff80000000).unwrap(), // max_supply
            Felt::new(8).unwrap(),                  // decimals
            Felt::new(83_333).unwrap(),             // symbol ("ETH")
        ]);

        let (symbol, decimals, max_supply) = decode_token_config(word).unwrap();
        assert_eq!(symbol, "ETH");
        assert_eq!(decimals, 8);
        assert_eq!(max_supply, 0x7fffffff80000000);
    }

    #[test]
    fn decode_token_config_rejects_invalid_symbol_felt() {
        let word = Word::new([
            Felt::new(0).unwrap(),
            Felt::new(0x7fffffff80000000).unwrap(),
            Felt::new(8).unwrap(),
            // 0 is below TokenSymbol::MIN_ENCODED_VALUE
            Felt::new(0).unwrap(),
        ]);
        assert!(decode_token_config(word).is_none());
    }
}
