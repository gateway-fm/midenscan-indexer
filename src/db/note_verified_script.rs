use super::models::DatabaseNoteVerifiedScript;

use anyhow::Result;
use sqlx::QueryBuilder;

fn canonicalize_script_root(script_root: String) -> String {
    script_root
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}

pub async fn upsert_standard_notes(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    notes: Vec<DatabaseNoteVerifiedScript>,
) -> Result<(), sqlx::Error> {
    if notes.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO note_verified_script (id, name, script_root, rust, masm, timestamp, is_custom) ",
    );
    query_builder.push_values(notes, |mut b, note| {
        b.push_bind(note.id)
            .push_bind(note.name)
            .push_bind(canonicalize_script_root(note.script_root))
            .push_bind(note.rust)
            .push_bind(note.masm)
            .push_bind(note.timestamp)
            .push_bind(note.is_custom);
    });
    query_builder.push(
        " ON CONFLICT (script_root) DO UPDATE SET name = EXCLUDED.name, is_custom = EXCLUDED.is_custom",
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::canonicalize_script_root;

    #[test]
    fn canonicalize_script_root_normalizes_hex() {
        assert_eq!(canonicalize_script_root(" 0xAbCd ".to_string()), "abcd");
    }
}
