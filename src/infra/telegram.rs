use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::ChatId;

use crate::domain::model::TokenBalance;
use crate::domain::ports::Notifier;

pub struct TelegramNotifier {
    bot: Bot,
    chat_id: ChatId,
}

impl TelegramNotifier {
    pub fn new(token: &str, chat_id: i64) -> Self {
        Self {
            bot: Bot::new(token),
            chat_id: ChatId(chat_id),
        }
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn send_alert(&self, balance: &TokenBalance) -> anyhow::Result<()> {
        let msg = format!(
            "BALANCE ALERT\n\n\
             Account: {}\n\
             Network: {}\n\
             Token: {}\n\
             Balance: {}\n\
             Threshold: {}",
            balance.account_alias,
            balance.network,
            balance.token,
            balance.balance,
            balance.threshold,
        );
        self.bot.send_message(self.chat_id, msg).await?;
        Ok(())
    }
}
