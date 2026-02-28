use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dat_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub dat_type: String,
    pub platform_slug: String,
    pub entry_count: i64,
    pub imported_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dat_entries::Entity")]
    DatEntries,
}

impl Related<super::dat_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DatEntries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
