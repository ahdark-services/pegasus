use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "pm_forwarding_bots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(created_at, default_expr = "Expr::current_timestamp()")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(updated_at, default_expr = "Expr::current_timestamp()")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[sea_orm(column_name = "bot_token", unique)]
    pub bot_token: String,
    pub bot_webhook_secret: String,
    pub target_chat_id: i64,
    #[sea_orm(unsigned)]
    pub telegram_user_refer: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pm_forwarding_message::Entity")]
    Messages,
}

impl ActiveModelBehavior for ActiveModel {}
