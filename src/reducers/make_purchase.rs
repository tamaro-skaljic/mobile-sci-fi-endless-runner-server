use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    checks::{
        non_ended_playthrough::get_non_ended_playthrough,
        playthrough_is_in_pause::playthrough_is_in_pause_reason,
    },
    energy::purchase_energy,
    or_ok_on_cheat,
    revive::purchase_revive,
    shop::{
        cosmetic::{purchase_level_skin, purchase_player_movement_trail, purchase_player_skin},
        power_up::{purchase_magnet, purchase_shield},
        power_up_upgrade::{
            MagnetUpgrade, ShieldUpgrade, purchase_magnet_upgrade, purchase_shield_upgrade,
        },
    },
    tables::{
        playthrough::{PauseReason, Playthrough},
        purchase::{CreatePurchase, CreatePurchaseRow, Currency, PurchaseVariant},
    },
};

#[reducer]
pub fn make_purchase(
    ctx: &ReducerContext,
    variant: PurchaseVariant,
    currency: Option<Currency>,
) -> Result<(), SpacetimeDSLError> {
    let (dsl, player) = get_authenticated_player(ctx)?;

    let non_ended_playthrough = get_non_ended_playthrough(&dsl, &player);

    match &variant {
        PurchaseVariant::Energy => {
            if let Some(ref playthrough) = non_ended_playthrough {
                or_ok_on_cheat!(playthrough_is_in_pause_reason(
                    &dsl,
                    &player,
                    playthrough,
                    &PauseReason::OutOfEnergy,
                ));
            }
        }
        PurchaseVariant::Revive => {
            if let Some(ref playthrough) = non_ended_playthrough {
                or_ok_on_cheat!(playthrough_is_in_pause_reason(
                    &dsl,
                    &player,
                    playthrough,
                    &PauseReason::Revive,
                ));
            }
        }
        _ => or_ok_on_cheat!(log_cheat_attempt_if_non_ended_playthrough(
            &dsl,
            &player,
            &non_ended_playthrough,
            &format!("{variant}"),
        )),
    }

    let (gems_charged, coins_charged) = match &variant {
        PurchaseVariant::Energy => {
            or_ok_on_cheat!(purchase_energy(
                &dsl,
                &player,
                &non_ended_playthrough,
                &currency
            ))
        }
        PurchaseVariant::Revive => {
            or_ok_on_cheat!(purchase_revive(
                &dsl,
                &player,
                &non_ended_playthrough,
                &currency
            ))
        }
        PurchaseVariant::Magnet => or_ok_on_cheat!(purchase_magnet(&dsl, &player, &currency)),
        PurchaseVariant::MagnetRangeUpgrade => {
            or_ok_on_cheat!(purchase_magnet_upgrade(
                &dsl,
                &player,
                &currency,
                MagnetUpgrade::Range
            ))
        }
        PurchaseVariant::MagnetDurationUpgrade => {
            or_ok_on_cheat!(purchase_magnet_upgrade(
                &dsl,
                &player,
                &currency,
                MagnetUpgrade::Duration
            ))
        }
        PurchaseVariant::MagnetSpawnChanceUpgrade => {
            or_ok_on_cheat!(purchase_magnet_upgrade(
                &dsl,
                &player,
                &currency,
                MagnetUpgrade::SpawnChance,
            ))
        }
        PurchaseVariant::Shield => or_ok_on_cheat!(purchase_shield(&dsl, &player, &currency)),
        PurchaseVariant::ShieldCollisionsUpgrade => {
            or_ok_on_cheat!(purchase_shield_upgrade(
                &dsl,
                &player,
                &currency,
                ShieldUpgrade::Collisions,
            ))
        }
        PurchaseVariant::ShieldDurationUpgrade => {
            or_ok_on_cheat!(purchase_shield_upgrade(
                &dsl,
                &player,
                &currency,
                ShieldUpgrade::Duration
            ))
        }
        PurchaseVariant::ShieldSpawnChanceUpgrade => {
            or_ok_on_cheat!(purchase_shield_upgrade(
                &dsl,
                &player,
                &currency,
                ShieldUpgrade::SpawnChance,
            ))
        }
        PurchaseVariant::PlayerSkin(skin_variant) => {
            or_ok_on_cheat!(purchase_player_skin(&dsl, &player, &currency, skin_variant))
        }
        PurchaseVariant::LevelSkin(skin_variant) => {
            or_ok_on_cheat!(purchase_level_skin(&dsl, &player, &currency, skin_variant))
        }
        PurchaseVariant::PlayerMovementTrail(trail_variant) => {
            or_ok_on_cheat!(purchase_player_movement_trail(
                &dsl,
                &player,
                &currency,
                trail_variant
            ))
        }
    };

    dsl.create_purchase(CreatePurchase {
        player_id: player.get_id(),
        variant,
        gems: gems_charged,
        coins: coins_charged,
    })?;

    Ok(())
}

fn log_cheat_attempt_if_non_ended_playthrough(
    dsl: &DSL<'_, ReducerContext>,
    player: &crate::tables::player::Player,
    non_ended_playthrough: &Option<Playthrough>,
    reason: &str,
) -> Result<(), CheatOrError> {
    if non_ended_playthrough.is_some() {
        return Err(cheat_attempt(
            dsl,
            player,
            &format!("Tried to purchase {reason} during an active playthrough"),
        ));
    }
    Ok(())
}
