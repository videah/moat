//! SeaORM Entity. Generated by sea-orm-codegen 0.9.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
    pub last_authenticated: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::key::Entity")]
    Key,
}

impl Related<super::key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Key.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}