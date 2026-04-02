use spacetimedb::ProcedureContext;
use spacetimedsl::{itertools::Either, prelude::*};

use crate::{
    scheduled_functions::refresh_google_play_android_developer_api_access_token::GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN,
    tables::{
        cheat_attempt_log::{CreateCheatAttemptLog, CreateCheatAttemptLogRow},
        config::GetConfigRowOptionByKey,
        in_app_purchase::{
            CreateInAppPurchase, CreateInAppPurchaseRow, DeleteInAppPurchaseRowById,
            GetInAppPurchaseRowOptionById, GetInAppPurchaseRowOptionByToken, InAppPurchaseId,
            InAppPurchasePrice, InAppPurchaseStatus, UpdateInAppPurchaseRowById,
        },
        player::{GetPlayerRowOptionById, PlayerId},
        wallet::{GetWalletRowOptionByPlayerId, UpdateWalletRowByPlayerId},
    },
};

const PACKAGE_NAME: &str = "mobile-sci-fi-endless-runner"; // Note: This is not a valid package name.

const PRODUCT_ID_FIVE: &str = "gems_five";
const PRODUCT_ID_TWENTY_FIVE: &str = "gems_twenty_five";
const PRODUCT_ID_FIFTY: &str = "gems_fifty";
const PRODUCT_ID_HUNDRED: &str = "gems_hundred";

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductPurchaseResponse {
    product_id: String,
    purchase_state: String,
    region_code: String,
}

fn product_id_to_price(product_id: &str) -> Result<InAppPurchasePrice, String> {
    match product_id {
        PRODUCT_ID_FIVE => Ok(InAppPurchasePrice::Five),
        PRODUCT_ID_TWENTY_FIVE => Ok(InAppPurchasePrice::TwentyFive),
        PRODUCT_ID_FIFTY => Ok(InAppPurchasePrice::Fifty),
        PRODUCT_ID_HUNDRED => Ok(InAppPurchasePrice::Hundred),
        _ => Err(format!("Unknown product ID: {product_id}")),
    }
}

fn to_string_error(error: SpacetimeDSLError) -> String {
    error.to_string()
}

#[spacetimedb::procedure]
pub fn handle_in_app_purchase(
    ctx: &mut ProcedureContext,
    purchase_token: String,
) -> Result<(), String> {
    // validate player, check for duplicate token, reserve token with pending row, read access token
    let (access_token, in_app_purchase_id) = match ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);

        // authenticate player
        let sender = dsl.ctx().sender().map_err(to_string_error)?;
        let player_id = PlayerId::new(sender);
        let player = dsl.get_player_by_id(&player_id).map_err(to_string_error)?;

        if let Some(ban_reason) = player.get_is_banned() {
            return Err(format!("Player is banned: {ban_reason}"));
        }

        // check for duplicate token
        let existing = dsl.get_in_app_purchase_by_token(&purchase_token);
        if existing.is_ok() {
            // log cheat attempt
            dsl.create_cheat_attempt_log(CreateCheatAttemptLog {
                player_id: player.get_id(),
                reason: format!("Duplicate in-app purchase token: {purchase_token}"),
            })
            .map_err(to_string_error)?;

            log::warn!(
                "Cheat attempt by player {:?}: Duplicate in-app purchase token: {purchase_token}",
                player.get_id()
            );

            return Ok(Either::Right(()));
        }

        // reserve the token with a pending row
        let in_app_purchase = dsl
            .create_in_app_purchase(CreateInAppPurchase {
                player_id: player.get_id(),
                token: purchase_token.clone(),
                price: None,
                region_code: None,
                status: InAppPurchaseStatus::Pending,
            })
            .map_err(to_string_error)?;

        // read access token from config
        let access_token_config = dsl
            .get_config_by_key(GOOGLE_PLAY_ANDROID_DEVELOPER_API_ACCESS_TOKEN)
            .map_err(to_string_error)?;

        Ok(Either::Left((
            access_token_config.get_value().clone(),
            in_app_purchase.get_id(),
        )))
    })? {
        Either::Left((access_token, in_app_purchase_id)) => (access_token, in_app_purchase_id),
        Either::Right(()) => return Ok(()),
    };

    // validate purchase with Google Play API
    let api_url = format!(
        "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{PACKAGE_NAME}/purchases/productsv2/tokens/{purchase_token}"
    );

    let request = spacetimedb::http::Request::builder()
        .uri(&api_url)
        .method("GET")
        .header("Authorization", format!("Bearer {access_token}"))
        .body(())
        .map_err(|error| format!("Failed to build Google Play API request: {error}"))?;

    let api_result = ctx.http.send(request);

    // parse and validate the API response
    let purchase_response = match api_result {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            if parts.status != 200 {
                let body_text = body.into_string_lossy();
                cleanup_pending_purchase(ctx, &in_app_purchase_id)?;
                return Err(format!(
                    "Google Play API returned status {}: {body_text}",
                    parts.status
                ));
            }

            let parsed: ProductPurchaseResponse = serde_json::from_slice(&body.into_bytes())
                .map_err(|error| format!("Failed to parse Google Play API response: {error}"))?;

            if parsed.purchase_state != "PURCHASED" {
                cleanup_pending_purchase(ctx, &in_app_purchase_id)?;
                return Err(format!(
                    "Purchase state is not PURCHASED: {}",
                    parsed.purchase_state
                ));
            }

            parsed
        }
        Err(error) => {
            cleanup_pending_purchase(ctx, &in_app_purchase_id)?;
            return Err(format!("Google Play API request failed: {error:?}"));
        }
    };

    // map product ID to price tier
    let price = product_id_to_price(&purchase_response.product_id)?;
    let gems = price.gems();

    // complete the purchase: update row, credit wallet
    ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);
        let sender = dsl.ctx().sender().map_err(to_string_error)?;
        let player_id = PlayerId::new(sender);

        // update in-app purchase to completed
        let mut in_app_purchase = dsl
            .get_in_app_purchase_by_id(&in_app_purchase_id)
            .map_err(to_string_error)?;

        in_app_purchase.set_price(Some(price.clone()));
        in_app_purchase.set_region_code(Some(purchase_response.region_code.clone()));
        in_app_purchase.set_status(InAppPurchaseStatus::Completed);

        dsl.update_in_app_purchase_by_id(in_app_purchase)
            .map_err(to_string_error)?;

        // credit gems to wallet
        let mut wallet = dsl
            .get_wallet_by_player_id(player_id)
            .map_err(to_string_error)?;

        wallet.set_gems(wallet.get_gems() + gems);

        dsl.update_wallet_by_player_id(wallet)
            .map_err(to_string_error)?;

        Ok::<_, String>(())
    })?;

    Ok(())
}

fn cleanup_pending_purchase(
    ctx: &mut ProcedureContext,
    in_app_purchase_id: &InAppPurchaseId,
) -> Result<(), String> {
    ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);
        dsl.delete_in_app_purchase_by_id(in_app_purchase_id)
            .map_err(to_string_error)?;
        Ok::<_, String>(())
    })
}
