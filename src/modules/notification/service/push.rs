use super::{types, Error, Notification, Result};
use crate::{
    modules::{notification::repository::push_token, user},
    types::Context,
};
use oauth_fcm::{send_fcm_message, FcmNotification};
// use std::fs::File;
use std::sync::Arc;

pub async fn send(ctx: Arc<Context>, notification: Notification) -> Result<()> {
    match notification {
        Notification::Registered(n) => send_registered_push_notification(ctx, n).await,
        Notification::OrderStatusUpdated(n) => {
            send_order_status_updated_push_notification(ctx, n).await
        }
        _ => Ok(()),
    }
}

async fn send_registered_push_notification(
    ctx: Arc<Context>,
    payload: types::Registered,
) -> Result<()> {
    let tokens = push_token::find_many_by_user_id(&ctx.db_conn.pool, payload.user.id)
        .await
        .map_err(|_| Error::NotSent)?;

    for token in tokens {
        send_fcm_message::<String>(
            &token.token,
            Some(FcmNotification {
                title: "Registration successful".to_string(),
                body: "Welcome to FoodHut".to_string(),
            }),
            None,
            &ctx.google.fcm_token_manager,
            &ctx.google.fcm_project_id,
        )
        .await
        .map_err(|_| Error::NotSent)?;
    }

    Ok(())
}

async fn send_order_status_updated_push_notification(
    ctx: Arc<Context>,
    payload: types::OrderStatusUpdated,
) -> Result<()> {
    tracing::debug!(
        "About to send push notification to user with id: {}",
        &payload.user.id
    );

    let tokens = push_token::find_many_by_user_id(&ctx.db_conn.pool, payload.user.id.clone())
        .await
        .map_err(|_| Error::NotSent)?;

    tracing::debug!(
        "Got {} tokens for user with id {}",
        tokens.len(),
        &payload.user.id
    );

    for token in tokens {
        send_fcm_message::<String>(
            &token.token,
            Some(FcmNotification {
                title: "Order status updated".to_string(),
                body: format!("Order {} status has been updated", payload.order.id),
            }),
            None,
            &ctx.google.fcm_token_manager,
            &ctx.google.fcm_project_id,
        )
        .await
        .map(|_| {
            tracing::info!(
                "Successfully sent push notification using token with id: {}",
                &token.id
            );
        })
        .map_err(|err| {
            tracing::error!(
                "Failed to send push notification using token with id {}: {:?}",
                &token.id,
                err
            );
            Error::NotSent
        })
        .ok();
    }

    Ok(())
}

// #[cfg(test)]
// mod test {
//     use oauth_fcm::{create_shared_token_manager, send_fcm_message, FcmNotification};
//     use std::fs::File;
//
//     #[tokio::test]
//     async fn should_send_push_notification() {
//         let payload: Option<String> = None;
//         let token_manager = create_shared_token_manager(
//             // File::open("config/messaging-service-account.json").unwrap(),
//             File::open("config/credentials.json").unwrap(),
//         )
//         .unwrap();
//         send_fcm_message(
//             "DEVICE_TOKEN",
//             Some(FcmNotification {
//                 title: "Order status updated".to_string(),
//                 body: "Your order is being prepared".to_string(),
//             }),
//             payload,
//             &token_manager,
//             // "foodhut-434413",
//             "foodhut-75a56",
//         )
//         .await
//         .unwrap();
//     }
// }
