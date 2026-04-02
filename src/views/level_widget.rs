use spacetimedb::SpacetimeType;
use spacetimedb::ViewContext;

use crate::tables::player::player__view;

#[derive(SpacetimeType)]
pub struct Level {
    amount: u32,
}

#[spacetimedb::view(accessor = level_widget, public)]
pub fn level_widget(ctx: &ViewContext) -> Option<Level> {
    let player = ctx.db.player().id().find(ctx.sender())?;
    Some(Level {
        amount: *player.get_level(),
    })
}
