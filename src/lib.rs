mod multipart;
mod proxies;
mod security;

use multipart::{MultipartFile, encode};
use piston_proxy_commands::{BotCommand, ProxyProtocol};
use proxies::{ProxyApiResponse, ProxyLists};
use serde::{Deserialize, Serialize};
use twilight_model::{
    application::interaction::InteractionType,
    channel::message::MessageFlags,
    http::{
        attachment::Attachment,
        interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    },
};
use worker::{
    CfProperties, Env, Fetch, Request, RequestInit, Response, ResponseBuilder, RouteContext,
    Router, event,
};

const DISCORD_APPLICATION_ID: &str = "DISCORD_APPLICATION_ID";
const DISCORD_PUBLIC_KEY: &str = "DISCORD_PUBLIC_KEY";
const PROXY_API_URL: &str = "PROXY_API_URL";
const SIGNATURE_HEADER: &str = "x-signature-ed25519";
const TIMESTAMP_HEADER: &str = "x-signature-timestamp";
const PROXY_CACHE_TTL_SECONDS: i32 = 2 * 60 * 60;
const PROXY_DISCLAIMER: &str = "-# Disclaimer: Most of these proxies might not work for you and might cause errors. If you are looking for more high-quality proxies, take a look at <http://soulfiremc.com/get-proxies>";

#[derive(Deserialize)]
struct IncomingInteraction {
    #[serde(rename = "type")]
    kind: InteractionType,
    data: Option<IncomingCommandData>,
}

#[derive(Deserialize)]
struct IncomingCommandData {
    name: String,
}

#[event(fetch)]
pub async fn fetch(
    request: Request,
    environment: Env,
    _context: worker::Context,
) -> worker::Result<Response> {
    Router::new()
        .get("/", root)
        .post_async("/interactions", handle_interaction)
        .run(request, environment)
        .await
}

fn root(_request: Request, context: RouteContext<()>) -> worker::Result<Response> {
    let application_id = environment_value(&context.env, DISCORD_APPLICATION_ID)?;
    Response::ok(format!("👋 {application_id}"))
}

async fn handle_interaction(
    mut request: Request,
    context: RouteContext<()>,
) -> worker::Result<Response> {
    let signature = request.headers().get(SIGNATURE_HEADER)?;
    let timestamp = request.headers().get(TIMESTAMP_HEADER)?;
    let body = request.bytes().await?;
    let public_key = environment_value(&context.env, DISCORD_PUBLIC_KEY)?;

    let is_valid = signature
        .zip(timestamp)
        .is_some_and(|(signature, timestamp)| {
            security::verify_discord_request(&public_key, &signature, &timestamp, &body)
        });

    if !is_valid {
        return Response::error("Bad request signature.", 401);
    }

    let interaction = match serde_json::from_slice::<IncomingInteraction>(&body) {
        Ok(interaction) => interaction,
        Err(_) => return error_response("Invalid interaction payload."),
    };

    match interaction.kind {
        InteractionType::Ping => json_response(&InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None,
        }),
        InteractionType::ApplicationCommand => match interaction.data {
            Some(command_data) => handle_application_command(command_data, &context.env).await,
            None => error_response("Invalid application command."),
        },
        _ => error_response("Unknown interaction type."),
    }
}

async fn handle_application_command(
    command_data: IncomingCommandData,
    environment: &Env,
) -> worker::Result<Response> {
    let Some(command) = BotCommand::from_name(&command_data.name) else {
        return error_response("Unknown application command.");
    };

    match command {
        BotCommand::Invite => {
            let application_id = environment_value(environment, DISCORD_APPLICATION_ID)?;
            ephemeral_response(format!(
                "https://discord.com/oauth2/authorize?client_id={application_id}"
            ))
        }
        BotCommand::Proxy(protocol) => match fetch_proxies(environment).await? {
            Some(proxies) => proxy_response(protocol, proxies),
            None => ephemeral_response("Failed to fetch proxies."),
        },
    }
}

async fn fetch_proxies(environment: &Env) -> worker::Result<Option<ProxyLists>> {
    let proxy_api_url = environment_value(environment, PROXY_API_URL)?;
    let mut request_init = RequestInit::new();
    request_init.with_cf_properties(CfProperties {
        cache_everything: Some(true),
        cache_ttl: Some(PROXY_CACHE_TTL_SECONDS),
        ..CfProperties::default()
    });

    let request = Request::new_with_init(&proxy_api_url, &request_init)?;
    let mut response = Fetch::Request(request).send().await?;

    if !(200..300).contains(&response.status_code()) {
        return Ok(None);
    }

    let payload = match response.json::<ProxyApiResponse>().await {
        Ok(payload) => payload,
        Err(_) => return Ok(None),
    };

    Ok(payload.into_proxy_lists())
}

fn proxy_response(protocol: ProxyProtocol, proxies: ProxyLists) -> worker::Result<Response> {
    let file_data = proxies.format(protocol).into_bytes();
    let mut attachment = Attachment::from_bytes(protocol.filename().to_owned(), file_data, 0);
    attachment.description(protocol.attachment_description().to_owned());

    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            attachments: Some(vec![attachment]),
            content: Some(format!(
                "Here are your {} proxies!\n{PROXY_DISCLAIMER}",
                protocol.label()
            )),
            ..InteractionResponseData::default()
        }),
    };
    let attachment = response
        .data
        .as_ref()
        .and_then(|data| data.attachments.as_ref())
        .and_then(|attachments| attachments.first())
        .expect("the proxy response always contains its attachment");
    let encoded = encode(
        &response,
        MultipartFile {
            field_name: "files[0]",
            filename: &attachment.filename,
            content_type: "text/plain",
            data: &attachment.file,
        },
    )?;

    Ok(ResponseBuilder::new()
        .with_header("content-type", &encoded.content_type)?
        .fixed(encoded.body))
}

fn ephemeral_response(content: impl Into<String>) -> worker::Result<Response> {
    json_response(&InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content.into()),
            flags: Some(MessageFlags::EPHEMERAL),
            ..InteractionResponseData::default()
        }),
    })
}

fn json_response<T: Serialize>(body: &T) -> worker::Result<Response> {
    ResponseBuilder::new().from_json(body)
}

fn error_response(message: &'static str) -> worker::Result<Response> {
    #[derive(Serialize)]
    struct ErrorResponse {
        error: &'static str,
    }

    ResponseBuilder::new()
        .with_status(400)
        .from_json(&ErrorResponse { error: message })
}

fn environment_value(environment: &Env, name: &str) -> worker::Result<String> {
    environment
        .secret(name)
        .map(|secret| secret.to_string())
        .or_else(|_| environment.var(name).map(|value| value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{BotCommand, IncomingInteraction, InteractionType, ProxyProtocol};

    #[test]
    fn ping_requires_no_command_data() {
        let interaction: IncomingInteraction =
            serde_json::from_str(r#"{"type":1}"#).expect("ping should deserialize");

        assert_eq!(interaction.kind, InteractionType::Ping);
        assert!(interaction.data.is_none());
    }

    #[test]
    fn command_payload_ignores_unrelated_discord_fields() {
        let interaction: IncomingInteraction = serde_json::from_str(
            r#"{
                "type": 2,
                "data": {"name": "all", "options": []},
                "authorizing_integration_owners": {"0": "123"},
                "future_field": true
            }"#,
        )
        .expect("application command should deserialize");
        let command = interaction
            .data
            .and_then(|data| BotCommand::from_name(&data.name));

        assert_eq!(interaction.kind, InteractionType::ApplicationCommand);
        assert_eq!(command, Some(BotCommand::Proxy(ProxyProtocol::All)));
    }
}
