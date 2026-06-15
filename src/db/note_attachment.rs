use super::models;
use anyhow::Result;
use sqlx::QueryBuilder;

pub async fn insert_or_ignore_note_attachments(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    note_attachments: Vec<models::DatabaseNoteAttachment>,
) -> Result<(), sqlx::Error> {
    if note_attachments.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO note_attachment (
            note_attachment_id,
            note_id,
            position,
            scheme,
            content
        ) ",
    );
    query_builder.push_values(note_attachments, |mut b, attachment| {
        b.push_bind(attachment.note_attachment_id)
            .push_bind(attachment.note_id)
            .push_bind(attachment.position)
            .push_bind(attachment.scheme)
            .push_bind(attachment.content);
    });
    query_builder.push(" ON CONFLICT (note_attachment_id) DO NOTHING");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
