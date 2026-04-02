use spacetimedb::{ReducerContext, SpacetimeType, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    cheat_attempt_log::cheat_attempt,
    checks::{
        player_has_playthrough::player_has_playthrough,
        playthrough_is_active::playthrough_is_active,
    },
    or_ok_on_cheat,
    tables::{
        magnet_data::{GetMagnetDataRowOptionByPlayerId, UpdateMagnetDataRowByPlayerId},
        playthrough::GetPlaythroughRowOptionById,
        shield_data::{GetShieldDataRowOptionByPlayerId, UpdateShieldDataRowByPlayerId},
    },
};

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum PowerUpType {
    Shield,
    Magnet,
}

macro_rules! use_power_up_branch {
    ($dsl:expr, $player:expr, $get:ident, $update:ident, $name:expr) => {{
        let mut data = $dsl.$get($player)?;
        let amount = *data.get_amount();
        if amount == 0 {
            or_ok_on_cheat!(Err(cheat_attempt(
                $dsl,
                $player,
                &format!("Tried to use a {} with amount 0", $name),
            )));
        }
        data.set_amount(amount - 1);
        $dsl.$update(data)?;
    }};
}

#[reducer]
pub fn use_power_up(
    ctx: &ReducerContext,
    power_up_type: PowerUpType,
) -> Result<(), SpacetimeDSLError> {
    let (dsl, player) = get_authenticated_player(ctx)?;

    let last_playthrough_id = or_ok_on_cheat!(player_has_playthrough(&dsl, &player));
    let active_playthrough = dsl.get_playthrough_by_id(last_playthrough_id)?;
    or_ok_on_cheat!(playthrough_is_active(&dsl, &player, &active_playthrough));

    match power_up_type {
        PowerUpType::Shield => use_power_up_branch!(
            &dsl,
            &player,
            get_shield_data_by_player_id,
            update_shield_data_by_player_id,
            "shield"
        ),
        PowerUpType::Magnet => use_power_up_branch!(
            &dsl,
            &player,
            get_magnet_data_by_player_id,
            update_magnet_data_by_player_id,
            "magnet"
        ),
    }

    Ok(())
}
