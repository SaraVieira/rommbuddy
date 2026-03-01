use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::json_vec::JsonVec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    #[sea_orm(string_value = "verified")]
    Verified,
    #[sea_orm(string_value = "unverified")]
    Unverified,
    #[sea_orm(string_value = "bad_dump")]
    BadDump,
}

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
    pub verification_status: Option<VerificationStatus>,
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
