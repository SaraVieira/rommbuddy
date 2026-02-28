use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "platforms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    pub igdb_id: Option<i64>,
    pub screenscraper_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub file_extensions: String,
    #[sea_orm(column_type = "Text")]
    pub folder_aliases: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
