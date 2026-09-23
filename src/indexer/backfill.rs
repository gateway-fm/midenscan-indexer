//! Backfills `account.code_commitment` for public accounts that were indexed before the
//! column existed.
//!
//! The commitment is taken from the account's code in its deployment block, exactly as
//! the account handler records it for new accounts, so the stored value is identical to
//! what a fresh index would produce. Only rows without a commitment are touched, which
//! makes the run safe to repeat and safe to run next to a live indexer.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use log::{info, warn};
use miden_protocol::{
    account::AccountUpdateDetails, block::SignedBlock, crypto::utils::Serializable,
};

use crate::config::CONFIG;
use crate::db;
use crate::db::account::AccountMissingCodeCommitment;
use crate::rpc;

pub async fn backfill_code_commitments() -> Result<()> {
    let db = db::Database::new(&CONFIG.postgres_url).await;
    let rpc = rpc::Rpc::new(&CONFIG.rpc_url);

    let accounts = db::account::select_accounts_missing_code_commitment(&db.db_conn).await?;
    info!(
        "Backfilling code commitments for {} public accounts",
        accounts.len()
    );

    // Accounts deployed in the same block share one block fetch.
    let mut by_block: BTreeMap<u32, Vec<AccountMissingCodeCommitment>> = BTreeMap::new();
    for account in accounts {
        by_block
            .entry(account.deployed_at_block_number)
            .or_default()
            .push(account);
    }

    let mut updated = 0usize;
    let mut skipped = 0usize;
    for (block_number, accounts) in by_block {
        let block = rpc
            .get_block_by_number_with_timeout(block_number)
            .await
            .with_context(|| format!("failed to fetch block {}", block_number))?;

        for account in accounts {
            match code_commitment_from_block(&block, &account.account_id) {
                Some(code_commitment) => {
                    if db::account::update_account_code_commitment(
                        &db.db_conn,
                        &account.account_bech,
                        &code_commitment,
                    )
                    .await?
                    {
                        updated += 1;
                    }
                }
                None => {
                    warn!(
                        "Account {} has no code in its deployment block {}; left without code commitment",
                        account.account_bech, block_number
                    );
                    skipped += 1;
                }
            }
        }

        if (updated + skipped).is_multiple_of(100) {
            info!(
                "Backfill progress: {} updated, {} skipped, at block {}",
                updated, skipped, block_number
            );
        }
    }

    info!(
        "Backfill finished: {} accounts updated, {} skipped",
        updated, skipped
    );
    Ok(())
}

/// The code commitment of the given account as recorded in a block, if the block carries
/// the account's code (its deployment or a code update of a public account).
fn code_commitment_from_block(block: &SignedBlock, account_id: &[u8]) -> Option<String> {
    block
        .body()
        .updated_accounts()
        .iter()
        .find(|updated_account| updated_account.account_id().to_bytes() == account_id)
        .and_then(|updated_account| match updated_account.details() {
            AccountUpdateDetails::Public(account_patch) => {
                account_patch.code().map(|code| code.commitment().to_hex())
            }
            AccountUpdateDetails::Private => None,
        })
}
