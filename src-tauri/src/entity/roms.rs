use sea_orm::entity::prelude::*;

use super::json_vec::JsonVec;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "roms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub platform_id: i64,
    pub name: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub hash_crc32: Option<String>,
    pub hash_md5: Option<String>,
    pub hash_sha1: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub regions: JsonVec,
    #[sea_orm(column_type = "Text")]
    pub languages: JsonVec,
    pub verification_status: Option<String>,
    pub dat_entry_id: Option<i64>,
    pub dat_game_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::platforms::Entity",
        from = "Column::PlatformId",
        to = "super::platforms::Column::Id"
    )]
    Platform,
}

impl Related<super::platforms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Platform.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
