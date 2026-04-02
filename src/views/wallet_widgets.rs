use spacetimedb::SpacetimeType;
use spacetimedb::ViewContext;

use crate::tables::wallet::wallet__view;

macro_rules! wallet_widget_view {
    ($view_name:ident, $getter:ident, $ty:ident) => {
        #[derive(SpacetimeType)]
        pub struct $ty {
            amount: u64,
        }

        #[spacetimedb::view(accessor = $view_name, public)]
        pub fn $view_name(ctx: &ViewContext) -> Option<$ty> {
            let wallet = ctx.db.wallet().player_id().find(ctx.sender())?;
            Some($ty {
                amount: *wallet.$getter(),
            })
        }
    };
}

wallet_widget_view!(gem_widget, get_gems, Gems);
wallet_widget_view!(coin_widget, get_coins, Coins);
