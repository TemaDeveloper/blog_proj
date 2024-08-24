use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Blog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Blog::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Blog::Title).string().not_null())
                    .col(ColumnDef::new(Blog::Content).string().not_null())
                    .col(ColumnDef::new(Blog::Images).array(ColumnType::Text))
                    .col(ColumnDef::new(Blog::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Blog::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog-user_id")
                            .from(Blog::Table, Blog::UserId)
                            .to(User::Table, User::Uuid)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Blog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Blog {
    Table,
    Id,
    UserId,
    Title,
    Content,
    CreatedAt,
    Images,
}


#[derive(Iden)]
enum User {
    Table,
    Uuid,
}
