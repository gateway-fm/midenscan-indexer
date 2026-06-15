use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::{
    asset::Asset::{Fungible, NonFungible},
    crypto::utils::Serializable,
    note::NoteAttachments,
    transaction::OutputNote,
};
use std::collections::HashMap;

fn normalize_script_root(script_root: String) -> String {
    script_root
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}

fn attachments_for(output_note: &OutputNote) -> &NoteAttachments {
    match output_note {
        OutputNote::Public(public_note) => public_note.as_note().attachments(),
        OutputNote::Private(private_note) => private_note.attachments(),
    }
}

pub async fn note_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_notes: Vec<db::models::DatabaseNote> = Vec::new();
    let mut database_note_assets: Vec<db::models::DatabaseNoteAsset> = Vec::new();
    let mut database_note_attachments: Vec<db::models::DatabaseNoteAttachment> = Vec::new();
    let mut database_note_tags: HashMap<u32, db::models::DatabaseNoteTag> = HashMap::new();

    for (block_note_index, output_note) in block.body().output_notes() {
        let note_metadata = output_note.metadata();
        let recipient: Option<Vec<u8>> = output_note
            .recipient()
            .map(|value| value.digest().as_bytes().to_vec());

        let note_id = output_note.id().as_bytes().to_vec();
        let note_id_hex = output_note.id().to_hex();
        let note_metadata_tag: u32 = note_metadata.tag().into();
        let note_sender = utils::format::account_id_to_bech32(&note_metadata.sender());

        let mut database_note = db::models::DatabaseNote {
            note_id: note_id.clone(),

            recipient,
            sender: note_sender,
            note_type: db::models::DatabaseMidenNoteType::from(note_metadata.note_type()),
            note_tag: note_metadata_tag,

            // Only for full notes, added later
            nullifier: None,
            script_root: None,
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
            database_note.script_root = Some(normalize_script_root(note.script().root().to_hex()));
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

        for (position, attachment) in attachments_for(output_note).iter().enumerate() {
            database_note_attachments.push(db::models::DatabaseNoteAttachment {
                note_attachment_id: format!("{}_{}", note_id_hex, position),
                note_id: note_id.clone(),
                position: position as i16,
                scheme: attachment.attachment_scheme().as_u16() as i32,
                content: Some(attachment.to_bytes()),
            });
        }

        if let Some(note_assets) = output_note.assets() {
            for asset in note_assets.iter() {
                let faucet_id_prefix = asset.faucet_id().prefix().to_bytes().to_vec();
                let amount: u64 = match asset {
                    Fungible(asset) => asset.amount().as_u64(),
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
    db::note_attachment::insert_or_ignore_note_attachments(db_tx, database_note_attachments)
        .await?;
    db::note_tag::insert_note_tags(db_tx, database_note_tags.values().cloned().collect()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_script_root;
    use miden_standards::note::StandardNote;

    #[test]
    fn script_root_is_formatted_as_lowercase_hex_without_prefix() {
        let script_root = normalize_script_root(StandardNote::P2ID.script_root().to_hex());

        assert!(!script_root.starts_with("0x"));
        assert_eq!(script_root, script_root.to_ascii_lowercase());
    }
}
