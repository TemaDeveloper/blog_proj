use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
       
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Session::SessionID)
                            .uuid()
                            .not_null()
                            .unique_key()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Session::AccessToken).unique_key().string().not_null())
                    .col(ColumnDef::new(Session::RefreshToken).string().not_null())
                    .col(ColumnDef::new(Session::Data).string())
                    .col(ColumnDef::new(Session::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Session::CSFRToken).string().unique_key().not_null())
                    .col(ColumnDef::new(Session::UserID).integer().not_null()).foreign_key(
                        ForeignKey::create()
                            .name("fk-session-user_id")
                            .from(Session::Table, Session::UserID)
                            .to(User::Table, User::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Session {
    Table,
    SessionID,
    RefreshToken,
    AccessToken,
    ExpiresAt,
    UserID,
    Data,
    CSFRToken,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
