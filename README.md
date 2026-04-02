# Mobile Sci-Fi Endless Runner Server

This repository contains the server for an arcade Sci-Fi Endless Runner game for mobile devices.

## Table of Contents

- [Source Files](#source-files)
  - [Root Modules](#root-modules)
  - [Tables (`src/tables/`)](#tables-srctables)
  - [Reducers (`src/reducers/`)](#reducers-srcreducers)
  - [Admin (`src/admin/`)](#admin-srcadmin)
  - [Checks (`src/checks/`)](#checks-srcchecks)
  - [Views (`src/views/`)](#views-srcviews)
  - [Energy (`src/energy/`)](#energy-srcenergy)
  - [Shop (`src/shop/`)](#shop-srcshop)
  - [Hooks (`src/hooks/`)](#hooks-srchooks)
  - [Scheduled Functions (`src/scheduled_functions/`)](#scheduled-functions-srcscheduled_functions)
  - [Procedures (`src/procedures/`)](#procedures-srcprocedures)
- [Honorable Features](#honorable-features)
  - [Energy System](#energy-system)
  - [Cheat Detection \& Logging](#cheat-detection--logging)
  - [Player Name Moderation](#player-name-moderation)
  - [In App Purchase Verification](#in-app-purchase-verification)
- [📜 Licensing](#-licensing)

## Source Files

### Root Modules

- [`src/lib.rs`](src/lib.rs) — Crate root that declares all public modules and re-exports for the server.
- [`src/constants.rs`](src/constants.rs) — Game-wide numeric constants
- [`src/authenticated_player.rs`](src/authenticated_player.rs) — Helper that retrieves and authenticates the calling player from a reducer context, rejecting banned players.
- [`src/add_to_wallet.rs`](src/add_to_wallet.rs) — Adds gems and/or coins to a player's wallet.
- [`src/remove_from_wallet.rs`](src/remove_from_wallet.rs) — Removes gems and/or coins from a player's wallet with cheat detection, and resolves pricing-mode currency selection.
- [`src/ban_player.rs`](src/ban_player.rs) — Core logic for banning a player by setting a ban reason and checking whether a player is banned.
- [`src/daily_reward_gems.rs`](src/daily_reward_gems.rs) — Returns the gem reward amount for a given daily-reward claim count using a lookup table.
- [`src/cheat_attempt_log.rs`](src/cheat_attempt_log.rs) — Defines the cheat-attempt error type, logs cheat attempts to the database, and provides the `or_ok_on_cheat!` macro for silently ignoring cheat attempts (because otherwise the transaction would be rolled back, which means the cheat attempt would not be persisted).
- [`src/purchase_price.rs`](src/purchase_price.rs) — Calculates prices for energy, revives, power-ups, power-up upgrades, and cosmetics, with escalating cost formulas and ad-availability checks.
- [`src/revive.rs`](src/revive.rs) — Executes a revive during a playthrough and handles gem-based revive purchases.
- [`src/player_name_generator.rs`](src/player_name_generator.rs) — Generates unique random player names in the format "Player" followed by a 10-digit number.

### Tables (`src/tables/`)

- [`src/tables/player.rs`](src/tables/player.rs) — Defines the Player table with identity, name, ban status, cosmetics, level, daily reward tracking, and foreign-key references to all related tables.
- [`src/tables/wallet.rs`](src/tables/wallet.rs) — Defines the Wallet table storing each player's gem and coin balances.
- [`src/tables/playthrough.rs`](src/tables/playthrough.rs) — Defines the Playthrough table tracking score, coins, gems, pause history, end reason, and revive count for each game session.
- [`src/tables/energy.rs`](src/tables/energy.rs) — Defines the Energy table tracking a player's current energy, regeneration timestamps, and energy-change direction.
- [`src/tables/config.rs`](src/tables/config.rs) — Defines the Config table for storing server-wide key-value configuration entries.
- [`src/tables/leaderboard_entry.rs`](src/tables/leaderboard_entry.rs) — Defines the LeaderboardEntry table storing player high scores and public visibility.
- [`src/tables/advertisement.rs`](src/tables/advertisement.rs) — Defines the AdvertisementWatch table tracking ad watch sessions with status, type, and gem-equivalent cost.
- [`src/tables/revive.rs`](src/tables/revive.rs) — Defines the Revive table recording each revive event with its type (gems or ad watch) and associated playthrough.
- [`src/tables/player_name.rs`](src/tables/player_name.rs) — Defines the PlayerName table for tracking player name moderation status (pending, approved, rejected).
- [`src/tables/purchase.rs`](src/tables/purchase.rs) — Defines the Purchase table logging all in-game purchases with variant, gem cost, and coin cost.
- [`src/tables/shield_data.rs`](src/tables/shield_data.rs) — Defines the ShieldData table storing a player's shield inventory, daily purchase count, and upgrade levels.
- [`src/tables/magnet_data.rs`](src/tables/magnet_data.rs) — Defines the MagnetData table storing a player's magnet inventory, daily purchase count, and upgrade levels.
- [`src/tables/player_skin.rs`](src/tables/player_skin.rs) — Defines the PlayerSkin table and its variant enum tracking which player skins a player has purchased.
- [`src/tables/level_skin.rs`](src/tables/level_skin.rs) — Defines the LevelSkin table and its variant enum tracking which level skins a player has purchased.
- [`src/tables/player_movement_trail.rs`](src/tables/player_movement_trail.rs) — Defines the PlayerMovementTrail table and its variant enum tracking which movement trails a player has purchased.
- [`src/tables/cheat_attempt_log.rs`](src/tables/cheat_attempt_log.rs) — Defines the CheatAttemptLog table recording detected cheat attempts with player ID and reason.
- [`src/tables/in_app_purchase.rs`](src/tables/in_app_purchase.rs) — Defines the InAppPurchase table for tracking real-money purchases with token, price tier, status, and region code.

### Reducers (`src/reducers/`)

- [`src/reducers/connect_client.rs`](src/reducers/connect_client.rs) — Handles client connection by loading an existing player or creating a new one with default values. Banned players are directly disconnected.
- [`src/reducers/disconnect_client.rs`](src/reducers/disconnect_client.rs) — Handles client disconnection by updating the player's modified_at timestamp.
- [`src/reducers/begin_playthrough.rs`](src/reducers/begin_playthrough.rs) — Starts a new playthrough after verifying no active session exists and the player has energy.
- [`src/reducers/end_playthrough.rs`](src/reducers/end_playthrough.rs) — Ends an active playthrough, validates the end reason against the current pause state, and updates energy regeneration.
- [`src/reducers/pause_playthrough.rs`](src/reducers/pause_playthrough.rs) — Pauses an active unpaused playthrough, stops energy consumption, and cancels the depletion schedule.
- [`src/reducers/continue_playthrough.rs`](src/reducers/continue_playthrough.rs) — Resumes a paused playthrough, restarts energy consumption, and reschedules energy depletion.
- [`src/reducers/make_purchase.rs`](src/reducers/make_purchase.rs) — Central purchase reducer that dispatches to the appropriate purchase handler based on the variant (energy, revive, power-ups, cosmetics).
- [`src/reducers/apply_cosmetic.rs`](src/reducers/apply_cosmetic.rs) — Applies a purchased cosmetic (player skin, level skin, or movement trail) to the player.
- [`src/reducers/begin_ad_watch.rs`](src/reducers/begin_ad_watch.rs) — Initiates an ad watch session after validating eligibility based on ad type and current game state.
- [`src/reducers/end_ad_watch.rs`](src/reducers/end_ad_watch.rs) — Completes or cancels an ad watch and grants the appropriate reward (energy, revive, gems, or double coins).
- [`src/reducers/sync_time.rs`](src/reducers/sync_time.rs) — Syncs the player's UTC time offset for local-day calculations, validating the offset is within valid bounds.
- [`src/reducers/use_power_up.rs`](src/reducers/use_power_up.rs) — Consumes one shield or magnet from the player's inventory during an active playthrough.
- [`src/reducers/claim_daily_reward.rs`](src/reducers/claim_daily_reward.rs) — Claims the daily gem reward if the player has not already claimed one today.
- [`src/reducers/rename_player.rs`](src/reducers/rename_player.rs) — Renames a player after validating name length and checking whether the name was already rejected by moderation.

### Admin (`src/admin/`)

- [`src/admin/init_database.rs`](src/admin/init_database.rs) — Initializes the database on first deploy by scheduling the player name moderation job and the Google Play token refresh.
- [`src/admin/ban_player.rs`](src/admin/ban_player.rs) — Admin reducer that bans a player by identity after verifying admin access.
- [`src/admin/unban_player.rs`](src/admin/unban_player.rs) — Admin reducer that unbans a player by clearing the ban reason after verifying admin access.
- [`src/admin/manage_config.rs`](src/admin/manage_config.rs) — Admin reducer for creating, updating, or deleting server configuration key-value pairs.

### Checks (`src/checks/`)

- [`src/checks/is_admin_client.rs`](src/checks/is_admin_client.rs) — Verifies that the calling client is the module owner (administrator), logging a cheat attempt otherwise.
- [`src/checks/is_same_local_day.rs`](src/checks/is_same_local_day.rs) — Determines whether two timestamps fall on the same calendar day given a UTC offset in minutes.
- [`src/checks/non_ended_playthrough.rs`](src/checks/non_ended_playthrough.rs) — Returns the player's current non-ended playthrough if one exists, or None.
- [`src/checks/player_has_playthrough.rs`](src/checks/player_has_playthrough.rs) — Asserts that the player has at least one playthrough, returning its ID or logging a cheat attempt.
- [`src/checks/playthrough_is_active.rs`](src/checks/playthrough_is_active.rs) — Asserts that a playthrough has not ended, logging a cheat attempt if it has.
- [`src/checks/playthrough_is_active_and_unpaused.rs`](src/checks/playthrough_is_active_and_unpaused.rs) — Asserts that a playthrough is both active and not paused.
- [`src/checks/playthrough_is_in_pause.rs`](src/checks/playthrough_is_in_pause.rs) — Asserts that a playthrough is paused with a specific pause reason (e.g., Revive or OutOfEnergy).
- [`src/checks/playthrough_is_paused.rs`](src/checks/playthrough_is_paused.rs) — Asserts that a playthrough is currently paused regardless of reason.
- [`src/checks/revive_is_allowed.rs`](src/checks/revive_is_allowed.rs) — Validates that a revive is allowed by checking the playthrough is in a Revive pause and the max revive count has not been reached.

### Views (`src/views/`)

- [`src/views/panels.rs`](src/views/panels.rs) — Defines the revive and energy panel views that expose current purchase price and ad availability to the client.
- [`src/views/wallet_widgets.rs`](src/views/wallet_widgets.rs) — Defines gem and coin widget views that expose the player's current wallet balances.
- [`src/views/cosmetic_widgets.rs`](src/views/cosmetic_widgets.rs) — Defines views for player skin, level skin, and movement trail widgets showing purchase/applied status and prices.
- [`src/views/power_up_widgets.rs`](src/views/power_up_widgets.rs) — Defines magnet and shield widget views exposing inventory amounts, upgrade levels, and current prices.
- [`src/views/daily_reward_widgets.rs`](src/views/daily_reward_widgets.rs) — Defines the daily reward view showing upcoming reward days, gem amounts, streak progress, and claim eligibility.
- [`src/views/energy_widget.rs`](src/views/energy_widget.rs) — Defines the energy widget view showing current energy, max energy, next change timestamp, and interval constants.
- [`src/views/level_widget.rs`](src/views/level_widget.rs) — Defines the level widget view exposing the player's current level.

### Energy (`src/energy/`)

- [`src/energy/calculation.rs`](src/energy/calculation.rs) — Core energy math: idle regeneration, active-play consumption with ceiling division, and depletion/regen-completion timestamp calculation.
- [`src/energy/purchase.rs`](src/energy/purchase.rs) — Handles adding energy to a player (via ad or purchase) and processing gem-based energy purchases with escalating prices.
- [`src/energy/scheduling.rs`](src/energy/scheduling.rs) — Schedules energy depletion callbacks, restores regeneration progress after a playthrough ends, and checks out-of-energy pause state.

### Shop (`src/shop/`)

- [`src/shop/cosmetic.rs`](src/shop/cosmetic.rs) — Handles purchasing cosmetic items (player skins, level skins, movement trails) with cheat detection for duplicate purchases.
- [`src/shop/power_up.rs`](src/shop/power_up.rs) — Handles purchasing magnet and shield power-ups with daily purchase tracking and escalating prices.
- [`src/shop/power_up_upgrade.rs`](src/shop/power_up_upgrade.rs) — Handles purchasing magnet and shield upgrades (range, duration, spawn chance, collisions) with max-level enforcement.

### Hooks (`src/hooks/`)

- [`src/hooks/after_player_insert.rs`](src/hooks/after_player_insert.rs) — Post-insert hook that initializes all related data for a new player: wallet, energy, leaderboard entry, power-up data, and all cosmetic variant rows.

### Scheduled Functions (`src/scheduled_functions/`)

- [`src/scheduled_functions/energy_depletion.rs`](src/scheduled_functions/energy_depletion.rs) — Defines the energy depletion schedule table and the reducer that sets energy to zero when the depletion timer fires.
- [`src/scheduled_functions/check_player_names.rs`](src/scheduled_functions/check_player_names.rs) — Defines the periodic player name moderation job (currently a stub awaiting implementation).
- [`src/scheduled_functions/refresh_google_play_android_developer_api_access_token.rs`](src/scheduled_functions/refresh_google_play_android_developer_api_access_token.rs) — Periodically refreshes the Google Play API access token by calling an external token service and scheduling the next refresh before expiry.

### Procedures (`src/procedures/`)

- [`src/procedures/handle_in_app_purchase.rs`](src/procedures/handle_in_app_purchase.rs) — Validates and processes real-money in-app purchases by verifying tokens with the Google Play API and crediting gems to the player's wallet.

## Honorable Features

### Energy System

The energy system governs how long players can play before needing to wait or spend
resources. Players start with 60 energy (the maximum). During active gameplay, one energy
unit is consumed every 29.28 seconds (one-third of a level, timed to the in-game
soundtrack). When not playing, energy regenerates at a rate of one unit every 3 minutes
until the maximum of 60 is reached again. Players can also gain 15 energy by watching an
ad or spending gems, with escalating gem costs for repeated purchases within a single
playthrough.

The core design principle is **on-demand calculation**: the server does not tick energy up
or down on a timer. Instead, the [`Energy`](src/tables/energy.rs#L20) table stores the
player's last known energy value alongside two key timestamps —
`last_energy_calculation_at` and `last_energy_regeneration_at` — plus an
`energy_boundary_reached_at` field indicating whether energy is currently increasing or
decreasing and when it will hit its boundary (zero or max). Whenever a reducer or view
needs the current energy, it calls one of two pure calculation functions.
[`recalculate_energy_for_idle`](src/energy/calculation.rs#L12) computes how many 3-minute
regeneration ticks have elapsed since the last regeneration timestamp and adds them to
the stored value, capping at 60.
[`calculate_energy_during_active_play`](src/energy/calculation.rs#L49) uses ceiling
division on the elapsed time since the last calculation to determine how many consumption
ticks have passed, subtracting them from the stored value and flooring at zero. Both
functions then update the relevant timestamps so subsequent calls remain correct.

The only scheduled reducer in the entire energy system is
[`on_energy_depleted`](src/scheduled_functions/energy_depletion.rs#L23), which fires
exactly once per playthrough at the pre-calculated moment when energy will reach zero.
When a playthrough begins, the server computes the precise future timestamp at which
energy will deplete — `(energy - 1) * 29.28s + 1μs` from now — and inserts a single row
into the [`EnergyDepletionSchedule`](src/scheduled_functions/energy_depletion.rs#L8)
table. That scheduled reducer sets energy to zero and clears the boundary timestamp. If
the player pauses, the depletion schedule
is deleted and energy is recalculated up to the pause moment. On resume, a new depletion
schedule is inserted based on the remaining energy. When the playthrough ends, the
schedule is also deleted and regeneration progress is restored by backdating the
regeneration timestamp to preserve any fractional progress from before the play session
started.

This architecture was chosen specifically to avoid the naive alternative: running one
scheduled reducer per actively playing player every 29.28 seconds to tick energy down,
and one scheduled reducer per sub-max-energy player every 3 minutes to tick energy up.
For a game with many concurrent players, the naive approach would require thousands or
tens of thousands of scheduled reducer invocations per minute, each performing a database
read and write to adjust a single integer. That pattern does not scale — it creates
per-player recurring computation cost regardless of whether anyone is observing the
energy value, and it imposes continuous storage churn from writing updated rows on every
tick. The on-demand approach reduces the cost to zero when no one is looking: energy is
only ever recomputed when a reducer genuinely needs the current value (begin/pause/
resume/end playthrough, purchase energy) or when a view is queried by the client (which
happens when they are in the main menu).

The [`energy_widget`](src/views/energy_widget.rs#L26) view demonstrates this pattern on
the read path. When the client
subscribes to the view, it computes the current energy by calling the appropriate
calculation function depending on player state (idle, paused, or actively playing), then
returns the current value along with the timestamp of the next energy change and both
interval constants. The client can then run its own local countdown or countup timer
using these values, without needing to poll the server. This pushes the UI update cost
entirely to the client while the server remains stateless between events.

The trade-off of this on-demand approach is tight coupling: the
[begin](src/reducers/begin_playthrough.rs),
[pause](src/reducers/pause_playthrough.rs),
[continue](src/reducers/continue_playthrough.rs), and
[end](src/reducers/end_playthrough.rs) playthrough reducers all contain explicit energy
recalculation, schedule management, and boundary-timestamp bookkeeping. The naive
tick-based alternative would be simpler in this regard — those reducers would not need to
know about energy at all, since a background timer would keep the value current — but
that simplicity comes at a far higher runtime cost, as discussed above. If this project
were to be used, the energy logic interwoven into playthrough reducers should ideally be
extracted behind SpacetimeDSL hooks (e.g. `after_playthrough_insert`,
`after_pause_insert`, `after_playthrough_update`) so that the playthrough lifecycle
remains decoupled from the energy system and the energy bookkeeping is triggered
automatically rather than scattered across multiple reducer implementations.

### Cheat Detection & Logging

The server guards every player-facing reducer against illegitimate requests through a
cheat detection and logging system paired with a player banning mechanism.

The core design principle is **silent logging**: when a client sends an impossible request
— buying an already-purchased cosmetic, upgrading past the maximum level, claiming an
unavailable ad reward, or acting on a playthrough that is in the wrong state — the server
records the violation in a private, append-only
[`CheatAttemptLog`](src/tables/cheat_attempt_log.rs#L6) table but still returns a
successful response to the client. This prevents cheating clients from learning *why*
their action was rejected and iterating toward a working exploit, while giving
administrators a complete audit trail of every attempt. All cheat logging flows through a
[single entry point](src/cheat_attempt_log.rs#L20) that writes the log row and emits a
server warning, and an [`or_ok_on_cheat!`](src/cheat_attempt_log.rs#L41) macro lets
reducers distinguish a logged cheat from a real infrastructure error with minimal
boilerplate — cheats silently succeed (from the client's perspective), while real errors
propagate normally.

Validation is spread across the codebase rather than centralized in one place. The
[`checks`](src/checks/) module contains reusable predicates — is the playthrough active,
is it paused, is a revive allowed, and so on — and each returns through the cheat-or-error
path so that failures are automatically logged. Shop modules for power-ups, upgrades, and
cosmetics perform their own domain-specific validation (duplicate purchase detection,
max-level enforcement, daily purchase limits) using the same logging path. This means
every reducer that a player can call already carries its own cheat guard; there is no
separate validation layer to keep in sync.

On top of per-request detection, the server supports **player banning**. Administrators
can [ban](src/admin/ban_player.rs#L9) or [unban](src/admin/unban_player.rs#L11) players
through privileged admin reducers. The ban check is enforced at two choke points: on
[client connection](src/reducers/connect_client.rs#L17) (preventing banned players from
establishing a session) and on
[player authentication](src/authenticated_player.rs#L7) (rejecting any reducer call from
a banned player). Together, the append-only cheat log gives administrators visibility into
suspicious behavior, and the banning mechanism gives them the ability to act on it.

### Player Name Moderation

The server implements a deferred moderation pipeline for player-chosen names. When a
player renames themselves, the server first checks if the requested name has already been
moderated. If the name was previously approved or is still pending review, the rename
proceeds immediately. If the name was previously rejected, the request is denied outright.
A name that has never been seen before is inserted into the
[`PlayerName`](src/tables/player_name.rs#L6) table with a `Pending` status and the rename
is allowed optimistically — the player sees their new name right away while it awaits
review. Names must be between 3 and 20 characters.

The actual moderation decisions would be made by a (not implemented) scheduled procedure,
[`check_player_names`](src/scheduled_functions/check_player_names.rs#L17), which runs on
an hourly interval starting from database initialization. Each invocation would collect all
names still in `Pending` status and send them as a bulk request to an external moderation
service. The service's verdicts would be written back as `Approved` or `Rejected`, and because
names are keyed by their string value, any future player attempting to use an
already-rejected name would be blocked instantly without another round-trip.

The core design principle is **optimistic-then-enforce**: players are never blocked waiting
for a moderation response, but inappropriate names are caught and rejected within the next
hourly cycle. This avoids the latency and complexity of synchronous per-rename moderation
calls while still ensuring that rejected names are permanently recorded and cannot be
reused by any player.

### In App Purchase Verification

The server supports real-money gem purchases through the Google Play Billing system. Players
can buy gems in four tiers corresponding to fixed price points on the store. When a player
completes a purchase on the client, the client sends the purchase token to the server for
verification. The server never trusts the client's claim about what was bought; the token is the
only input, and everything else is derived server-side.

Verification is handled by a dedicated procedure, [`handle_in_app_purchase`](src/procedures/handle_in_app_purchase.rs#L49), which calls the Google Play Android Developer API to confirm that
the token represents a real, completed purchase. The API response provides the product ID, purchase
state, and region code. If the API confirms the purchase state is `PURCHASED`, the server maps the
product ID to the corresponding gem tier, credits the gems to the player's wallet, and marks the
purchase row as completed. If verification fails for any reason — network error, non-purchased
state, or an unexpected API response — the pending row is cleaned up and no gems are awarded.

To authenticate with the Google Play API, the server maintains a valid OAuth access token
through a self-scheduling refresh cycle. A scheduled procedure,
[`refresh_google_play_android_developer_api_access_token`](src/scheduled_functions/refresh_google_play_android_developer_api_access_token.rs#L39), calls an external token service
(not included in this repository) to obtain a fresh access token. Each successful refresh schedules
the next one to run five minutes before the token expires, ensuring the server
always has a valid credential ready without manual intervention. The refresh cycle is
bootstrapped at database initialization, where the first refresh is scheduled to run
immediately.

Duplicate purchase tokens are guarded against by a unique constraint on the token column in
the [`InAppPurchase`](src/tables/in_app_purchase.rs#L31) table. If a player submits a token
that already exists, the server logs a cheat attempt and silently returns success — consistent
with the cheat detection philosophy used elsewhere in the codebase. Every purchase is
persisted with its price tier, region code, and status, providing a complete audit trail of
all real-money transactions.

The core design principle is **reserve-then-verify**: the server inserts a `Pending` row
before making the external API call, ensuring that the token is claimed even if a concurrent
request arrives with the same token. Only after successful verification is the row promoted
to `Completed` and gems credited. This two-phase approach prevents double-spend scenarios
without requiring distributed locks or external deduplication infrastructure.

## 📜 Licensing

This repository is licensed under:

- ⚖️ [Unlicense](https://choosealicense.com/licenses/unlicense/)

**Open Source** ❤️
