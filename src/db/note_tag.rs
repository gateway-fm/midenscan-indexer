use super::models;
use anyhow::Result;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_note_tags(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    note_tags: Vec<models::DatabaseNoteTag>,
) -> Result<(), sqlx::Error> {
    if note_tags.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO note_tag (
            note_tag,
            number_of_notes
        ) ",
    );

    query_builder.push_values(note_tags, |mut b, tag| {
        b.push_bind(BigDecimal::from(tag.note_tag))
            .push_bind(BigDecimal::from(tag.number_of_notes));
    });

    query_builder.push(" ON CONFLICT (note_tag) DO UPDATE SET number_of_notes = note_tag.number_of_notes + EXCLUDED.number_of_notes");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
