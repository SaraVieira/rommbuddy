use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "screenscraper_cache")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub rom_id: i64,
    pub screenscraper_game_id: Option<i64>,
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
