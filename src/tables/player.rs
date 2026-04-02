use spacetimedb::{Identity, Timestamp, table};
use spacetimedsl::prelude::*;

use super::level_skin::LevelSkinVariant;
use super::player_movement_trail::PlayerMovementTrailVariant;
use super::player_skin::PlayerSkinVariant;

#[dsl(plural_name = players, method(update = true, delete = true), hook(after(insert)))]
#[table(accessor = player, private)]
pub struct Player {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = super::advertisement,                         table = advertisement_watch      )]
    #[referenced_by(path = super::cheat_attempt_log,                     table = cheat_attempt_log        )]
    #[referenced_by(path = super::energy,                                table = energy                   )]
    #[referenced_by(path = super::in_app_purchase,                       table = in_app_purchase          )]
    #[referenced_by(path = super::leaderboard_entry,                     table = leaderboard_entry        )]
    #[referenced_by(path = super::level_skin,                            table = level_skin               )]
    #[referenced_by(path = super::magnet_data,                           table = magnet_data              )]
    #[referenced_by(path = super::player_movement_trail,                 table = player_movement_trail    )]
    #[referenced_by(path = super::player_skin,                           table = player_skin              )]
    #[referenced_by(path = super::playthrough,                           table = playthrough              )]
    #[referenced_by(path = super::purchase,                              table = purchase                 )]
    #[referenced_by(path = super::revive,                                table = revive                   )]
    #[referenced_by(path = super::shield_data,                           table = shield_data              )]
    #[referenced_by(path = super::wallet,                                table = wallet                   )]
    #[referenced_by(path = crate::scheduled_functions::energy_depletion, table = energy_depletion_schedule)]
    pub id: Identity,

    /// If players never changed their names there is no corresponding PlayerNameResult because during registration a compliant name for the player is generated, so the name is always valid - so do not add a foreign key or use the pk's wrapper type here..
    #[unique]
    pub name: String,

    pub is_banned: Option<String>,

    /// Used to get the player's last playthrough by O(1) instead of O(n) (through iterating through the playthrough table and finding the one with end = None).
    /// Referential Integrity is not enforced for this column, but there is no risk of dangling references, as playthroughs are never deleted, except when the player is deleted.
    #[use_wrapper(super::playthrough::PlaythroughId)]
    pub last_playthrough_id: Option<u64>,

    pub next_get_gems_ad_watch_available_at: Timestamp,

    pub last_daily_reward_claimed_at: Option<Timestamp>,

    pub number_of_claimed_daily_rewards: u16,

    pub time_difference_from_utc_in_minutes: i16,

    pub player_skin: PlayerSkinVariant,

    pub level_skin: LevelSkinVariant,

    pub player_movement_trail: Option<PlayerMovementTrailVariant>,

    pub level: u32,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
