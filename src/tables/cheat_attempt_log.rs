use spacetimedb::{Identity, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = cheat_attempt_logs, method(update = false, delete = true))]
#[table(accessor = cheat_attempt_log, private)]
pub struct CheatAttemptLog {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    #[index(btree)]
    reason: String,

    created_at: Timestamp,
}
