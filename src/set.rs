use crate::scryfall::fetch_sets;
use sqlx::PgPool;
use std::collections::HashMap;

use scryfall::Set;
use std::error::Error;

struct FormattedSet {
    id: String,
    scryfall_id: String,
    name: String,
    code: String
}

impl FormattedSet {
    fn new(set: &Set) -> Self {
        let id = cuid::cuid2();
        let scryfall_id = set.id.as_hyphenated().to_string();
        let name = set.name.to_owned();
        let code = set.code.to_string();

        Self {
            id,
            scryfall_id,
            name,
            code
        }
    }
}

/// Returns a hashmap mapping the set code to the set's id
pub async fn upsert_sets(pool: &PgPool) -> HashMap<String, String> {
    let opt_sets = fetch_sets().await;
    if opt_sets.is_none() {
        return HashMap::new();
    }
    
    let sets = opt_sets.unwrap();
    let real_sets_iter = sets.into_iter().filter(|set| !set.digital);
    let real_sets = Vec::from_iter(real_sets_iter);

    

    batch_upsert_sets(real_sets, pool)
        .await
        .unwrap_or_default()
}

async fn batch_upsert_sets(sets: Vec<Set>, pool: &PgPool) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut ids: Vec<String> = Vec::new();
    let mut scryfall_ids: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut codes: Vec<String> = Vec::new();

    for set in sets {
        let formatted_set = FormattedSet::new(&set);

        ids.push(formatted_set.id);
        scryfall_ids.push(formatted_set.scryfall_id);
        names.push(formatted_set.name);
        codes.push(formatted_set.code);
    }

    let db_sets = sqlx::query!(r#"INSERT INTO "Expansion"
        ("id", "scryfallId", "name", "code")
        (SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[]))
        ON CONFLICT DO NOTHING
        RETURNING "id", "code" ;"#,
        &ids[..],
        &scryfall_ids[..],
        &names[..],
        &codes[..]
    )
    .fetch_all(pool)
    .await?;

    let mut set_code_to_id_map: HashMap<String, String> = HashMap::new();
    for set in db_sets {
        set_code_to_id_map.insert(set.code, set.id);
    }

    Ok(set_code_to_id_map)
}
