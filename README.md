# Silent Gear — a Horizon port

**Silent Gear** is a Minecraft mod. Modular tool/armor mod (formerly part of Silent's Gems). This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [SilentChaos512/Silent-Gear](https://github.com/SilentChaos512/Silent-Gear) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Silent Gear ([source](https://github.com/SilentChaos512/Silent-Gear)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `silentgear.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **One gear: `silentgear:pickaxe`**, assembled by
- **Material stat block.** The material table is the jar's own data
- **The stat block lives on the item.** The assembled stack carries a
- **Vanilla mining drives it:** stone breaks in 8 ticks with the pickaxe and
- **Refusals, not lies.** The command refuses a combination whose folded block
- **The tooltip is an item Part, not a global hook.** It fires for stacks of
- **The command is typed.** `run_mod_command` no longer receives the raw line (`perform_command`
- **A console invocation now answers.** 0.1.0 could only be reached from chat; a `CommandSource`

## Not yet ported

- **No in-game run for 0.2.0 (the migration's own gap).** `horizon-launcher` is not installed on
- **Per-stack tool stats (the big one).** Mining speed, correct-for-drops,
- **Other gear.** Axe, shovel, hoe, sword, dagger, armour, hammer, excavator,
- **Traits, grades and synergy.** The material files' trait lists and condition
- **Compound parts and the custom recipe types.** `silentgear:gear_crafting`,
- **Salvage, repair, the blueprint book, upgrade parts** (`red_card`,
- **The client renderer.** Upstream's `silentgear:gear` item model (material
- **Enchanting at the assembled value.** The item's `enchantable` is 14 (iron);

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `silentgear.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Silent Gear was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/SilentChaos512/Silent-Gear

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run silentgear` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/silentgear/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
