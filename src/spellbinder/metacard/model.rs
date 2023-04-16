use sqlx::FromRow;

#[derive(Debug, Default, FromRow)]
pub struct MetaCard {
    pub scryfallId: String,
}
