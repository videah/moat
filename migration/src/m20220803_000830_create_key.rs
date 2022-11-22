use sea_orm_migration::prelude::*;
use crate::m20220803_000818_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum Key {
    Table,
    Id,
    UserId,
    Name,
    Color,
    CreatedAt,
    LastUsed,
    Credential,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.create_table(
            Table::create()
                .table(Key::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Key::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(Key::UserId).uuid().not_null())
                .col(ColumnDef::new(Key::Name).string().not_null())
                .col(ColumnDef::new(Key::Color).string().not_null())
                .col(ColumnDef::new(Key::CreatedAt).timestamp().not_null())
                .col(ColumnDef::new(Key::LastUsed).timestamp().not_null())
                .col(ColumnDef::new(Key::Credential).json().not_null())
                .foreign_key(ForeignKey::create()
                    .name("FK-key-user")
                    .from(Key::Table, Key::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::NoAction)
                )
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Key::Table).to_owned()).await
    }
}
