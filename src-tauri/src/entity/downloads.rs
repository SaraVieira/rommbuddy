use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "downloads")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub rom_id: i64,
    pub source_id: i64,
    pub status: String,
    pub progress: f64,
    pub file_path: Option<String>,
    pub error_message: Option<String>,
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
    #[sea_orm(
        belongs_to = "super::sources::Entity",
        from = "Column::SourceId",
        to = "super::sources::Column::Id"
    )]
    Source,
}

impl Related<super::roms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rom.def()
    }
}

impl Related<super::sources::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Source.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
