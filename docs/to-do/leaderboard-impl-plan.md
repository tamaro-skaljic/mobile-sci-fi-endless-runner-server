# Implementation Plan: High-Score Logic & Leaderboard View

## Decisions Made (Q&A Summary)

| Question | Decision |
| --- | --- |
| `end_playthrough` leaderboard update fields | Only `score` |
| Case 3 return value | Top 25 **plus** surrounding 25 appended |
| Player with no leaderboard entry | Never happens (created in `after_player_insert`) |
| Duplicate player entry in result | Deduplicate — player appears exactly once |
| Fewer than 12 surrounding entries | Include however many exist |
| Return type | New `LeaderboardPanel` struct with `entries: Vec<LeaderboardEntryWidget>` and `player_name: String` |
| Score sorting approach | Add `#[index(btree)]` on `score` for query builder range queries; sort in Rust |
| `player_id` constraint | Change `#[index(btree)]` → `#[unique]` |
| View table access style | `ctx.from` query builder for all table access |
| New file location | `src/views/leaderboard_panel.rs` |
| View accessor name | `leaderboard_panel` |

---

## Files to Change

1. `src/tables/leaderboard_entry.rs` — schema changes
2. `src/reducers/end_playthrough.rs` — remove `is_high_score` param, add leaderboard update logic
3. `src/views/leaderboard_panel.rs` — new file with view and widget structs
4. `src/lib.rs` — register the new view module

---

## Step 1 — `src/tables/leaderboard_entry.rs`

### 1a. Change `player_id` from `#[index(btree)]` to `#[unique]`

The guarantee that there is exactly one leaderboard entry per player should be enforced at the schema level. `#[unique]` generates a single-row `get_leaderboard_entry_by_player_id(&player)?` DSL method instead of an iterator.

`#[use_wrapper(super::player::PlayerId)]` stays as-is.

Before:

```rust
#[index(btree)]
#[use_wrapper(super::player::PlayerId)]
#[foreign_key(...)]
player_id: Identity,
```

After:

```rust
#[unique]
#[use_wrapper(super::player::PlayerId)]
#[foreign_key(...)]
player_id: Identity,
```

### 1b. Add `#[index(btree)]` and `#[create_wrapper]` on `score`

The btree index lets the `ctx.from` query builder run efficient range queries (`score.gt(x)`, `score.lt(x)`) for finding surrounding entries. Per the SpacetimeDSL requirement, every `#[index]` column needs a wrapper.

`#[create_wrapper]` with no custom name generates `LeaderboardEntryScore(u64)`.

Before:

```rust
pub score: u64,
```

After:

```rust
#[index(btree)]
#[create_wrapper]
pub score: u64,
```

---

## Step 2 — `src/reducers/end_playthrough.rs`

### 2a. Remove `is_high_score: bool` parameter

The server now computes this from the leaderboard entry rather than trusting the client.

### 2b. Add leaderboard update logic

After computing `score` (the local variable already passed into the reducer), before setting playthrough fields:

1. Fetch leaderboard entry: `let mut leaderboard_entry = dsl.get_leaderboard_entry_by_player_id(&player)?;`
2. Compare: `let is_high_score = score > *leaderboard_entry.get_score();`
3. If `is_high_score`:
   - `leaderboard_entry.set_score(score);`
   - `dsl.update_leaderboard_entry_by_player_id(leaderboard_entry)?;`
4. `playthrough.set_is_high_score(is_high_score);`

### 2c. Update imports

Add to the `use crate::tables::leaderboard_entry::` import:

- `GetLeaderboardEntryRowByPlayerId`
- `UpdateLeaderboardEntryRowByPlayerId`

---

## Step 3 — `src/views/leaderboard_panel.rs` (new file)

### 3a. Define widget structs

```rust
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct LeaderboardEntryWidget {
    pub score: u64,
    pub player_name: String,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct LeaderboardPanel {
    pub entries: Vec<LeaderboardEntryWidget>,
    pub player_name: String,
}
```

### 3b. View function skeleton

```rust
#[spacetimedb::view(accessor = leaderboard_panel, public)]
pub fn leaderboard_panel(ctx: &ViewContext) -> Option<LeaderboardPanel> {
    // ... see logic below
}
```

### 3c. View logic (step by step)

**Step 1 — Fetch player and their name**

```rust
let player = ctx.from.player()
    .r#where(|p| p.id.eq(ctx.sender()))
    .next()?;
let player_name = player.name.clone();
```

**Step 2 — Fetch calling player's leaderboard entry (always exists):**

```rust
let player_entry = ctx.from.leaderboard_entry()
    .r#where(|e| e.player_id.eq(ctx.sender()))
    .next()?;
let player_score = player_entry.score;
let player_is_public = player_entry.is_public;
```

**Step 3 — Fetch and sort top 25 public entries:**

```rust
let mut all_public: Vec<LeaderboardEntry> = ctx.from.leaderboard_entry()
    .r#where(|e| e.is_public.eq(true))
    .collect();
all_public.sort_by(|a, b| b.score.cmp(&a.score));
let top_25: Vec<LeaderboardEntry> = all_public.into_iter().take(25).collect();
```

**Case 1 — Player is in the top 25:**

The player is `is_public = true` and their score is high enough:

```rust
if top_25.iter().any(|e| *e.get_player_id() == ctx.sender()) {
    let entries = to_widgets(ctx, &top_25);
    return Some(LeaderboardPanel { entries, player_name });
}
```

**Case 2 — Player is `is_public = false` and beats the 25th place**

Only applies when top 25 has exactly 25 entries (i.e., there are at least 25 public players):

```rust
if !player_is_public {
    let beats_twenty_fifth = top_25
        .get(24)
        .is_some_and(|e| player_score > e.score);

    if beats_twenty_fifth {
        let insert_pos = top_25
            .iter()
            .position(|e| player_score > e.score)
            .unwrap_or(top_25.len());
        let mut result = top_25;
        result.insert(insert_pos, player_entry.clone());
        result.truncate(25);
        let entries = to_widgets(ctx, &result);
        return Some(LeaderboardPanel { entries, player_name });
    }
}
```

**Case 3 — Player not shown in top 25 (and is_public = false with score ≤ 25th):**

Get 12 closest entries above and below the player (both `is_public = true`), deduplicate against top 25, append to result:

```rust
// Top-25 identity set for deduplication
let top_25_ids: HashSet<u64> = top_25.iter()
    .map(|e| e.get_id().value())
    .collect();

// 12 entries closest above player (is_public = true, score > player_score)
let mut higher: Vec<LeaderboardEntry> = ctx.from.leaderboard_entry()
    .r#where(|e| e.is_public.eq(true).and(e.score.gt(player_score)))
    .collect();
higher.sort_by(|a, b| a.score.cmp(&b.score)); // ascending = closest first
higher.truncate(12);
higher.reverse(); // descending for display

// 12 entries closest below player (is_public = true, score < player_score)
let mut lower: Vec<LeaderboardEntry> = ctx.from.leaderboard_entry()
    .r#where(|e| e.is_public.eq(true).and(e.score.lt(player_score)))
    .collect();
lower.sort_by(|a, b| b.score.cmp(&a.score)); // descending = closest first
lower.truncate(12);

// Build surrounding section, skipping entries already in top 25
let mut surrounding: Vec<LeaderboardEntry> = Vec::new();
for entry in higher {
    if !top_25_ids.contains(&entry.get_id().value()) {
        surrounding.push(entry);
    }
}
surrounding.push(player_entry);
// Lower entries are always below the 25th place score, so no deduplication needed
surrounding.extend(lower);

let mut result = top_25;
result.extend(surrounding);

let entries = to_widgets(ctx, &result);
Some(LeaderboardPanel { entries, player_name })
```

### 3d. Helper: `to_widgets`

Converts a slice of `LeaderboardEntry` rows into `Vec<LeaderboardEntryWidget>` by looking up each player's name:

```rust
fn to_widgets(ctx: &ViewContext, entries: &[LeaderboardEntry]) -> Vec<LeaderboardEntryWidget> {
    entries.iter().map(|entry| entry_to_widget(ctx, entry)).collect()
}

fn entry_to_widget(ctx: &ViewContext, entry: &LeaderboardEntry) -> LeaderboardEntryWidget {
    let player_name = ctx.from.player()
        .r#where(|p| p.id.eq(*entry.get_player_id()))
        .next()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    LeaderboardEntryWidget {
        score: entry.score,
        player_name,
    }
}
```

### 3e. Imports needed

```rust
use std::collections::HashSet;
use spacetimedb::{SpacetimeType, ViewContext};
use crate::tables::leaderboard_entry::LeaderboardEntry;
```

---

## Step 4 — `src/lib.rs`

Add the new module under the `views` block:

```rust
pub mod views {
    // ... existing
    pub mod leaderboard_panel;
}
```

Also remove the `leaderboard_entry` mention from the FIXME comment if the leaderboard functions are now fully covered.

---

## Edge Cases Confirmed

| Scenario | Behaviour |
| --- | --- |
| Player has no leaderboard entry | Impossible — `after_player_insert` always creates one |
| Fewer than 25 public entries | `top_25` holds however many exist; Case 2 guard checks `top_25.get(24)` |
| Fewer than 12 above/below | `truncate(12)` is a no-op if fewer exist; no padding |
| Player in top 25 but is_public = false | Cannot happen — `top_25` only contains `is_public = true` entries |
| Duplicate player entry in surrounding | Player is excluded from higher/lower (filters use `score.gt`/`score.lt`, not `≥`/`≤`); player entry pushed explicitly |
| Higher entries overlapping with top 25 | Deduplicated using `top_25_ids` HashSet |
| Lower entries overlapping with top 25 | Impossible — lower scores cannot be in a top-25-by-score list |

---

## Notes on SpacetimeDSL Generated Methods (after schema change)

After changing `player_id` to `#[unique]`:

- Old (index): `dsl.get_leaderboard_entries_by_player_id(&player)` → iterator
- New (unique): `dsl.get_leaderboard_entry_by_player_id(&player)?` → single `Result<LeaderboardEntry, SpacetimeDSLError>`

After adding `#[index(btree)]` + `#[create_wrapper]` on `score`:

- New wrapper type: `LeaderboardEntryScore(u64)`
- New DSL method: `dsl.get_leaderboard_entries_by_score(score)` → iterator (unused; btree index is used by the view query builder)
- Setter `set_score()` will now accept `impl Into<LeaderboardEntryScore>` (both `u64` and `LeaderboardEntryScore` work)
