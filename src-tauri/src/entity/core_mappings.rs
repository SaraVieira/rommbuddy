use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "core_mappings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub platform_id: i64,
    pub core_name: String,
    pub core_path: String,
    pub is_default: i32,
    pub created_at: String,
    pub emulator_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::platforms::Entity",
        from = "Column::PlatformId",
        to = "super::platforms::Column::Id"
    )]
    Platforms,
}

impl Related<super::platforms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Platforms.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
