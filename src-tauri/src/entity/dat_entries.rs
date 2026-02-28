use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dat_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub dat_file_id: i64,
    pub game_name: String,
    pub rom_name: String,
    pub size: Option<i64>,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub status: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dat_files::Entity",
        from = "Column::DatFileId",
        to = "super::dat_files::Column::Id"
    )]
    DatFiles,
}

impl Related<super::dat_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DatFiles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
