use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Email,
    Role,
    CreatedAt,
    LastAuthenticated,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(User::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(User::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(User::Email).string().not_null())
                .col(ColumnDef::new(User::Role).string().not_null())
                .col(ColumnDef::new(User::CreatedAt).timestamp().not_null())
                .col(ColumnDef::new(User::LastAuthenticated).timestamp().not_null())
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(User::Table).to_owned()).await
    }
}