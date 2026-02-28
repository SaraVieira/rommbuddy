use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "metadata")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub rom_id: i64,
    pub igdb_id: Option<i64>,
    pub screenscraper_id: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub genres: String,
    #[sea_orm(column_type = "Text")]
    pub themes: String,
    pub metadata_fetched_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
