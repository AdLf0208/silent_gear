# Silent Gear — upstream analysis (slice 1: gear crafting + tool stats)

Analysed from the public source `https://github.com/SilentChaos512/Silent-Gear`
(branch `1.21.x`, commit `1dc29c7`, the same line as the pack's 4.2.1.1) and the
pack jar itself (`silent-gear-1.21.1-neoforge-4.2.1.1.jar`, MIT, SHA-1
`928064b0…f581`, provenance in
[`../_shared/carry-work/silentgear/SOURCE.md`](../_shared/carry-work/silentgear/SOURCE.md)).
Behaviour only: no code was copied or translated. This is a clean-room Rust
rewrite.

## The model upstream builds

| Concept | Upstream | Data read |
|---|---|---|
| Gear type | `GearType` registry entry (`silentgear:pickaxe` → `silentgear:harvest_tool` → `silentgear:tool` → `silentgear:all`), with a **durability stat** and a set of **relevant property groups** | `setup/gear/GearTypes.java` |
| Part type | `PartType` registry (`silentgear:main`, `silentgear:rod`, `silentgear:tip`, …); a gear type has required parts | `setup/gear/PartTypes.java`, `GearItem.REQUIRED_PARTS` |
| Part | a `GearPart` data entry (`silentgear:pickaxe_head`, `silentgear:rod`, …) with the gear type it belongs to, its part type and its own property modifiers | `data/silentgear/silentgear_parts/*.json` (49) |
| Material | a `Material` data entry with a `properties` map **per part type**; each property is a number or `{value, operation}` | `data/silentgear/silentgear_materials/*.json` (138) |
| Property | a registry entry with a base value, minimum/maximum, a default operation and a clamp; `durability`, `harvest_speed`, `enchantment_value`, `harvest_tier`, `attack_damage`, `attack_speed`, `repair_efficiency`, … | `setup/gear/GearProperties.java` |
| Number operation | `AVERAGE`, `MAX`, `ADD`, `MULTIPLY_BASE`, `MULTIPLY_TOTAL` | `api/property/NumberProperty.java` |

### Stat combination (the rule the rewrite implements)

`GearData.calculateBaseProperties` collects, for every property the gear type
cares about, every modifier the gear's parts contribute for their own part
type, then `NumberProperty.compute` folds them:

1. `base` (0 for `durability`, `harvest_speed`, `enchantment_value`, `harvest_tier` has its own type);
2. `AVERAGE`: a **weighted** average — each modifier's weight is
   `1 + value / (1 + |first AVERAGE value|)`; `sum(value × weight) / sum(weight)`;
3. `MAX`: the largest value;
4. `MULTIPLY_BASE`: `f1 += f0 × value` for each;
5. `MULTIPLY_TOTAL`: `f1 *= 1 + value` for each;
6. `ADD`: `f1 += value` for each;
7. clamp to the property's range (`0 .. i32::MAX` for these three).

A part's contributions are its material's properties for **that part type**
plus the part's own base modifiers (`CoreGearPart.getPropertyModifiers`), and
the pickaxe head is `main`, the rod is `rod`. The durability is
`round(value)` (`GearType.getBaseDurability`). `harvest_tier` is a separate
value type (`HarvestTierProperty`) whose default is `ZERO`; the main material's
tier (e.g. iron → `incorrect_for_iron_tool`) becomes the tool's deny tag.

Worked example from the jar (what the port must reproduce):

| material | durability | harvest_speed | enchantment_value | tier |
|---|---:|---:|---:|---|
| `silentgear:iron` (main) | 250 | 6.0 | 14 | iron |
| `silentgear:wood` (rod) | — (no numeric rod properties; traits only) | — | — | — |
| **iron-headed pickaxe** | **250** | **6.0** | **14** | **iron** |

These are exactly vanilla's iron pickaxe components in 26.3
(`minecraft:max_damage` 250, `minecraft:tool` rule speed 6.0, `minecraft:enchantable` 14).

### How the gear item gets its behaviour (and why the pin cannot do it per stack)

Upstream builds the item's vanilla data components **per stack** at assembly:

- `GearItem.construct` → `GearData.writeConstructionParts` → `recalculateGearData`
  → `setGearDataComponentsFromProperties` → each property's `addDataComponents`
  (`enchantment_value` sets `DataComponents.ENCHANTABLE`, for instance);
- `BasicGearItem.getMaxDamage(stack)` returns the gear's `durability` property;
- `GearDiggerTool.createToolProperties` builds a `Tool` component per stack:
  `deniesDrops(<tier>.incorrectForTool())` + `minesAndDrops(#minecraft:mineable/pickaxe, harvest_speed)`.

On NeoForge 1.21.1 all three are read per stack (`ItemStack.get(DataComponents.TOOL)`,
`getMaxDamage`), which is why Silent Gear's materials change mining speed and
durability.

**Horizon at pin `1d63e0a` reads item defaults, not stack components:** 

| What | Pin file:line | Consequence |
|---|---|---|
| mining speed | `crates/horizon-items/src/items.rs:1135` (`Items::mining_speed` → `ItemInfo.tool`) | a stack's `minecraft:tool` component is ignored |
| correct-for-drops | `crates/horizon-items/src/items.rs:1156` (`correct_for_drops`, same `ItemInfo`) | a stack's tool rules cannot grant drops |
| wear per block | `crates/horizon-client/src/game.rs:6531` (`mine_block_with_held`: `ItemInfo.tool_damage_per_block`, `ItemInfo.max_damage`) | stack `max_damage`/`damage_per_block` ignored |
| breaking point | `crates/horizon-client/src/game.rs:14312-14314` (`damage_held_in_slot`: `ItemInfo.max_damage`) | stack `max_damage` ignored |
| HUD bar denominator | `crates/horizon-client/src/game.rs:9118` | item `max_damage` |
| the faithful per-stack hook | `crates/horizon-client/src/game/native_hooks.rs:2768` (`native_break_speed`), fed from `game/hand.rs:421` | **not in `native-hooks.json`**; only the pending fragment `tools/native-hook-fragments/break_speed.json` |
| setting components | `crates/horizon-client/src/game/native_components.rs:48` (`native_set_component`) | custom (non-`minecraft`) component ids only |

So a *per-stack* tool stat block cannot drive vanilla mining at this pin: the
`break_speed` hook that could adjust speed is pending (`E_HOOK_UNKNOWN`), and
the engine's tool reads are item-level. The campaign's rule is to port what
the named hooks allow and record the gap, not to add a raw mixin.

### The recipes upstream uses

`data/silentgear/recipe/gear/pickaxe.json` is `silentgear:gear_crafting`:
`pickaxe_head` + a `silentgear:rod` part → `silentgear:pickaxe`.
`gear/pickaxe_quick.json` adds a `silentgear:blueprint` (pickaxe) + three main
materials. `compound_part` makes the head from a blueprint + three materials.
Those recipe **types** (`gear_crafting`, `compound_part`, the `blueprint` /
`material` / `part_type` ingredient types) are custom serializers; Horizon's
recipe loader only knows vanilla types, and custom recipe types are a later
API. The blueprint item itself is made by a plain shaped recipe
(`recipe/pickaxe_blueprint.json`: 3 paper + 2 sticks).

### Durability / enchantability / traits

- Wear is vanilla's `hurtAndBreak` against the per-stack max damage; Unbreaking
  applies first (`BasicGearItem.damageItem` → `GearHelper.damageItem`).
- `enchantment_value` sets `ENCHANTABLE`; upstream also allows disabling
  enchanting in its config.
- `traits` are a second modifier system (bonus properties per trait level,
  condition-gated). Out of scope for slice 1; the trait lists stay in the
  carried material data.

## What slice 1 takes (and what it refuses to invent)

**In scope:** one `silentgear:pickaxe`, assembled from a `silentgear:pickaxe_head`
(main) and a `silentgear:rod` (rod) plus the `silentgear:pickaxe_blueprint`,
whose material stat block (`durability`, `harvest_speed`, `enchantment_value`,
`harvest_tier`) is computed by the pure runtime and written into the stack's
`silentgear:gear` component; vanilla mining then uses the item's components.

**The slice's materials:** `silentgear:iron` head + `silentgear:wood` rod. The
carried pickaxe item's components are the iron stat block (250 / 6.0 / 14 /
iron) — the one combination the pin's item-level tool reads can honour. The
runtime still implements the full operation fold and is tested over synthetic
iron/stone/diamond parts; the command **refuses** a combination whose stat
block differs from the item's fixed components (a stone or diamond head), so
the port never hands out an item whose component says 1561 durability while
vanilla breaks it at 250.

**Out of scope (gap list, next slices):** other tools and armour; all other
materials as *functional* gear (needs per-stack tool stats — the `break_speed`
stamp and item-level reads above); traits and their bonus properties; grading
and synergy; salvage/repair; compound materials and the custom recipe types;
the blueprint book and other blueprints; the gear-slot/Curios system; the
client renderer that tints the gear model per material (slice 1 patches the
item model to a generated one from upstream's own textures); upgrade parts.
