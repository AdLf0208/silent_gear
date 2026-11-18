# Licence provenance

- Mod: `silentgear` (Silent Gear) by SilentChaos512 — <https://github.com/SilentChaos512/Silent-Gear>
- Licence: **MIT**, read from the mod's own `mods.toml` in the campaign's jar audit for
  `silent-gear-1.21.1-neoforge-4.2.1.1.jar` (`ports/_shared/licence-audit.csv`, row
  `silentgear`: `declared=MIT`, `effective=MIT`, family permissive, rank 0). The jar itself is
  **not** on disk: `audit_licences.py --mod silentgear --fetch` re-read the recorded row but did
  not land the archive, so this notice quotes the audit, not the jar. Rank 0 — publishable in
  principle, but **local-only until the maintainer approves publishing** (ARCHITECTURE.md §8).
- Upstream version analysed: 4.2.1.1 (the exact ATM10 jar, SHA-1
  `928064b09748d6e963077c114d65f5ce79aff581`); the source read for behaviour was
  `SilentChaos512/Silent-Gear` branch `1.21.x` at `1dc29c7` (see the port's `FINDINGS.md`, in the
  repository).
- This port is a **clean-room Rust rewrite** written from the mod's observable behaviour, its
  data files and its public documentation; no Java source was copied or translated and no jar was
  decompiled.
- Carried content, redistributed under the same MIT licence:
  - `assets/silentgear/items/`, `assets/silentgear/models/item/` — the item definitions and models
    for `pickaxe`, `pickaxe_head`, `rod`, `pickaxe_blueprint`, `axe_head` and `blueprint_paper`;
  - `assets/silentgear/textures/item/` — the ten textures those models name;
  - `assets/silentgear/lang/en_us.json` — the jar's own language file;
  - `data/silentgear/horizon/` — the item declarations and the creative tab;
  - `data/silentgear/recipe/`, `data/silentgear/tags/item/`, `data/c/tags/item/` — the jar's
    recipes and tags, upgraded for 26.3.
- Patches applied to that carry: `silentgear:pickaxe` is declared from the `minecraft:iron_pickaxe`
  template (the carry's `minecraft:stick` guess would give no tool rule), `pickaxe_blueprint` names
  its item, the pickaxe model is generated from upstream's own textures instead of the unrenderable
  `builtin/entity` parent, `data/c/tags/item/rods/wooden.json` adds `minecraft:stick`, and
  `en_us.json` gains one key. The port's `PATCHES.md` (in the repository, not packed) has the full
  per-file table.
- The complete corresponding source of this package is this directory: `src/`, `horizon-mod.toml`,
  `Cargo.toml`, `assets/`, `data/` and the `LICENSE` text beside this notice.
