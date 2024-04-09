use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "pm_forwarding_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(created_at, default_expr = "Expr::current_timestamp()")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(updated_at, default_expr = "Expr::current_timestamp()")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub bot_id: i64,

    pub telegram_chat_id: i64,
    pub telegram_message_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pm_forwarding_bot::Entity",
        from = "Column::BotId"
        to = "super::pm_forwarding_bot::Column::Id"
    )]
    Bot,
}

impl Related<super::pm_forwarding_bot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bot.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
