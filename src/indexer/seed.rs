use std::collections::BTreeSet;

use anyhow::Result;
use log::info;
use miden_standards::account::components::StandardAccountComponent;
use miden_standards::account::interface::AccountComponentInterface;
use uuid::Uuid;

use crate::db;
use crate::db::models::DatabaseAccountVerifiedComponent;

/// A fixed UUID v5 namespace for midenscan standard account components.
/// Generated once and kept stable so re-runs produce the same IDs.
const MIDENSCAN_COMPONENTS_NAMESPACE: Uuid =
    Uuid::from_bytes([0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8]);

pub async fn seed_standard_components(db: &db::Database) -> Result<()> {
    info!("Seeding standard account components...");

    let components = build_standard_components();
    let count = components.len();

    db.execute_transaction(|db_tx| {
        let components = components.clone();
        Box::pin(async move {
            let existing_digests =
                db::account_verified_component::get_existing_procedure_digests(db_tx).await?;
            let existing_sets: Vec<BTreeSet<&str>> = existing_digests
                .iter()
                .map(|digests| digests.iter().map(|s| s.as_str()).collect())
                .collect();
            let new_components: Vec<_> = components
                .into_iter()
                .filter(|c| {
                    let candidate: BTreeSet<&str> =
                        c.procedure_digests.iter().map(|s| s.as_str()).collect();
                    !existing_sets.contains(&candidate)
                })
                .collect();
    
            let new_component_names = new_components.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
            db::account_verified_component::insert_standard_components(db_tx, new_components)
                .await?;

            if !new_component_names.is_empty() {
                for component in new_component_names {
                    info!("Seeded standard component: {}", component);
                }
            }
            Ok(())
        })
    })
    .await?;

    info!("Seeded {} standard account components total", count);
    Ok(())
}

fn build_standard_components() -> Vec<DatabaseAccountVerifiedComponent> {
    let variants: &[(AccountComponentInterface, StandardAccountComponent)] = &[
        (AccountComponentInterface::BasicWallet, StandardAccountComponent::BasicWallet),
        (AccountComponentInterface::BasicFungibleFaucet, StandardAccountComponent::BasicFungibleFaucet),
        (AccountComponentInterface::NetworkFungibleFaucet, StandardAccountComponent::NetworkFungibleFaucet),
        (AccountComponentInterface::AuthSingleSig, StandardAccountComponent::AuthSingleSig),
        (AccountComponentInterface::AuthSingleSigAcl, StandardAccountComponent::AuthSingleSigAcl),
        (AccountComponentInterface::AuthMultisig, StandardAccountComponent::AuthMultisig),
        (AccountComponentInterface::AuthMultisigPsm, StandardAccountComponent::AuthMultisigPsm),
        (AccountComponentInterface::AuthNoAuth, StandardAccountComponent::AuthNoAuth),
    ];

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs() as i64;

    variants
        .iter()
        .map(|(interface, component)| {
            let name = interface.name();
            let id = Uuid::new_v5(&MIDENSCAN_COMPONENTS_NAMESPACE, name.as_bytes());
            let procedure_digests: Vec<String> = component
                .procedure_digests()
                .map(|word| word.to_hex())
                .collect();

            DatabaseAccountVerifiedComponent {
                id,
                name,
                procedure_digests,
                rust: None,
                masm: None,
                timestamp: now,
                is_custom: false,
            }
        })
        .collect()
}
