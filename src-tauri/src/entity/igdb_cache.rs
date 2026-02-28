use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "igdb_cache")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub rom_id: i64,
    pub igdb_id: Option<i64>,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub aggregated_rating: Option<f64>,
    pub first_release_date: Option<String>,
    pub genres: Option<String>,
    pub themes: Option<String>,
    pub game_modes: Option<String>,
    pub player_perspectives: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub cover_image_id: Option<String>,
    pub screenshot_image_ids: Option<String>,
    pub franchise_name: Option<String>,
    pub raw_response: Option<String>,
    pub fetched_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
