use super::models;
use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_notes(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    notes: Vec<models::DatabaseNote>,
) -> Result<(), sqlx::Error> {
    if notes.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO note (
            note_id,
            recipient,
            sender,
            note_type,
            note_tag,
            note_aux,
            is_network,
            nullifier,
            script_code,
            inputs,
            block_number,
            timestamp,
            batch_index,
            note_index_in_batch,
            leaf_index,
            internal_time
        ) ",
    );

    query_builder.push_values(notes, |mut b, note| {
        let note_inputs: Option<Vec<BigDecimal>> = note
            .inputs
            .as_ref()
            .map(|inputs| inputs.iter().map(|&v| BigDecimal::from(v)).collect());
        b.push_bind(note.note_id)
            .push_bind(note.recipient)
            .push_bind(note.sender)
            .push_bind(note.note_type)
            .push_bind(BigDecimal::from(note.note_tag))
            .push_bind(BigDecimal::from(note.note_aux))
            .push_bind(note.is_network)
            .push_bind(note.nullifier)
            .push_bind(note.script_code)
            .push_bind(note_inputs)
            .push_bind(BigDecimal::from(note.block_number))
            .push_bind(BigDecimal::from(note.timestamp))
            .push_bind(BigDecimal::from(note.batch_index))
            .push_bind(BigDecimal::from(note.note_index_in_batch))
            .push_bind(BigDecimal::from(note.leaf_index))
            .push_bind(BigDecimal::from(BigInt::from(note.internal_time)));
    });

    // TODO a single note can be created many times
    // 0xa24a7a69e8d64309ac4aa40011ada29550039753308e392eeaa6357debbdfe8a
    // - block 447145 AND block 447296
    // this behavior is incorrect and we should rethink how to do this
    query_builder.push(" ON CONFLICT (note_id) DO NOTHING");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}

pub async fn insert_or_ignore_note_assets(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    note_assets: Vec<models::DatabaseNoteAsset>,
) -> Result<(), sqlx::Error> {
    if note_assets.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO note_asset (
            note_asset_id,
            note_id,
            faucet_id_prefix,
            amount
        ) ",
    );
    query_builder.push_values(note_assets, |mut b, note_asset| {
        b.push_bind(note_asset.note_asset_id)
            .push_bind(note_asset.note_id)
            .push_bind(note_asset.faucet_id_prefix)
            .push_bind(BigDecimal::from(note_asset.amount));
    });
    // TODO verify. on conflict, because the note asset hash is used in calculating the actual note_id
    // this should be okay. Eg every note with the same note hash WILL have the same note assets
    query_builder.push(" ON CONFLICT (note_asset_id) DO NOTHING");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
