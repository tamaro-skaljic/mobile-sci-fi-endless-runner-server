use spacetimedb::{Identity, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = leaderboard_entries, method(update = true, delete = true))]
#[table(accessor = leaderboard_entry, private)]
pub struct LeaderboardEntry {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    pub score: u64,

    pub is_public: bool,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
