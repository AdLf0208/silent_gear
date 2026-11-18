# Silent Gear → Horizon

Silent Gear **4.2.1.1** (`silent-gear-1.21.1-neoforge-4.2.1.1.jar`, MIT) ported to Horizon as **one
flat SDK package** (`api = "0.2"`): `horizon-mod.toml` + `Cargo.toml` + `src/` beside the carried
`assets/` and `data/`, built by `cargo horizon-mod build` against the pinned SDK. The retired
`.hnmod` container and its embedded content (the AE2 `content_root` write-out) are gone: one
`.hmod` carries the items **and** the behaviour, so one install is enough.

**Slice 1:** one `silentgear:pickaxe`, assembled by `/silentgear assemble pickaxe iron wood` from
the carried blueprint, pickaxe head and rod. The pure runtime folds Silent Gear's material
properties (weighted AVERAGE, MAX, MULTIPLY_BASE, MULTIPLY_TOTAL, ADD) into the stat block (250
durability, 6.0 mining speed, 14 enchantability, iron tier for iron head + wood rod) and stores it
in the stack's `silentgear:gear` component; `/silentgear stats` and the item's tooltip read it back.

**The stat block cannot drive vanilla mining at this pin.** Tool stats are read from the *item*,
not from the stack (`docs/mods.md`), and the `break_speed` hook that would change that is not
stamped. The carried `silentgear:pickaxe` is therefore declared from the `minecraft:iron_pickaxe`
template, and the command refuses a combination whose folded block the item cannot honour (a
diamond head, say) instead of handing out a tool whose component claims one stat and whose mining
uses another. A wrong part, a missing part, an unknown material or another gear type is refused
the same way.

`notes.md` has the behaviour, the hook mapping, that limitation (`FINDINGS.md`), the deviations and
the verification; `LICENSE-NOTICE.md` lists the carried content and its provenance; the other
tools, materials, traits and recipe types are the gap list in `notes.md`.

## Build and test

Every command runs in the port. `cargo horizon-mod build` fetches `.horizon/sdk/` (gitignored) on
first use; the launcher compiles the native code against its own copy of the same pinned build.

```sh
# The pure runtime and the glue, on the host target:
CARGO_INCREMENTAL=0 cargo test --offline
CARGO_INCREMENTAL=0 cargo clippy --all-targets --offline -- -D warnings
cargo fmt --check

# The package (a Cargo.lock must not be packed, or the artifact changes):
rm -f Cargo.lock
CARGO_INCREMENTAL=0 cargo horizon-mod build      # -> target/horizon-mod/silentgear.hmod
cp target/horizon-mod/silentgear.hmod ./silentgear.hmod

# The registry's own checks on the delivered file:
cargo horizon-mod check silentgear.hmod
cargo horizon-mod fits silentgear.hmod --build nightly
```

The campaign's slot registry still wraps each compile-heavy step:

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job silentgear-sdk --port ports/silentgear \
    --step test -- bash -c 'cd ports/silentgear && cargo test --offline'
```

**In game:** `tools/e2e.sh` is the scenario (`GAME=` a launcher-built game, `SELFTEST=1` for the
fail-capability leg). It has **not** been run for 0.2.0: `horizon-launcher` is not installed here,
so no game built at this SDK's pin exists. That is recorded as a gap in `notes.md`.
