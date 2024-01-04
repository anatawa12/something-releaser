use crate::CmdResult;
use clap::Parser;
use reqwest::Url;
use std::num::NonZeroU64;
use tokio::io::AsyncReadExt;

#[derive(Debug, Parser)]
#[command(name = "send-discord")]
#[command(no_binary_name = true)]
#[clap(disable_help_flag = true)]
pub(crate) struct SendDiscord {
    /// The ID of webhook
    #[arg(short = 'i', long = "webhook-id")]
    webhook_id: u64,
    /// The token of webhook
    #[arg(short = 't', long = "webhook-token")]
    webhook_token: String,

    /// The thread the message will be sent
    #[arg(short = 'h', long = "thread")]
    thread: Option<NonZeroU64>,
    /// The name of the user the message will be sent as
    #[arg(short = 'n', long = "name")]
    name: Option<String>,
    /// The avatar of the user the message will be sent as
    #[arg(short = 'a', long = "avatar")]
    avatar: Option<Url>,

    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>, // long help only
}

#[derive(Debug, serde::Serialize)]
struct WebhookContentJson<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<&'a Url>,
}

impl SendDiscord {
    pub async fn run(self) -> CmdResult {
        let mut content = String::new();
        tokio::io::stdin()
            .read_to_string(&mut content)
            .await
            .expect("reading stdin");

        let client = reqwest::Client::builder()
            .user_agent(concat!("something-releaser/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("building reqwest client");

        let url = format!(
            "https://discord.com/api/webhooks/{}/{}",
            self.webhook_id, self.webhook_token
        );
        let url = if let Some(thread) = self.thread {
            Url::parse_with_params(&url, &[("thread_id", thread.get().to_string())])
                .expect("creating webhook url")
        } else {
            Url::parse(&url).expect("creating webhook url")
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&WebhookContentJson {
                content: &content,
                username: self.name.as_deref(),
                avatar_url: self.avatar.as_ref(),
            })
            .send()
            .await
            .expect("sending webhook");

        let status = response.status();

        let response_body = response.text().await.expect("reading response");

        println!("{}", response_body);

        if status.is_client_error() || status.is_server_error() {
            err!("status error: {}", status);
        }

        ok!()
    }
}
