use spacetimedb::{Identity, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = wallets, method(update = true, delete = true))]
#[table(accessor = wallet, private)]
pub struct Wallet {
    #[primary_key]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    pub(crate) player_id: Identity,

    pub gems: u64,

    pub coins: u64,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
