use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::tables::player::PlayerId;
use crate::tables::wallet::{GetWalletRowOptionByPlayerId, UpdateWalletRowByPlayerId, Wallet};

pub fn add_to_wallet(
    dsl: &DSL<'_, ReducerContext>,
    player: impl Into<PlayerId> + Clone,
    gems: u64,
    coins: u64,
) -> Result<Wallet, SpacetimeDSLError> {
    let mut wallet = dsl.get_wallet_by_player_id(player)?;

    wallet.set_gems(wallet.get_gems() + gems);
    wallet.set_coins(wallet.get_coins() + coins);

    let wallet = dsl.update_wallet_by_player_id(wallet)?;

    Ok(wallet)
}
