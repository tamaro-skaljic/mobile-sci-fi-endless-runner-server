use spacetimedb::{Identity, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = shield_data, method(update = true, delete = true))]
#[table(accessor = shield_data, private)]
pub struct ShieldData {
    #[primary_key]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    pub(crate) player_id: Identity,

    pub amount: u8,

    pub last_purchase_day: Timestamp,

    pub purchased_today: u8,

    pub collisions_upgrade_level: u8,

    pub duration_upgrade_level: u8,

    pub spawn_chance_upgrade_level: u8,

    modified_at: Option<Timestamp>,
}
