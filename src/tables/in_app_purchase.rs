use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum InAppPurchasePrice {
    Five,
    TwentyFive,
    Fifty,
    Hundred,
}

impl InAppPurchasePrice {
    pub fn gems(&self) -> u64 {
        match self {
            InAppPurchasePrice::Five => 50,
            InAppPurchasePrice::TwentyFive => 300,
            InAppPurchasePrice::Fifty => 750,
            InAppPurchasePrice::Hundred => 2000,
        }
    }
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum InAppPurchaseStatus {
    Pending,
    Completed,
}

#[dsl(plural_name = in_app_purchases, method(update = true, delete = true))]
#[table(accessor = in_app_purchase, private)]
pub struct InAppPurchase {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    #[unique]
    #[create_wrapper]
    token: String,

    pub price: Option<InAppPurchasePrice>,

    pub region_code: Option<String>,

    pub status: InAppPurchaseStatus,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
