use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::{
    asset::Asset::{Fungible, NonFungible},
    crypto::utils::Serializable,
    transaction::OutputNote,
};
use miden_standards::note::NetworkAccountTarget;
use std::collections::HashMap;

pub async fn note_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_notes: Vec<db::models::DatabaseNote> = Vec::new();
    let mut database_note_assets: Vec<db::models::DatabaseNoteAsset> = Vec::new();
    let mut database_note_tags: HashMap<u32, db::models::DatabaseNoteTag> = HashMap::new();

    for (block_note_index, output_note) in block.body().output_notes() {
        let note_metadata = output_note.metadata();
        let recipient: Option<Vec<u8>> = output_note
            .recipient()
            .map(|value| value.digest().as_bytes().to_vec());

        let note_id = output_note.id().as_bytes().to_vec();
        let note_metadata_tag: u32 = note_metadata.tag().into();
        let note_sender = utils::format::account_id_to_bech32(&note_metadata.sender());

        let mut database_note = db::models::DatabaseNote {
            note_id: note_id.clone(),

            recipient,
            sender: note_sender,
            note_type: db::models::DatabaseMidenNoteType::from(note_metadata.note_type()),
            note_tag: note_metadata_tag,
            note_aux: note_metadata.attachment().content().to_word()[0].as_canonical_u64(),
            is_network: note_metadata.attachment().attachment_scheme()
                == NetworkAccountTarget::ATTACHMENT_SCHEME,

            // Only for full notes, added later
            nullifier: None,
            script_code: None,
            inputs: None,

            block_number: block.header().block_num().as_u32(),
            timestamp: block.header().timestamp(),
            batch_index: block_note_index.batch_idx() as u16,
            note_index_in_batch: block_note_index.note_idx_in_batch() as u16,
            leaf_index: block_note_index.leaf_index_value(),
            // {block_number}_{leaf_index}_{}
            internal_time: utils::internal_time::get_internal_time(
                block.header().block_num().as_u32(),
                block_note_index.leaf_index_value() as u32,
                0,
            ),
        };
        if let OutputNote::Public(public_note) = output_note {
            let note = public_note.as_note();
            database_note.nullifier = Some(note.nullifier().as_bytes().to_vec());
            let script_code = format!("{}", note.script());
            database_note.script_code = Some(script_code);
            database_note.inputs = Some(
                note.recipient()
                    .storage()
                    .items()
                    .iter()
                    .map(|v| v.as_canonical_u64())
                    .collect(),
            );
        }
        database_notes.push(database_note);
        if let Some(note_tag) = database_note_tags.get_mut(&note_metadata_tag) {
            note_tag.number_of_notes += 1;
        } else {
            let database_note_tag = db::models::DatabaseNoteTag {
                note_tag: note_metadata_tag,
                number_of_notes: 1,
            };
            database_note_tags.insert(note_metadata_tag, database_note_tag);
        }

        if let Some(note_assets) = output_note.assets() {
            for asset in note_assets.iter() {
                let faucet_id_prefix = asset.faucet_id().prefix().to_bytes().to_vec();
                let amount: u64 = match asset {
                    Fungible(asset) => asset.amount(),
                    NonFungible(_) => 1,
                };
                database_note_assets.push(db::models::DatabaseNoteAsset {
                    note_asset_id: format!(
                        "{}_{}",
                        output_note.id().to_hex(),
                        asset.faucet_id().to_hex(),
                    ),
                    note_id: note_id.clone(),
                    faucet_id_prefix,
                    amount,
                });
            }
        }
    }
    db::note::insert_notes(db_tx, database_notes).await?;
    db::note::insert_or_ignore_note_assets(db_tx, database_note_assets).await?;
    db::note_tag::insert_note_tags(db_tx, database_note_tags.values().cloned().collect()).await?;

    Ok(())
}
