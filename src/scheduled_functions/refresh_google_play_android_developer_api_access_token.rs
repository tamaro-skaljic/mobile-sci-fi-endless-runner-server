use spacetimedb::{ReducerContext, ScheduleAt, Timestamp, table};
use spacetimedsl::prelude::*;

use crate::tables::config::{GetConfigRowOptionByKey, UpdateConfigRowByKey};

const TOKEN_ENDPOINT: &str = "https://localhost:8080/api/token";
const SAFETY_BUFFER_SECONDS: u64 = 5 * 60;
pub const GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN_REFRESH_SERVICE_API_KEY: &str =
    "google-play-android-developer-api-access-token-refresh-service-api-key";
pub const GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN: &str =
    "google-play-android-developer-api-access-token";

#[dsl(plural_name = refresh_google_play_android_developer_api_access_token_jobs, method(update = false, delete = true))]
#[table(
    accessor = refresh_google_play_android_developer_api_access_token_job,
    scheduled(refresh_google_play_android_developer_api_access_token),
    private,
)]
pub struct RefreshGooglePlayAndroidDeveloperApiAccessTokenJob {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,

    scheduled_at: ScheduleAt,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_at: u64,
}

fn to_string_error(error: SpacetimeDSLError) -> String {
    error.to_string()
}

#[spacetimedb::procedure]
pub fn refresh_google_play_android_developer_api_access_token(
    ctx: &mut spacetimedb::ProcedureContext,
    _job: RefreshGooglePlayAndroidDeveloperApiAccessTokenJob,
) -> Result<(), String> {
    // read API key from config
    let api_key = ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);
        let config = dsl
            .get_config_by_key(
                GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN_REFRESH_SERVICE_API_KEY,
            )
            .map_err(to_string_error)?;
        Ok::<_, String>(config.get_value().clone())
    })?;

    // request a new access token
    let request_body = format!("api_key={api_key}");

    let request = spacetimedb::http::Request::builder()
        .uri(TOKEN_ENDPOINT)
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(request_body)
        .map_err(|error| format!("Failed to build token request: {error}"))?;

    let response = ctx
        .http
        .send(request)
        .map_err(|error| format!("Token request failed: {error:?}"))?;

    let (parts, body) = response.into_parts();
    if parts.status != 200 {
        let body_text = body.into_string_lossy();
        return Err(format!(
            "Token request returned status {}: {body_text}",
            parts.status
        ));
    }

    let token_response: TokenResponse = serde_json::from_slice(&body.into_bytes())
        .map_err(|error| format!("Failed to parse token response: {error}"))?;

    // store the new access token and schedule the next refresh
    ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);

        // persist access token
        let mut config = dsl
            .get_config_by_key(GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN)
            .map_err(to_string_error)?;
        config.set_value(token_response.access_token.clone());
        dsl.update_config_by_key(config).map_err(to_string_error)?;

        // schedule next refresh 5 minutes before expiry
        let next_refresh_seconds = token_response.expires_at - SAFETY_BUFFER_SECONDS;
        let next_refresh_microseconds = (next_refresh_seconds * 1_000_000) as i64;
        let next_refresh_timestamp =
            Timestamp::from_micros_since_unix_epoch(next_refresh_microseconds);
        dsl.create_refresh_google_play_android_developer_api_access_token_job(
            CreateRefreshGooglePlayAndroidDeveloperApiAccessTokenJob {
                scheduled_at: ScheduleAt::Time(next_refresh_timestamp),
            },
        )
        .map_err(to_string_error)?;

        Ok::<_, String>(())
    })?;

    Ok(())
}

pub fn schedule_google_play_android_developer_api_access_token_refresh(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), SpacetimeDSLError> {
    // schedule immediately so the token is fetched on startup
    dsl.create_refresh_google_play_android_developer_api_access_token_job(
        CreateRefreshGooglePlayAndroidDeveloperApiAccessTokenJob {
            scheduled_at: ScheduleAt::Time(dsl.ctx().timestamp),
        },
    )?;

    Ok(())
}
