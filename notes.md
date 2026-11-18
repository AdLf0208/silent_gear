# Silent Gear — port notes (slice 1: gear crafting + tool stats)

Port of **Silent Gear 4.2.1.1** to Horizon, now as **one flat SDK package** (`api = "0.2"`,
version **0.2.0**): Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18),
horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78. The retired
`.hnmod` bundle (0.1.0, campaign pin `1d63e0a5`) and its closed submission **#851** are superseded
— see *Migration to the SDK format* below for the file map, the hook table and the changelog.

Ported from **Silent Gear 4.2.1.1** (`silent-gear-1.21.1-neoforge-4.2.1.1.jar`,
SHA-1 `928064b09748d6e963077c114d65f5ce79aff581`, the exact ATM10 jar) for
Minecraft Java 1.21.1 / NeoForge. **MIT → licence rank 0**; local-only until
publishing is approved (ARCHITECTURE.md §8).

Status: **ported-partial** — slice 1 (the pickaxe's material stat block, its
assembly and the vanilla mining it drives) is implemented, and its pure model and glue are green
on the SDK ladder. Its **in-game leg has not been re-run for 0.2.0**: `horizon-launcher` is not
installed here, so no game built at this SDK's pin exists (gap 0 below). The other tools/armour,
the other materials as functional gear, traits, grading, salvage, compound parts/recipe types,
the blueprint book and the gear-slot system are the gap list below.

**Why this mod now:** the rank-0 deep mechanics slice the campaign queue
dispatched as wave 6 (`ports/_shared/queue/repin-wave-resume.md`, `sg-1d63e0a`),
after the machines waves. `precheck-mod.py silentgear` →
`DISPATCHABLE`, jar licence MIT (rank 0).

## Class + tier and the approach

- **Class:** `content`; **tier:** `deep` (source-map row and ledger).
- **Approach:** per `horizon/docs/modpacks.md` a deep gear mod is an **agent
  port**: carried content (items, models, textures, lang, one vanilla recipe)
  plus native Rust logic (the material property fold, the assembly, the
  component). This slice is not a data-only carry and builds no entity,
  renderer or dimension feature.

## Behaviour implemented (slice 1)

- **One gear: `silentgear:pickaxe`**, assembled by
  `/silentgear assemble pickaxe <head> <rod>` from a carried
  `silentgear:pickaxe_blueprint`, `silentgear:pickaxe_head` and
  `silentgear:rod` (one of each from the main inventory, then the off hand; the
  blueprint, head and rod are consumed).
- **Material stat block.** The material table is the jar's own data
  (`data/silentgear/silentgear_materials/*.json`) for iron, wood, stone,
  diamond, gold, flint (and the empty netherite entry). The runtime folds each
  part's properties with Silent Gear's `NumberProperty.compute`: weighted
  `AVERAGE`, `MAX`, `MULTIPLY_BASE`, `MULTIPLY_TOTAL`, `ADD`, clamped to
  `0..i32::MAX`, durability rounded. Iron head + wood rod = **250 / 6.0 / 14 /
  iron** — exactly the jar's numbers and exactly vanilla's iron pickaxe
  components in 26.3.
- **The stat block lives on the item.** The assembled stack carries a
  `silentgear:gear` component (registered before content loads) with the gear
  type, the parts and their materials, and the stats; `/silentgear stats` and
  the item tooltip read it back. `silentgear:pickaxe` is declared from the
  `minecraft:iron_pickaxe` template, so vanilla's own `minecraft:tool` rule
  (`#minecraft:mineable/pickaxe`, speed 6, correct for drops), `max_damage`
  250, `enchantable` 14 and `repairable` are the iron stat block written as
  item components (`docs/mods.md`: "that is how … a tool its mining speed").
- **Vanilla mining drives it:** stone breaks in 8 ticks with the pickaxe and
  drops cobblestone, and the held stack loses one durability per block; a
  stick in the same budget breaks nothing (16-tick timeout) and drops nothing.
- **Refusals, not lies.** The command refuses a combination whose folded block
  is not the item's fixed components (e.g. a diamond head: 1561 / 8 / 10), a
  wrong part (`silentgear:axe_head`), a missing blueprint/head/rod, an unknown
  material, or another gear type. It never hands out an item whose component
  claims one stat while vanilla breaks/mines it with another.

## Hooks (0.1.0, retired `.hnmod` — the SDK mapping is in *Migration to the SDK format*)

| Hook (API) | Helper | What it does |
|---|---|---|
| `component_types` (65) | `component_types.rs` | registers `silentgear:gear` before content loads |
| `register_commands` (8) | `register.rs` | `/silentgear assemble|stats|help` at op level 2 |
| `perform_command` (8) | `perform.rs` | the command body, in the acting player's state |
| `tooltip` (12) | `tooltip.rs` | the gear's stat line in its tooltip |

No pending/unstamped hook is named; see the pin gap below.

## The pin's limitation this slice had to respect (still true at the SDK: gap 1)

Upstream sets `Tool`, `getMaxDamage` and `ENCHANTABLE` **per stack** when a gear
is assembled (`GearDiggerTool.createToolProperties`, `BasicGearItem.getMaxDamage`),
and NeoForge 1.21.1 reads them per stack. Horizon at `1d63e0a` reads the
**item's defaults**:

| What | Pin file:line |
|---|---|
| mining speed | `crates/horizon-items/src/items.rs:1135` (`Items::mining_speed` → `ItemInfo.tool`) |
| correct-for-drops | `crates/horizon-items/src/items.rs:1156` |
| wear per block / breaking point | `crates/horizon-client/src/game.rs:6531`, `:14312-14314` |
| HUD bar denominator | `crates/horizon-client/src/game.rs:9118` |
| the faithful per-stack hook (`BreakSpeed`), **pending** | impl `crates/horizon-client/src/game/native_hooks.rs:2768`, called at `game/hand.rs:421`; fragment `tools/native-hook-fragments/break_speed.json` — **not in `native-hooks.json`**, so `E_HOOK_UNKNOWN` |

So a per-stack stat block cannot drive vanilla mining at this pin, and
`native_set_component` only accepts custom (`silentgear:`) component ids
(`crates/horizon-client/src/game/native_components.rs:48`). Slice 1 therefore
builds the one combination the item components can honour (iron head + wood
rod) and keeps the general fold tested for iron/stone/diamond/gold. Closing
this needs the upstream `break_speed` stamp plus engine reads of the stack's
`minecraft:tool`/`max_damage` — recorded as the first gap, not worked around
with a raw mixin.

## Content (targeted carry, no invented assets)

Full carry: `ports/_shared/carry-work/silentgear/mods/silentgear/carry`
(2952 files, 314 item definitions, 63 blocks — the whole mod). The slice's
`content/` keeps only what it uses; `content/PATCHES.md` lists every file and
every patch:

- item definitions and assets for `pickaxe`, `pickaxe_head`, `rod`,
  `pickaxe_blueprint`, `axe_head` (models, textures, 26.3 `items/` files);
  `pickaxe`'s definition is patched from the carry's `minecraft:stick` guess to
  the iron stat block, and the gear model from the unrenderable
  `builtin/entity`/`silentgear:gear` parent to a generated model built from
  upstream's own textures;
- the jar's `en_us.json` (plus a `item.silentgear.pickaxe_blueprint` key: the
  jar names blueprints with a `%s` code template);
- the vanilla blueprint recipe (`3 paper + 2 sticks`) upgraded by the carry,
  with `silentgear:blueprint_paper` and a `c:rods/wooden` tag that adds
  `minecraft:stick` (the jar's own convention tag lists only netherwood sticks);
- a `Silent Gear` creative tab with the five items.

## Embedded content (AE2 pattern) — 0.1.0, retired with `.hnmod`

The registry stores **one file per project** (playbook §1.0), so a port with both
halves could not be listed faithfully: an `.hnmod` is code-only and an `.hmod`
is content. This bundle now carries its content, exactly as the merged AE2 entry
does and as `charginggadgets` first proved here:

- **`mixins/silentgear/data/silentgear-content.hmod`** is the built
  `content/silentgear.hmod`, byte for byte (24,569 B,
  `ec9806d9…df729`). `tools/package.py` packs `data/**` as
  `mixins/<id>/data/…` (top-level `.rs`/`.toml` plus everything under `data/`)
  with the same fixed zip timestamps, and it refuses a
  `data/<id>-content.hmod` that is not byte-identical to `content/<id>.hmod`, so
  a rebuilt content mod can never ship stale inside the bundle. Two consecutive
  packaging runs gave the same SHA-256.
- **`mixins/silentgear/embedded_content.rs`** with the `embedded-content`
  `[[injection]]` (`target = "horizon_client::game::mods::content_root"`,
  `kind = "HEAD"`, `cancellable = true`, bindings `root`/`enabled`/`dev`)
  compiles the file in with `native_data!` (API 41), writes it to
  `.horizon-cache/silentgear-embedded-content.hmod` when the copy on disk
  differs, appends it to the dev-mod list and returns its `content_root`. The
  helper calls `content_root` once more and stops at its own
  `dev.contains(&file)` guard, so the second call runs the vanilla body — the
  same one-level recursion AE2 uses. An unwritable cache logs a warning and
  `Continue`s (no content) instead of failing the game.
- **`tools/package.sh`** builds the `.hmod` with the pin's own
  `cargo-horizon-mod`, syncs it into `data/`, runs `tools/package.py`, then
  checks the `.hnmod` with `horizon-mod check --kind native-mod`. The delivered
  `.hnmod` is written into the port root, never `target/`.
- `content/horizon-mod.toml` now names the **port's** version (`0.1.0`), per
  `AGENT_MOD_STRUCTURE.md` §6, so the one registry artifact and its embedded
  content agree.

Verified in game with **only the bundle** (`EMBEDDED=1`, no `--dev-mod`): the
whole scenario A–E passes, and the file the helper wrote is byte-identical to
`content/silentgear.hmod`. The control (`CONTROL=1`) launches the same binary
with `.horizon-cache` replaced by a plain file, so the helper refuses its
content: `give @s silentgear:pickaxe_blueprint` answers
`Unknown item 'silentgear:pickaxe_blueprint'` and no blueprint arrives — while
the native `/silentgear` command still answers (`carry a
silentgear:pickaxe_blueprint first.`), so the run distinguishes "no content"
from "no mod at all". The PASS is therefore not vacuous.

This makes the `.hnmod` the single listable artifact of this port; it does not
by itself publish it (licence approval is still required, ARCHITECTURE.md §8).

### In-game transcript (quoted from `e2e-embedded.log`)

```
== control: the same game with no embedded content file ==
  launch: no --dev-mod; the bundle carries its own content
  give: give @s silentgear:pickaxe_blueprint 1<--[HERE]|Unknown item 'silentgear:pickaxe_blueprint'|…
  native command: carry a silentgear:pickaxe_blueprint first.
  without the embedded file: no blueprint item; the native command still refuses (it is in the binary)
== launch (Overworld arena, content + native bundle) ==
  launch: no --dev-mod; the bundle carries its own content
  embedded content: 24569 B at /tmp/opencode/silentgear-home.CtDDXp/.horizon-cache/silentgear-embedded-content.hmod, identical to content/silentgear.hmod
== A. an unassembled blueprint builds nothing ==
  refusal: carry a silentgear:pickaxe_head first.
== B. blueprint + head + rod assemble the pickaxe ==
  Assembled silentgear:pickaxe from iron and wood: durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron
  stats: durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron
== C. the pickaxe mines stone: a drop, its speed, one durability ==
  break: {"block":"minecraft:air","block_before":"minecraft:stone","drops":{"minecraft:cobblestone":1},…,"ticks":8}
  held damage after one block: 1
== D. control: a stick in the same budget breaks nothing ==
  stick break: {"block":"minecraft:stone","error":"timed out before the block broke","pos":[1,101,2],"ticks":16}
== E. control: a wrong part is refused ==
  refusal: a silentgear:axe_head is not a silentgear:pickaxe_head.
PASS: …
EXIT:0
```

The same scenario with `SELFTEST=1 EMBEDDED=1` ends `FAIL` + `EXIT:1`
(`the pickaxe's damage is 1, expected 2`, `e2e-embedded-selftest.log`), after
the same embedded-content check — so the fail-capability leg still runs from the
one file.

## Verification (0.1.0 — historical; the 0.2.0 ladder is in *Migration to the SDK format*)

Every build/test/e2e ran through `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0`; no bare `cargo`, no `--target-dir`. Evidence:
`ports/_shared/evidence/silentgear/`.

| Step | Evidence | Result |
|---|---|---|
| runtime TDD (red) | `test-red.log` | **0 passed, 22 failed**, each `not yet implemented` |
| runtime TDD (green) | `test-green.log` | **21 passed**, 0 failed |
| fmt | `fmt.log`, `fmt-helper.log` | `cargo fmt --all -- --check` and `rustfmt --check` on the injected helper both exit 0 (empty) |
| clippy | `clippy.log` | `--all-targets --locked --offline -- -D warnings`, exit 0 |
| stage | `stage.log` | `stage_changed: true`; the plan now carries 5 injections (the 4 existing hooks + `embedded-content`) |
| check | `check.log`, `target/horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| embedded test | `test.log` | `test result: ok. **21 passed**; 0 failed; 1240 filtered` |
| fast build | `build.log` | `cargo_exit: 0`, 0 warnings |
| e2e (dev-mod) | `e2e.log` | **PASS**, `EXIT:0` |
| fail-capability (dev-mod) | `selftest.log` | `SELFTEST=1` → **EXIT:1** (`the pickaxe's damage is 1, expected 2`) |
| e2e, one file only (`EMBEDDED=1`, no `--dev-mod`) | `e2e-embedded.log` | **PASS**, `EXIT:0`; the control leg (same binary, `.horizon-cache` unusable) shows the ids unknown while the command still answers |
| embed fail-capability | `e2e-embedded-selftest.log` | `SELFTEST=1 EMBEDDED=1` → **EXIT:1** |
| log checker | `check-e2e-logs.py silentgear --strict` | `logs=3 ok=3 missing-exit=0 bad-exit=0` |
| `.hmod` build/check | `hmod-check.log`, `package.log` | 24,569 B, `ec9806d9…f729`, `no code: content only` |
| `.hnmod` (content embedded) | `package.log` | 34,940 B, `59e43dee…56b8`, 10 entries incl. `mixins/silentgear/data/silentgear-content.hmod` |
| `cargo horizon-mod check --kind native-mod` | `check-native.log` | exit 0, `native-mod … for 26.3` |
| load-check | `hmod-load.log` (previous slice) | **PASS** with `--dev-mod`, negative control registers none |

Reproduction: `tools/e2e-embed.sh` runs both embedded game legs in one lease
(control + scenario, then the selftest) and `tools/selftest-embed.sh` runs the
fail-capability leg alone; both go through `build-slot.sh` (`BUILD_LOCAL=1` when
the leg must stay on this box).

### In-game transcript (quoted from `e2e.log`)

```
  refusal: carry a silentgear:pickaxe_head first.
  Assembled silentgear:pickaxe from iron and wood: durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron
  stats: durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron
  held: {"components":"{\"silentgear:gear\":{durability:250,enchantment_value:14,gear_type:\"silentgear:pickaxe\",harvest_speed:6.0d,harvest_tier:\"iron\",parts:[{material:\"silentgear:iron\",part:\"silentgear:main\"},{material:\"silentgear:wood\",part:\"silentgear:rod\"}]}}","count":1,"item":"silentgear:pickaxe"}
  break: {"block":"minecraft:air","block_before":"minecraft:stone","drops":{"minecraft:cobblestone":1},...,"ticks":8}
  held damage after one block: 1
  stick break: {"block":"minecraft:stone","error":"timed out before the block broke","pos":[1,101,2],"ticks":16}
  refusal: a silentgear:axe_head is not a silentgear:pickaxe_head.
PASS: …
EXIT:0
```

Fail-capability: `SELFTEST=1` expects damage 2 instead of 1 and the run exits 1;
the stick phase is a same-run differential (nothing breaks, nothing drops) and
the two refusals are the negative controls.

### Packaged artifacts

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `silentgear.hnmod` (content embedded) | 34,940 B | `59e43deeb731f2f788046a9cdd731c554a228fc80a9294c7f244d2d45b4d56b8` | 10 |
| `content/silentgear.hmod` | 24,569 B | `ec9806d9eac596e656d5e0a0be2ea3e48c88a0006205b32876ffb7981a4df729` | 29 |

The `.hnmod` carries the content mod as
`mixins/silentgear/data/silentgear-content.hmod` (24,569 B, the same SHA-256),
so it is the single artifact a registry entry needs. Both were packaged twice
with identical SHA-256. The `.hmod` load-check stays fail-capable: the same
launch **without** `--dev-mod` registers none of the five ids (`hmod-load.log`).

### Campaign gates at this touch

- `python3 ports/tools/precheck-mod.py silentgear` → exit 3, **not a new
  dispatch**: `status=ported-partial`, the jar licence is MIT (rank 0) and the
  port directory exists; this is a continuation slice, which the orchestrator
  dispatched.
- `python3 ports/tools/check-pin-updates.py` → exit **2** (`PIN MOVED`):
  upstream is 4,490 commits ahead and the mixins pin also moved, catalog
  **+2 hooks** (`world_hold`, `world_warp`) **+3 helpers** (`force_chunk`,
  `force_chunk_in`, `forced_chunks`) — worldgen holds/warps, unrelated to this
  slice. Stayed on the brief's pin `1d63e0a`; a repin decision belongs to the
  orchestrator.
- `python3 ports/tools/check-e2e-logs.py silentgear --strict` → `logs=3 ok=3
  missing-exit=0 bad-exit=0`.
- `python3 ports/tools/check-skills.py --notes ports/silentgear` → `Skills line
  present`, gate PASS.
- `python3 ports/tools/upstream_ready.py silentgear` → **READY** (MIT), 0
  blocked of 1.

## Review repairs (PR #583, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-583-silentgear.md` (CHANGES_REQUESTED, 5 items).

| Reviewer's item | This job |
|---|---|
| [P1] artifact not staged | **orchestrator** (no write access) |
| **[P1] the packager omits `data/silentgear-content.hmod` and `package.sh` checks the stale root file** | **fixed**: the publish tree's `tools/package.py` is the port's recursive packager (it now refuses a bundle whose `data/` file is not the built content and writes the bundle it just produced), and `tools/package.sh` synced |
| [P2] `blueprint_paper` tag names only `silentgear:blueprint_paper`, which the archive never defines | **fixed**: the item is carried (item definition, 26.3 `items/` file, model, 16×16 texture, lang already present) with its jar recipe (4 paper + a blue dye → 4 paper), plus the `c:paper` and `c:dyes/blue` tags the recipe needs (26.3 ships no `c:` tags) |
| [P2] the "folded stats drive mining" claim is not implemented; assembly refuses everything but iron | **fixed as disclosure**: the description now states the hard limitation — the pin has no per-stack tool stats, so the pickaxe is always a vanilla iron pickaxe and the folded numbers are a readout; assembly refuses a combination whose folded stats differ rather than producing an item that cannot honour them |
| [P2] MIT notices missing from the archive | **fixed** by the packager |

Verified after the content change: `tools/e2e.sh` — **PASS + `EXIT:0`**: the unassembled
blueprint builds nothing, blueprint + head + rod gives one pickaxe with its folded component,
it breaks stone with its drop and takes 1 damage, a stick breaks nothing and an axe head is
refused (`ports/_shared/evidence/silentgear/review-repairs-e2e.log`). The new
`blueprint_paper` item and recipe are content, so the assembly legs stand on the same
`ctl give` path as before.

Artifact: `silentgear.hnmod` **37,360 B** `474a5fd4a686c5b7111317c59109b465fbf98c215bb423fee70622e927cc0516`
(content `silentgear.hmod`).

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired the `.hnmod` container, and
the registry closed every `.hnmod` submission on 2026-11-18 with one reason: *"republish this as a
native `.hmod` with the SDK from the next Nightly"*. This port's submission, **PR #851**
(`silentgear` 0.1.0), was closed by that purge. **0.2.0** is the same slice republished as **one
flat SDK `.hmod`** — content, native code and the manifest in one archive. Recipe followed:
`ports/imfast/notes.md` → *Migration to the SDK format*, then `ports/enderstorage/notes.md` (the
closest analogue: a custom item component + commands) and `ports/framedblocks/notes.md`
(`Registrar` declarations and the 0.2 ladder discipline).

Pins: **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18, `package_format` 6,
`content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (moved from
`5a64af93…`), launcher 0.2.78. The old campaign pin `1d63e0a` does not apply to a package build.
The SDK is cached in-tree at `../_shared/sdk/0b14f8a/` and copied into `.horizon/sdk/`
(gitignored) for offline builds; it is never packed.

**Done in this order:** the retired layout was moved into
`../_shared/evidence/silentgear/retired-hnmod-0.1.0/` **before** the first build (the tool refuses
a root holding retired package paths), `src/runtime.rs` was copied with `cp` and proved with `cmp`
+ `sha256sum`, `cargo test` ran on the new crate with **no glue at all** (21 passed), and only then
was the SDK glue written — its own tests first on `todo!()` (red), then implemented (green).

### File map

| 0.1.0 (retired `.hnmod`) | 0.2.0 (flat SDK package) | Evidence / note |
|---|---|---|
| `mixins/silentgear/runtime.rs` (1,020 lines, also the crate lib) | `src/runtime.rs` — **byte-identical**, `pub mod runtime;` | `cmp` clean, sha256 `b65ff9249c14766df4f48d9c27e04642cba8b85aa88464bbe1e0c436d46d9f1c`; the packaged copy has the same hash |
| `mixins/silentgear/support.rs` (the `silentgear_state` module) + 5 hook helpers | `src/lib.rs`: one `impl Hooks`, one `ItemPart`, the component definition and the glue | helpers are gone; their bodies are functions in `src/lib.rs` |
| `mixins/silentgear/horizon-mixin.toml` (2 `[[module]]`, 5 `[[injection]]`) | no plan: `horizon-mod.toml` + typed `Hooks`/Part methods | `retired-hnmod-0.1.0/mixins/` |
| `mixins/silentgear/data/silentgear-content.hmod` + `embedded_content.rs` (AE2 `content_root` HEAD write-out to `.horizon-cache`) | **gone** | one package carries `assets/` + `data/`; nothing is embedded, written or appended to the dev list |
| `content/horizon-mod.toml` | merged into the root `horizon-mod.toml` (`id/name/version/description/authors/api/side/capabilities` + `[native]`) | `retired-hnmod-0.1.0/content-old/` |
| `content/assets/` (22 files) → `assets/` | package root, **byte-identical** | `diff -r` against the retired copy: identical |
| `content/data/` (13 files) → `data/` | package root, **byte-identical** | same `diff -r` |
| `content/PATCHES.md` | `PATCHES.md` at the package root | content provenance, kept |
| `content/silentgear.hmod` (24,569 B, 29 entries, content only) | not needed: content is inside the one package | `retired-hnmod-0.1.0/content-old/` |
| `silentgear.hnmod` (37,360 B, content embedded) | `silentgear.hmod` (**50,986 B**, 43 entries, native + content) | `retired-hnmod-0.1.0/silentgear.hnmod` |
| `compatibility.json`, `rust-toolchain.toml` | `[native] game/horizon/mixins` in the manifest; the pinned commit decides the toolchain | `retired-hnmod-0.1.0/` |
| `Cargo.toml` (`[lib] path = "mixins/…"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`[lib] src/lib.rs`, one dependency: `horizon-mod-api` from `.horizon/sdk/`) | — |
| `tools/horizon.py`, `package.py`, `package.sh`, `notice.py`, `test_stage.py`, `test_template.py`, `audit/`, `hmod-load.sh`, `e2e-embed.sh`, `selftest-embed.sh`, `ladder-embed.sh` | **gone**: staging, `.hnmod` packaging, the notice injector, the plan tests and the embedded-content legs belong to the retired format | `retired-hnmod-0.1.0/tools/` |
| `AGENTS.md`, `PLAN.md` (the port's own: they instruct reading `compatibility.json` and running `tools/horizon.py`, and forbid a bare `cargo` — which the SDK ladder requires) | **retired** | `retired-hnmod-0.1.0/` |
| `Cargo.lock` | not kept: it is packed and changes the artifact | not kept (trap 5) |

**Kept and rewritten:** `README.md` (api 0.2, the SDK ladder, the missing in-game leg),
`tools/e2e.sh` (same A–E scenario, now a `GAME=` launcher-built game with `CONTROL_GAME=` and the
`SELFTEST=1` leg; the `EMBEDDED`/`.horizon-cache` legs are gone), `notes.md`, `VALIDATION.md`,
`FINDINGS.md`, `LICENSE`, `PATCHES.md`. **New:** `horizon-mod.toml`, `Cargo.toml`, `.gitignore`,
`src/lib.rs`, `LICENSE-NOTICE.md`, `silentgear.hmod`.

### Helper → hook table

| 0.1.0 injection (hook, API) | 0.2.0 |
|---|---|
| `[[module]] runtime` (the pure model, injected as `crate::game::silentgear`) | `src/runtime.rs` as an ordinary module (`pub mod runtime;`) — no injection, no planner |
| `[[module]] state` (`support.rs`, injected as `crate::game::silentgear_state`) | merged into `src/lib.rs`; the state was only functions, so no second module is needed |
| `embedded-content` (`content_root` HEAD, AE2 pattern) | **gone** — the flat `.hmod` carries its own `assets/`/`data/` |
| `component-types` (`component_types`, 65) | `Hooks::component_types` (`ComponentTypesEvent.types`) pushes `silentgear:gear` — same registration, before content loads |
| `register-commands` (`register_commands`, 8) | `Hooks::register_commands` pushes the same three usages with `Command::op(usage, 2)` |
| `perform-command` (`perform_command`, 8; raw line, `&mut bool ran`) | `Hooks::run_mod_command` returns `Flow`; the line is a typed `CommandInvocation` (`literals` + `arguments`), so `action_of` rebuilds the runtime's `Action` from named words instead of splitting text |
| `gear-tooltip` (`tooltip`, 12; a global `Hooks` hook in 0.1.0) | **`ItemPart::tooltip` on the pickaxe** — at 0.2 `tooltip` is a *scoped* hook (`scope = "item"`), not a `Hooks` method, so it is attached with `Registrar::item_part(r.item_key("silentgear:pickaxe"), GearTooltip)` |
| `component_types` + `native_set_component`/`native_component` (custom component id only) | a typed `Component` descriptor `GearTag`; `stack.set(&GearTag, tag)` / `stack.get(&GearTag)` |
| `game.held()` / `game.inventory` / `game.offhand` / `native_give_stack` | `LivingRef::hand_item(host, Hand::Main)`, `PlayerRef::inventory_slots(host)`, `PlayerRef::set_inventory_slot(host, slot, stack)`, `PlayerRef::give_stack(host, stack)` |
| `game.native_feedback(Text::literal(..))` | `host.commands().feedback(Text::literal(..))` |
| `NativeCommand::op(usage, 2)` | `Command::op(usage, 2)` |

### What changed in the glue (not in `runtime.rs`)

1. **The tooltip is an item Part, not a global hook.** It fires for stacks of
   `silentgear:pickaxe` only. That is the same set the 0.1.0 hook answered for, because assembly
   is the only path that writes the component.
2. **The command is typed.** `run_mod_command` no longer receives the raw line (`perform_command`
   is gone). `action_of` maps `literals` + named `word` arguments onto the runtime's `Action`, so
   the runtime's `parse` is still unit-tested but the live path uses the dispatcher's parse. The
   reply text and every refusal string are **byte-identical** to 0.1.0, so `tools/e2e.sh`'s
   `grep`s still match.
3. **A console invocation now answers.** 0.1.0 could only be reached from chat; a `CommandSource`
   that is not a player (console, command block, function) gets `This command needs a player.` and
   `Flow::Cancel` instead of a silent no-op.
4. **The inventory write is the engine's slot write.** Consuming a part is
   `split(1)` + `set_inventory_slot` (the off hand is slot 40, so no separate hand path), and the
   assembled stack goes back through `give_stack` — `/give`'s own path. **Deviation:** where the
   0.1.0 glue refused with "no room for the pickaxe (N left)." and refunded, `give_stack` throws
   the overflow at the player's feet, so nothing is lost; a *refused write* still refunds the three
   inputs. Recorded rather than reproduced.
5. **The stat block still cannot drive vanilla mining** (unchanged limitation, gap 1): the carried
   item is the `minecraft:iron_pickaxe` template and assembly refuses a combination the item
   cannot honour.
6. **No logger, and nothing needs one.** The retired `embedded_content.rs` reported an unwritable
   `.horizon-cache` through `log::warn!`; that path and its diagnostic are gone with the embedded
   content, and a flat package cannot fail to load its own `assets/`/`data/`. Every remaining
   diagnostic is a player-facing readout: the command's refusals and `This command needs a
   player.` Nothing is silently swallowed except a refused `set_inventory_slot`, which reports as
   the same "carry a … first." refusal the player already sees (recorded, not logged).

### The ladder (measured 2026-11-18 — every command inside a `build-slot.sh` lease)

`build-slot.sh` still speaks the retired staged tool, so each command runs **directly in the
port**; the lease supplies the slot, the thread sizing and `CARGO_INCREMENTAL=0` and nothing else.
`BUILD_LOCAL=1` is used because the brief's commands `cd ports/silentgear`, and the remote runner
already starts inside the port (a forwarded run fails at that `cd`). No `--target-dir`, no stage,
no mixins plan.

| Step | Command | Result |
|---|---|---|
| runtime only, **no glue** | `cargo test --offline` | **21 passed**, 0 failed — `sdk-test-runtime-only.log` |
| glue red | same, with `action_of`/`slot_holding`/`assemble_command` on `todo!()` | **25 passed, 4 failed** ("not yet implemented") — `sdk-test-red-glue-todo.log` |
| glue green | same | **29 passed**, 0 failed (21 `runtime::`, 8 `tests::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`: that checks the fetched SDK) | `CARGOFMT_EXIT:0` — `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk`, `Built ./target/horizon-mod/silentgear.hmod` — `sdk-build.log` |
| registry check | `cargo horizon-mod check silentgear.hmod` | see the verbatim block below, **EXIT 0** — `sdk-check.log` |
| fits | `cargo horizon-mod fits silentgear.hmod --build nightly` | `Silent Gear:` / `0b14f8a  made for`, **EXIT 0** — `sdk-fits.log` |
| reproducible | a second `rm -f Cargo.lock && cargo horizon-mod build` | same 50,986 B, same SHA-256, `cmp` clean — `sdk-build-second.log`, `sdk-sha256.log` |
| runtime untouched | `cmp` against the retired file, and `unzip -p silentgear.hmod src/runtime.rs \| sha256sum` | `b65ff924…d9f1c` in all three places |

The three clippy failures the first lint run reported were mine, not the SDK's: an unusual hex
byte grouping (`0xFFAA_AA_AA`), and two `needless_borrow`s on event fields that are already
references. Fixed, then 0 warnings.

`check` output, verbatim:

```
silentgear 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  50986 bytes, SHA-256 c1bc5a29aef1b16184817a66a0aace9e661ea400d243aa1e0e916eadba9d1bb7, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

`fits` output, verbatim:

```
Silent Gear:
0b14f8a  made for
```

### The package

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `silentgear.hmod` (native + content) | 50,986 B | `c1bc5a29aef1b16184817a66a0aace9e661ea400d243aa1e0e916eadba9d1bb7` | 43 |
| `silentgear.hnmod` (retired, 0.1.0) | 37,360 B | `474a5fd4a686c5b7111317c59109b465fbf98c215bb423fee70622e927cc0516` | 10 |

43 entries: `horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
`src/lib.rs`, `src/runtime.rs`, `uses.json`, `assets/silentgear/**` (22) and `data/**` (13,
`silentgear/` and `c/`). **No `Cargo.lock`.** `notes.md`, `FINDINGS.md`, `PATCHES.md`,
`VALIDATION.md` and `tools/` are **not** packed (the packer takes the manifest, the root
documents it knows, `src/`, `assets/` and `data/`), so the content provenance a user needs travels
in `LICENSE-NOTICE.md`, which does ship.

`uses.json` (in the package, copied to `../_shared/evidence/silentgear/sdk-uses-from-package.json`)
has **179 surface entries**: **16 `hook:`** — the four this mod implements (`component_types`,
`register_commands`, `run_mod_command`, `tooltip`, with their event fields) plus the two the build
records as dependencies of the services it calls (`command` via `Commands::feedback`, `items` via
`ItemKey::id`); **18 `method:`** including `Hooks::{register, component_types, register_commands,
run_mod_command}`, `ItemPart::tooltip`, `Commands::feedback` and the three `Component` methods;
**2 `part:`** (item, entity); **2 `macro:`** (`horizon_mod`, `horizon_part`); **1
`capability:`** (`registries`); and 84 types / 39 handles / 15 traits /
1 assoc / 1 free function. Summary: `../_shared/evidence/silentgear/sdk-uses-summary.log`.

### Traps new to this port (each one cost a run here)

1. **`tooltip` is not a `Hooks` method at 0.2.** `impl Hooks { fn tooltip(..) }` fails with
   *"method `tooltip` is not a member of trait `Hooks`"*: a hook with `scope`/`key` is a **Part**
   method. The line needs `ItemPart::tooltip` and `Registrar::item_part(r.item_key(...), part)` —
   which also means a `register` hook and an item key declaration the 0.1.0 port never had.
2. **A body written with `todo!()` needs the `;` and an `allow`,** or the next statement is a
   parse error (`expected ';', found '#'`). The red run wants the stubs to *compile*, not to be a
   syntax error: a compile failure is not the fail-capability proof.
3. **`event.source` and `event.stack` are already references** — `&event.source` is a
   `needless_borrow` under `-D warnings`.
4. **`0xFFAA_AA_AA` is an `unusual_byte_groupings` error**, not a warning: write
   `0xFFAA_AAAA` (same value, equal-sized groups).
5. **`PlayerDataRefusal` has no `Display`.** Formatting a refused `give_stack` needs `{error:?}`;
   `{error}` does not compile.
6. **A `Component` descriptor is the typed route to a custom component** (`id`/`decode`/`encode`
   over `Tag`), and `ItemStack::get` reads only the stack's **patch** — no item defaults — which is
   what the 0.1.0 `native_component` read. `JavaString` is not exported: `"key".into()` infers it.
7. **`Slot` numbers, not hands, are the inventory API.** `player_inventory_slots` numbers 0..35
   main, 36..39 armour, 40 off hand, 41 body, 42 saddle; a part in an armour slot is *not* carried
   for assembly (the retired port's scan could not see one either), so the glue filters to
   `slot <= 35 || slot == 40` and takes the lowest number — which is exactly "main first, then off
   hand".
8. **`build-slot.sh` forwards `test`/`build` to the second machine, which already `cd`s into the
   port.** A forwarded command that starts `cd ports/silentgear` dies with
   *"No such file or directory"*. Either use `BUILD_LOCAL=1` (what this job did, like every
   reference SDK migration) or drop the `cd` for a forwarded run.
9. **`literals` is the matched literal path and a `word` argument keeps its registered name.**
   `silentgear assemble pickaxe iron wood` arrives as `literals = ["silentgear","assemble"]` and
   three `String` arguments named `gear`/`head`/`rod` — the same shape `framedblocks`' `/framedblocks
   camo` proves in game. A registered usage is executable at its prefixes, so a lone
   `["silentgear"]` is still handled (as help) rather than falling through.

### Changelog / supersede text (for the submission)

> **silentgear 0.2.0** — republished as a native `.hmod` with the SDK from the 2026-11-18 Nightly.
> Supersedes **#851** (`silentgear` 0.1.0, the retired `.hnmod`), closed by the format purge.
>
> What changed for `api = "0.2"`: the content and the code are **one package** — `assets/` and
> `data/` sit beside `src/` and `horizon-mod.toml`, so the AE2 embedded-content trick, its
> `content_root` injection and the `.horizon-cache` write-out are gone. The plan file, the four
> per-hook helper files and the `support` module are replaced by one `#[mod_api::horizon_mod] impl
> Hooks` (typed `component_types`, `register_commands`, `run_mod_command`) plus an `ItemPart`
> for the tooltip, which is a scoped item hook at this API. The custom `silentgear:gear` component
> is now a typed `Component` descriptor. `/silentgear assemble|stats|help` and the item tooltip
> behave as before, with byte-identical readouts; the pins are `0b14f8a…` / `e59da90…` and the
> version is the port's own **0.2.0**. `src/runtime.rs` is byte-identical to 0.1.0 (same SHA-256),
> and `cargo test` is 29 passed / 0 failed with clippy 0 warnings. **No in-game run for this
> version**: this machine has no `horizon-launcher`, so no game at the pinned SDK exists yet.


`cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` were
run before hand-back (see the table); `#[expect(..., reason)]` is used only
where the injected module would otherwise report dead code.

## Skills

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` were run before
hand-back (see the ladder above); `#[expect(…, reason = "…")]` is used only where the injected
module would otherwise report dead code.

## Gaps (next slices)

0. **No in-game run for 0.2.0 (the migration's own gap).** `horizon-launcher` is not installed on
   this machine, so nothing can compile this package into a `horizon-client`, and there is no
   shared binary at the SDK's pin (`ports/_shared/bin/horizon-0b14f8ac/` does not exist).
   `tools/e2e.sh` is rewritten for a launcher-built game and keeps its fail-capable `SELFTEST=1`
   leg, but **no PASS is claimed**: the 0.1.0 transcripts in `../_shared/evidence/silentgear/`
   belong to the retired `.hnmod`. The next slice on a machine with a launcher is: build the game,
   run `tools/e2e.sh`, then `SELFTEST=1`, then `CONTROL_GAME=` — and record all three.
1. **Per-stack tool stats (the big one).** Mining speed, correct-for-drops,

   wear-per-block and max damage are item-level at the pin
   (`items.rs:1135/1156`, `game.rs:6531/14312`, `game.rs:9118`) and the
   `break_speed` hook is pending (`native_hooks.rs:2768`). Until upstream stamps
   it (or the engine reads stack components), only the iron combination is
   functional; stone/diamond/gold/flint heads are computed and refused.
2. **Other gear.** Axe, shovel, hoe, sword, dagger, armour, hammer, excavator,
   sickle, shears, bow/crossbow, shield, fishing rod, trident, arrows, curios.
3. **Traits, grades and synergy.** The material files' trait lists and condition
   gates are carried in the data only; no trait effects, no grading/synergy.
4. **Compound parts and the custom recipe types.** `silentgear:gear_crafting`,
   `compound_part`, the `blueprint`/`material`/`part_type` ingredient types and
   `silentgear:material` recipes need a recipe-type API; slice 1 uses the
   command and the vanilla blueprint recipe.
5. **Salvage, repair, the blueprint book, upgrade parts** (`red_card`,
   `wide_plate`, …), the stone anvil/alloy forge/stellar refractor machines.
6. **The client renderer.** Upstream's `silentgear:gear` item model (material
   tinting, broken overlay, part markers) is replaced by a generated model with
   upstream's textures; no `item_model_property`/material tint, no traits on the
   tooltip.
7. **Enchanting at the assembled value.** The item's `enchantable` is 14 (iron);
   per-stack enchantability needs the same stack-level read as gap 1.
8. **The full content carry.** 63 blocks, worldgen (12 files), 363 kept
   recipes, 76 traits, advancements and curios integration stay out of the
   slice's package.

## Provenance (upstream analysis)

Repository `https://github.com/SilentChaos512/Silent-Gear`, branch `1.21.x`,
commit `1dc29c7` (4.1.6.1; the pack's 4.2.1.1 is the same 1.21.x line). Areas
read: `api/item/GearItem`, `GearDiggerTool`, `item/gear/BasicGearItem`,
`GearPickaxeItem`; `api/property/NumberProperty`, `GearProperty`, `GearType`;
`gear/part/PartInstance`, `CoreGearPart`, `PartList`; `setup/gear/GearTypes`,
`GearProperties`; `util/GearData`; `crafting/recipe/ShapelessGearRecipe`; plus
the jar's `silentgear_materials/*.json`, `silentgear_parts/*.json`, recipes,
models and lang. Full extraction and the pin comparison live in
[`FINDINGS.md`](FINDINGS.md). This is a clean-room rewrite: no Java was copied
or translated, and no jar was decompiled.
