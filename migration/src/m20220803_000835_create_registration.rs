use sea_orm_migration::prelude::*;
use crate::m20220803_000830_create_key::Key;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum RegistrationLink {
    Table,
    Id,
    HumanId,
    KeyId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(RegistrationLink::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(RegistrationLink::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(RegistrationLink::HumanId).string().not_null())
                .col(ColumnDef::new(RegistrationLink::KeyId).uuid())
                .foreign_key(ForeignKey::create()
                    .name("FK-registration_link-key")
                    .from(RegistrationLink::Table, RegistrationLink::KeyId)
                    .to(Key::Table, Key::Id)
                    .on_delete(ForeignKeyAction::NoAction)
                )
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RegistrationLink::Table).to_owned()).await
    }
}
