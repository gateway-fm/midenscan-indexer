use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::{
    account::{delta::AccountUpdateDetails, Account, NonFungibleDeltaAction},
    crypto::utils::Serializable,
    PrettyPrint,
};
use miden_standards::account::faucets::BasicFungibleFaucet;
use std::collections::HashMap;

pub async fn account_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_accounts: Vec<db::models::DatabaseAccount> = Vec::new();
    let mut database_account_updates: Vec<db::models::DatabaseAccountUpdate> = Vec::new();
    let mut database_account_vault_assets_changes: Vec<db::models::DatabaseAccountVaultAsset> =
        Vec::new();
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

            account_type: None,
            storage_mode: db::models::DatabaseMidenAccountStorageMode::Private,
            code: None,
            code_procedure_roots: None,

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
            AccountUpdateDetails::Delta(account_delta) => {
                if let Some(code) = account_delta.code() {
                    // new account
                    let account = Account::try_from(account_delta).ok();

                    database_account.account_type = account
                        .as_ref()
                        .map(|acc| db::models::DatabaseMidenAccountType::from(acc.account_type()));

                    database_account.storage_mode =
                        db::models::DatabaseMidenAccountStorageMode::from(
                            account_delta.id().storage_mode(),
                        );

                    database_account.code = Some(format!("{}", PrettyPrint::render(code)));

                    let mut account_code_procedure_roots: Vec<String> = Vec::new();
                    for account_code_procedure in code.procedures() {
                        account_code_procedure_roots
                            .push(account_code_procedure.mast_root().to_hex());
                    }
                    database_account.code_procedure_roots = Some(account_code_procedure_roots);

                    if let Some(basic_fungible_faucet) =
                        account.and_then(|acc| BasicFungibleFaucet::try_from(acc).ok())
                    {
                        let mut db_acc = db::models::DatabaseFungibleFaucetAccount {
                            account_bech: account_bech.clone(),
                            faucet_id_prefix: account_id_prefix,
                            symbol: None,
                            decimals: Some(basic_fungible_faucet.decimals()),
                            max_supply: Some(basic_fungible_faucet.max_supply().as_canonical_u64()),
                        };
                        let token_symbol_str = basic_fungible_faucet.symbol().to_string();
                        db_acc.symbol = Some(token_symbol_str);
                        database_fungible_faucet_accounts.push(db_acc);
                    }
                }

                database_account_update.nonce_delta = Some(account_delta.nonce_delta().as_canonical_u64());

                let account_delta_vault = account_delta.vault();
                for (faucet_id, amount) in account_delta_vault.fungible().iter() {
                    let faucet_id_prefix_formatted = faucet_id.faucet_id().prefix().to_bytes().to_vec();

                    database_account_vault_assets_changes.push(
                        db::models::DatabaseAccountVaultAsset {
                            account_vault_asset_id: format!(
                                "{}_{}",
                                account_bech,
                                faucet_id.faucet_id().prefix().to_hex(),
                            ),
                            account_bech: account_bech.clone(),
                            faucet_id_prefix: faucet_id_prefix_formatted,
                            amount: *amount,
                        },
                    );
                }
                for (non_fungible_asset, non_fungible_delta) in
                    account_delta_vault.non_fungible().iter()
                {
                    let faucet_id_prefix_formatted =
                        non_fungible_asset.faucet_id().to_bytes().to_vec();
                    let amount: i64 = match non_fungible_delta {
                        NonFungibleDeltaAction::Add => 1,
                        NonFungibleDeltaAction::Remove => -1,
                    };
                    database_account_vault_assets_changes.push(
                        db::models::DatabaseAccountVaultAsset {
                            account_vault_asset_id: format!(
                                "{}_{}",
                                account_bech,
                                non_fungible_asset.faucet_id().to_hex(),
                            ),
                            account_bech: account_bech.clone(),
                            faucet_id_prefix: faucet_id_prefix_formatted,
                            amount,
                        },
                    );
                }
                for (slot_index, leaf) in account_delta.storage().values() {
                    let slot_name = slot_index.as_str().to_string();
                    let slot_id_hex = slot_index.id().to_string();
                    let account_storage_slot_id = format!("{}_{}", account_bech, slot_id_hex);
                    let database_update_account_storage_slot =
                        db::models::DatabaseAccountStorageSlot {
                            account_storage_slot_id: account_storage_slot_id.clone(),
                            account_bech: account_bech.clone(),
                            slot_index: slot_name.clone(),
                            value: leaf.to_bytes(),
                            // DEVNOTE: this is defaulted to Value, and will be updated
                            // when indexing storage changes if there any.
                            account_storage_slot_type:
                                db::models::DatabaseAccountStorageSlotType::Value,
                            last_updated_at_block_number: block.header().block_num().as_u32(),
                            last_updated_at_account_update_id: account_update_id.clone(),
                        };
                    database_account_storage_slot_changes.insert(
                        account_storage_slot_id,
                        database_update_account_storage_slot,
                    );
                }
                for (slot_index, storage_map_delta) in account_delta.storage().maps() {
                    let slot_name = slot_index.as_str().to_string();
                    let slot_id_hex = slot_index.id().to_string();
                    let account_storage_slot_id = format!("{}_{}", account_bech, slot_id_hex);
                    if let Some(database_account_storage_slot) =
                        database_account_storage_slot_changes.get_mut(&account_storage_slot_id)
                    {
                        database_account_storage_slot.account_storage_slot_type =
                            db::models::DatabaseAccountStorageSlotType::Map;
                    }

                    for (storage_slot_map_key, storage_slot_map_value) in
                        storage_map_delta.entries()
                    {
                        let account_storage_slot_map_id = format!(
                            "{}_{}_{}",
                            account_bech,
                            slot_id_hex,
                            storage_slot_map_key.to_hex(),
                        );
                        let database_account_storage_slot_map =
                            db::models::DatabaseAccountStorageSlotMap {
                                account_storage_slot_map_id: account_storage_slot_map_id.clone(),
                                account_bech: account_bech.clone(),
                                slot_index: slot_name.clone(),
                                key: storage_slot_map_key.as_bytes().to_vec(),
                                value: storage_slot_map_value.to_bytes(),
                                last_updated_at_block_number: block.header().block_num().as_u32(),
                                last_updated_at_account_update_id: account_update_id.clone(),
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
    db::account_vault_asset::insert_or_add_account_vault_assets(
        db_tx,
        database_account_vault_assets_changes,
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
