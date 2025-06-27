use crate::infra::config::Twilio;
use crate::utils::helpers::send_otp_via_whatsapp;
use futures::StreamExt;
use serde::Deserialize;

#[derive(Deserialize)]
struct OtpPayload {
    to: String,
    otp: String,
}

pub async fn worker_send_otp(twilio: Twilio, redis_url: String) {
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let mut pubsub = match client.get_async_pubsub().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return;
        }
    };

    if let Err(e) = pubsub.subscribe("send_otp_channel").await {
        tracing::error!("[Worker] Failed to subscribe to channel: {}", e);
        return;
    }

    let mut pubsub_stream = pubsub.on_message();

    tracing::info!("[Worker] subscribed to send_otp_channel");

    loop {
        let msg: String = match pubsub_stream.next().await.unwrap().get_payload() {
            Ok(payload) => payload,
            Err(e) => {
                tracing::error!("[Worker] Failed to get message payload: {}", e);
                continue;
            }
        };

        let payload: Result<OtpPayload, _> = serde_json::from_str(&msg);

        match payload {
            Ok(otp) => {
                tracing::info!("[Worker] Received OTP request for: {}", otp.to);

                let from = twilio.whatsapp_sandbox.clone();
                let account_sid = twilio.account_sid.clone();
                let auth_token = twilio.auth_token.clone();

                if let Err(e) =
                    send_otp_via_whatsapp(&otp.to, &from, &auth_token, &account_sid, &otp.otp).await
                {
                    tracing::error!("[Worker] Failed to send OTP: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("[Worker] Failed to parse OTP payload: {}", e);
            }
        }
    }
}
