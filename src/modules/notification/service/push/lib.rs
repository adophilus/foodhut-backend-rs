use super::super::{Notification, Result};
use crate::types;
use oauth_fcm::{create_shared_token_manager, send_fcm_message, FcmNotification};
use std::fs::File;
use std::sync::Arc;

pub async fn send(ctx: Arc<types::Context>, notification: Notification) -> Result<()> {
    Ok(())
    // match notification {
    //     Notification::BankAccountCreationFailed()
    // }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn should_send_push_notification() {
        let payload: Option<String> = None;
        let token_manager = create_shared_token_manager(
            File::open("config/messaging-service-account.json").unwrap(),
        )
        .unwrap();
        let result = send_fcm_message(
            "",
            Some(FcmNotification {
                title: "Order status updated".to_string(),
                body: "Your order is being prepared".to_string(),
            }),
            payload,
            &token_manager,
            "foodhut-434413",
        )
        .await;
        println!("{:?}", result);
    }
}
