use spacetimedsl::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    constants::{INITIAL_COINS, INITIAL_GEMS, INITIAL_MAX_ENERGY},
    tables::{
        energy::{CreateEnergy, CreateEnergyRow},
        leaderboard_entry::{CreateLeaderboardEntry, CreateLeaderboardEntryRow},
        level_skin::{
            CreateLevelSkin, CreateLevelSkinRow, GetLevelSkinRowOptionByPlayerIdAndVariant,
            LevelSkinVariant, UpdateLevelSkinRowByPlayerIdAndVariant,
        },
        magnet_data::{CreateMagnetData, CreateMagnetDataRow},
        player::{AfterPlayerInsertHook, Player},
        player_movement_trail::{
            CreatePlayerMovementTrail, CreatePlayerMovementTrailRow, PlayerMovementTrailVariant,
        },
        player_skin::{
            CreatePlayerSkin, CreatePlayerSkinRow, GetPlayerSkinRowOptionByPlayerIdAndVariant,
            PlayerSkinVariant, UpdatePlayerSkinRowByPlayerIdAndVariant,
        },
        shield_data::{CreateShieldData, CreateShieldDataRow},
        wallet::{CreateWallet, CreateWalletRow},
    },
};

#[hook]
fn after_player_insert(dsl: &DSL<'_, T>, player: &Player) -> Result<(), SpacetimeDSLError> {
    let player_id = player.get_id();
    let now = dsl.ctx().timestamp()?;

    dsl.create_wallet(CreateWallet {
        player_id: player_id.clone(),
        gems: INITIAL_GEMS,
        coins: INITIAL_COINS,
    })?;

    dsl.create_energy(CreateEnergy {
        player_id: player_id.clone(),
        energy: INITIAL_MAX_ENERGY,
        energy_boundary_reached_at: None,
        last_energy_regeneration_at: now,
        last_energy_calculation_at: now,
    })?;

    dsl.create_leaderboard_entry(CreateLeaderboardEntry {
        player_id: player_id.clone(),
        score: 0,
        is_public: true,
    })?;

    dsl.create_magnet_data(CreateMagnetData {
        player_id: player_id.clone(),
        amount: 0,
        last_purchase_day: now,
        purchased_today: 0,
        range_upgrade_level: 0,
        duration_upgrade_level: 0,
        spawn_chance_upgrade_level: 0,
    })?;

    dsl.create_shield_data(CreateShieldData {
        player_id: player_id.clone(),
        amount: 0,
        last_purchase_day: now,
        purchased_today: 0,
        collisions_upgrade_level: 0,
        duration_upgrade_level: 0,
        spawn_chance_upgrade_level: 0,
    })?;

    for variant in PlayerSkinVariant::iter() {
        dsl.create_player_skin(CreatePlayerSkin {
            player_id: player_id.clone(),
            variant,
            purchased: false,
        })?;
    }

    for variant in LevelSkinVariant::iter() {
        dsl.create_level_skin(CreateLevelSkin {
            player_id: player_id.clone(),
            variant,
            purchased: false,
        })?;
    }

    for variant in PlayerMovementTrailVariant::iter() {
        dsl.create_player_movement_trail(CreatePlayerMovementTrail {
            player_id: player_id.clone(),
            variant,
            purchased: false,
        })?;
    }

    let mut player_skin = dsl
        .get_player_skin_by_player_id_and_variant(player_id.clone(), &PlayerSkinVariant::Default)?;
    player_skin.set_purchased(true);
    dsl.update_player_skin_by_player_id_and_variant(player_skin)?;

    let mut level_skin =
        dsl.get_level_skin_by_player_id_and_variant(player_id, &LevelSkinVariant::NeonSectorOne)?;
    level_skin.set_purchased(true);
    dsl.update_level_skin_by_player_id_and_variant(level_skin)?;

    Ok(())
}
