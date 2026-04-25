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
            db::account_verified_component::insert_standard_components(db_tx, components).await?;
            Ok(())
        })
    })
    .await?;

    info!("Seeded {} standard account components", count);
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
                timestamp: 0,
                is_custom: false,
            }
        })
        .collect()
}
