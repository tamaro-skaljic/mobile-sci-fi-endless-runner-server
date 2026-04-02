use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::cheat_attempt_log::{CheatOrError, cheat_attempt};
use crate::purchase_price::PricingMode;
use crate::tables::player::Player;
use crate::tables::purchase::Currency;
use crate::tables::wallet::{GetWalletRowOptionByPlayerId, UpdateWalletRowByPlayerId, Wallet};

pub fn remove_from_wallet(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    gems: u64,
    coins: u64,
) -> Result<Wallet, CheatOrError> {
    let mut wallet = dsl.get_wallet_by_player_id(player)?;

    let current_gems = *wallet.get_gems();
    let current_coins = *wallet.get_coins();

    if current_gems < gems || current_coins < coins {
        return Err(cheat_attempt(
            dsl,
            player,
            &format!(
                "Tried to purchase without enough currency (need {gems} gems and {coins} coins, have {current_gems} gems and {current_coins} coins)",
            ),
        ));
    }

    wallet.set_gems(current_gems - gems);
    wallet.set_coins(current_coins - coins);

    let wallet = dsl.update_wallet_by_player_id(wallet)?;

    Ok(wallet)
}

pub fn validate_currency_and_resolve_charge(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    price: &PricingMode,
    currency: &Option<Currency>,
) -> Result<(u64, u64), CheatOrError> {
    match price {
        PricingMode::Gems(gems) => {
            if currency.is_some() {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Tried to specify currency for a fixed-currency purchase",
                ));
            }
            Ok((*gems, 0))
        }
        PricingMode::Coins(coins) => {
            if currency.is_some() {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Tried to specify currency for a fixed-currency purchase",
                ));
            }
            Ok((0, *coins))
        }
        PricingMode::GemsAndCoins(price) => {
            if currency.is_some() {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    "Tried to specify currency for a fixed-currency purchase",
                ));
            }
            Ok((price.gems, price.coins))
        }
        PricingMode::GemsOrCoins(price) => {
            let chosen = match currency {
                Some(currency) => currency,
                None => {
                    return Err(cheat_attempt(
                        dsl,
                        player,
                        "Tried to purchase without specifying currency",
                    ));
                }
            };
            match chosen {
                Currency::Gems => Ok((price.gems, 0)),
                Currency::Coins => Ok((0, price.coins)),
            }
        }
    }
}

pub fn charge_wallet(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    price: &PricingMode,
    currency: &Option<Currency>,
) -> Result<(u64, u64), CheatOrError> {
    let (gems, coins) = validate_currency_and_resolve_charge(dsl, player, price, currency)?;
    remove_from_wallet(dsl, player, gems, coins)?;
    Ok((gems, coins))
}
