use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::cheat_attempt_log::{CheatOrError, cheat_attempt};
use crate::constants::ENERGY_PER_PURCHASE_OR_AD;
use crate::purchase_price;
use crate::remove_from_wallet::charge_wallet;
use crate::tables::energy::{Energy, GetEnergyRowOptionByPlayerId, UpdateEnergyRowByPlayerId};
use crate::tables::player::Player;
use crate::tables::playthrough::{
    GetPlaythroughRowOptionById, Playthrough, UpdatePlaythroughRowById,
};
use crate::tables::purchase::Currency;

use super::calculation::{calculate_regen_completion_timestamp, recalculate_energy_for_idle};
use super::scheduling::is_in_out_of_energy_pause;

pub fn add_energy(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    active_playthrough: &Option<Playthrough>,
) -> Result<Energy, CheatOrError> {
    let mut energy = dsl.get_energy_by_player_id(&player.get_id())?;
    let now = dsl.ctx().timestamp;

    match active_playthrough {
        Some(playthrough) => {
            if !is_in_out_of_energy_pause(playthrough) {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Tried to get energy without being in an OutOfEnergy pause",
                ));
            }
        }
        None => {
            recalculate_energy_for_idle(&mut energy, now);
            if *energy.get_energy() > 1 {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Tried to watch an ad or pay gems to get energy with energy above 1",
                ));
            }
        }
    }

    energy.set_energy(energy.get_energy() + ENERGY_PER_PURCHASE_OR_AD);
    energy.set_last_energy_calculation_at(now);

    match active_playthrough {
        Some(_) => {
            energy.set_energy_boundary_reached_at(None);
        }
        None => {
            energy.set_energy_boundary_reached_at(calculate_regen_completion_timestamp(
                *energy.get_energy(),
                now,
            ));
        }
    }

    let energy = dsl.update_energy_by_player_id(energy)?;

    Ok(energy)
}

pub fn purchase_energy(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    active_playthrough: &Option<Playthrough>,
    currency: &Option<Currency>,
) -> Result<(u64, u64), CheatOrError> {
    let (gems_charged, coins_charged) = match active_playthrough {
        Some(playthrough) => {
            let price = purchase_price::energy_purchase_price(*playthrough.get_energy_purchases());
            let charge = charge_wallet(dsl, player, &price, currency)?;

            let mut playthrough = dsl.get_playthrough_by_id(playthrough.get_id())?;
            playthrough.set_energy_purchases(*playthrough.get_energy_purchases() + 1);
            dsl.update_playthrough_by_id(playthrough)?;

            charge
        }
        None => {
            let price = purchase_price::energy_purchase_price_without_playthrough();
            charge_wallet(dsl, player, &price, currency)?
        }
    };

    add_energy(dsl, player, active_playthrough)?;

    Ok((gems_charged, coins_charged))
}
