use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = advertisement_watches, method(update = true, delete = true))]
#[table(accessor = advertisement_watch, private)]
pub struct AdvertisementWatch {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    pub status: AdWatchStatus,

    // How many gems the player would have needed to pay in order to get the same reward instead of watching the ad, if an option to get the reward by gems existed.
    instead_of: Option<u32>,

    ad_type: AdType,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum AdWatchStatus {
    /// The ad is currently being watched.
    Watching,
    /// The ad watch was cancelled by the player.
    Cancelled,
    /// The ad watch was completed successfully.
    Finished,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum AdType {
    Revive,
    DoubleCoins,
    Gems,
    Energy,
}
