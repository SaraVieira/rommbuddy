use sea_orm::entity::prelude::*;

use super::json_vec::JsonVec;

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
    pub genres: Option<JsonVec>,
    pub themes: Option<JsonVec>,
    pub game_modes: Option<JsonVec>,
    pub player_perspectives: Option<JsonVec>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub cover_image_id: Option<String>,
    pub screenshot_image_ids: Option<JsonVec>,
    pub franchise_name: Option<String>,
    pub raw_response: Option<String>,
    pub fetched_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::roms::Entity",
        from = "Column::RomId",
        to = "super::roms::Column::Id"
    )]
    Rom,
}

impl Related<super::roms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
