use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "launchbox_games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub database_id: String,
    pub name: String,
    pub name_normalized: String,
    pub platform: String,
    pub overview: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub genres: String,
    pub release_date: Option<String>,
    pub community_rating: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
