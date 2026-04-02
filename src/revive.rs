use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    checks::revive_is_allowed::revive_is_allowed,
    purchase_price,
    remove_from_wallet::charge_wallet,
    tables::{
        player::Player,
        playthrough::{Playthrough, UpdatePlaythroughRowById},
        purchase::Currency,
        revive::{CreateRevive, CreateReviveRow, ReviveType},
    },
};

pub fn revive(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    revive_type: ReviveType,
) -> Result<(), CheatOrError> {
    let (mut active_playthrough, _revive_count) = revive_is_allowed(dsl, player.get_id())?;

    dsl.create_revive(CreateRevive {
        player_id: player.get_id(),
        playthrough_id: active_playthrough.get_id(),
        revive_type,
    })?;

    active_playthrough.set_revive_count(*active_playthrough.get_revive_count() + 1);
    dsl.update_playthrough_by_id(active_playthrough)?;

    Ok(())
}

pub fn purchase_revive(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    active_playthrough: &Option<Playthrough>,
    currency: &Option<Currency>,
) -> Result<(u64, u64), CheatOrError> {
    let playthrough = match active_playthrough {
        Some(playthrough) => playthrough,
        None => {
            return Err(cheat_attempt(
                dsl,
                player,
                "Tried to purchase a revive without an active playthrough",
            ));
        }
    };

    let price = purchase_price::revive_purchase_price(*playthrough.get_revive_count());
    let charge = charge_wallet(dsl, player, &price, currency)?;

    revive(dsl, player, ReviveType::Gems(charge.0 as u8))?;

    Ok(charge)
}
