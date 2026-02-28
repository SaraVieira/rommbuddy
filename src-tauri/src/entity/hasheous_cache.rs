use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "hasheous_cache")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub rom_id: i64,
    pub hasheous_id: Option<i64>,
    pub name: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub description: Option<String>,
    pub genres: String,
    pub igdb_game_id: Option<String>,
    pub igdb_platform_id: Option<String>,
    pub thegamesdb_game_id: Option<String>,
    pub retroachievements_game_id: Option<String>,
    pub retroachievements_platform_id: Option<String>,
    pub wikipedia_url: Option<String>,
    pub raw_response: Option<String>,
    pub fetched_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
