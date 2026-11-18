//! Silent Gear for Horizon — slice 1: one gear, assembled from parts with material stats.
//!
//! Upstream assembles a gear from a blueprint, a head and a rod, folds the parts' material
//! properties into one stat block and stores it on the assembled stack. This package does the
//! same at the SDK: [`runtime`] holds the pure model (the material table, Silent Gear's
//! `NumberProperty` fold, the gear record and its codec — byte-identical to the retired
//! bundle's `runtime.rs`), and this file is the glue:
//!
//! | Retired `.hnmod` injection (API) | here |
//! |---|---|
//! | `component-types` (`component_types`, 65) | [`Mod::component_types`] declares `silentgear:gear` |
//! | `register-commands` (`register_commands`, 8) | [`Mod::register_commands`] |
//! | `perform-command` (`perform_command`, 8) | [`Mod::run_mod_command`], typed `CommandInvocation` |
//! | `gear-tooltip` (`tooltip`, 12) | [`GearTooltip`], an `ItemPart` on the pickaxe (`tooltip` is a scoped hook, not a `Hooks` method) |
//! | `embedded-content` (`content_root` HEAD, AE2 pattern) | gone: one flat `.hmod` carries `assets/` and `data/` |
//!
//! The stat block cannot drive vanilla mining at this pin, and never could: tool stats are read
//! from the **item**, not from the stack (`docs/mods.md`; the `break_speed` hook the retired
//! port waited for is still not stamped). So the carried `silentgear:pickaxe` is declared from
//! the `minecraft:iron_pickaxe` template and the command refuses a combination the item cannot
//! honour, instead of handing out a tool whose component claims one stat and whose mining uses
//! another. See `notes.md` gaps 1 and 6.
use mod_api::{
    self, ClientHost, Command, CommandArgument, CommandArgumentValue, CommandSource, Component,
    ComponentTypesEvent, Flow, Hand, Hooks, Identifier, ItemPart, ItemStack, PlayerRef,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost, Tag,
    Text, TooltipEvent,
};

pub mod runtime;

use runtime::{
    AXE_HEAD, Action, BLUEPRINT, COMPONENT, GearRecord, HEAD, PICKAXE, PartType, ROD, Value,
    assemble, help_lines, material, stats_line_with_damage, supported,
};

/// The tooltip colour of the stat line, in the HUD's `0xAABBGGRR` packing (`rgba(170, 170, 170,
/// 255)`, vanilla's secondary tooltip grey).
const STAT_COLOR: u32 = 0xFFAA_AAAA;

/// The last slot of the player's main inventory (`Inventory`: 0 to 35, the hotbar 0 to 8).
const MAIN_INVENTORY_END: u32 = 35;

/// The off-hand slot number (`Inventory` 40, what `player_inventory_slots` reports).
const OFF_HAND: u32 = 40;

/// Answered when a usage that needs an acting player comes from the console.
const NEEDS_PLAYER: &str = "This command needs a player.";

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Attaches the tooltip Part to the assembled gear's item. `tooltip` is a scoped hook, so it
    /// is a Part method: the game asks it only for this item's stacks.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        // Bound first: `registrar.item_part(registrar.item_key(id), part)` is a double `&mut`.
        let pickaxe = r.item_key(PICKAXE);
        r.item_part(pickaxe, GearTooltip);
    }

    /// Registers the item component an assembled gear carries, before content (and so the
    /// items' stack codecs) loads — `RegisterEvent`, as the retired `component_types` hook did.
    fn component_types(&self, _context: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(COMPONENT.to_owned());
    }

    /// `/silentgear assemble|stats|help`, at operator level 2 as the retired port declared them.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        for usage in [
            "silentgear assemble <gear:word> <head:word> <rod:word>",
            "silentgear stats",
            "silentgear help",
        ] {
            event.commands.push(Command::op(usage, 2));
        }
    }

    /// Runs one of the usages above. The readouts are byte-for-byte the retired port's, so the
    /// scenario in `tools/e2e.sh` asserts the same text it always did.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        if event.cancelled {
            return Flow::Continue;
        }
        let action = action_of(&event.command.literals, &event.command.arguments);
        if action == Action::Unknown {
            return Flow::Continue;
        }
        let Some(player) = command_player(event.source) else {
            let _ = host.commands().feedback(Text::literal(NEEDS_PLAYER));
            return Flow::Cancel;
        };
        let lines: Vec<String> = match action {
            Action::Help => help_lines().into_iter().map(str::to_owned).collect(),
            Action::Stats => vec![stats_text(&*host, player)],
            Action::Assemble { gear, head, rod } => {
                vec![assemble_command(host, player, &gear, &head, &rod).unwrap_or_else(|line| line)]
            }
            Action::Unknown => return Flow::Continue,
        };
        for line in lines {
            let _ = host.commands().feedback(Text::literal(line));
        }
        Flow::Cancel
    }
}

/// `Item.appendHoverText`: the assembled gear's stat line, read from the component on the stack.
struct GearTooltip;

#[mod_api::horizon_part]
impl ItemPart for GearTooltip {
    fn tooltip(&self, _host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let Some(record) = gear_record(event.stack) else {
            return;
        };
        event.lines.push((
            Text::literal(stats_line_with_damage(&record, event.stack.damage())),
            STAT_COLOR,
        ));
    }
}

// ---------------------------------------------------------------------------
// The `silentgear:gear` component
// ---------------------------------------------------------------------------

/// The `silentgear:gear` component: the gear type, its parts and their folded stats, as the one
/// compound the retired port's `native_set_component` wrote.
struct GearTag;

impl Component for GearTag {
    type Value = Tag;

    fn id(&self) -> &str {
        COMPONENT
    }

    /// Only a compound is this component: anything else on a stack is not a gear record and is
    /// read as "not assembled Silent Gear" rather than guessed at.
    fn decode(&self, value: &Tag) -> Option<Tag> {
        matches!(value, Tag::Compound(_)).then(|| value.clone())
    }

    fn encode(&self, value: Tag) -> Tag {
        value
    }
}

/// Reads the gear record a stack carries.
fn gear_record(stack: &ItemStack) -> Option<GearRecord> {
    let tag = stack.get(&GearTag)?;
    let fields = tag_to_fields(&tag)?;
    GearRecord::decode(&fields).ok()
}

/// Writes a gear record into a stack's component patch; whether the stack took it.
fn write_gear(stack: &mut ItemStack, record: &GearRecord) -> bool {
    stack.set(&GearTag, fields_to_tag(&record.encode())).is_ok()
}

/// Converts the runtime's flat component pairs into NBT.
fn fields_to_tag(fields: &[(String, Value)]) -> Tag {
    Tag::compound(
        fields
            .iter()
            .map(|(key, value)| (key.as_str(), value_to_tag(value))),
    )
}

fn value_to_tag(value: &Value) -> Tag {
    match value {
        Value::Int(number) => Tag::Int(*number as i32),
        Value::Double(number) => Tag::Double(number.to_bits()),
        Value::Str(text) => Tag::String(text.as_str().into()),
        Value::List(items) => Tag::List(items.iter().map(value_to_tag).collect()),
        Value::Compound(pairs) => Tag::compound(
            pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value_to_tag(value))),
        ),
    }
}

/// Converts an NBT compound back into the runtime's flat pairs.
fn tag_to_fields(tag: &Tag) -> Option<Vec<(String, Value)>> {
    let Tag::Compound(pairs) = tag else {
        return None;
    };
    Some(
        pairs
            .iter()
            .map(|(key, value)| (key.to_string_lossy(), tag_to_value(value)))
            .collect(),
    )
}

fn tag_to_value(tag: &Tag) -> Value {
    match tag {
        Tag::Byte(number) => Value::Int(i64::from(*number)),
        Tag::Short(number) => Value::Int(i64::from(*number)),
        Tag::Int(number) => Value::Int(i64::from(*number)),
        Tag::Long(number) => Value::Int(*number),
        Tag::Float(number) => Value::Double(f64::from(f32::from_bits(*number))),
        Tag::Double(number) => Value::Double(f64::from_bits(*number)),
        Tag::String(text) => Value::Str(text.to_string_lossy()),
        Tag::List(items) => Value::List(items.iter().map(tag_to_value).collect()),
        Tag::Compound(pairs) => Value::Compound(
            pairs
                .iter()
                .map(|(key, value)| (key.to_string_lossy(), tag_to_value(value)))
                .collect(),
        ),
        _ => Value::Str(String::new()),
    }
}

// ---------------------------------------------------------------------------
// The command
// ---------------------------------------------------------------------------

/// The action a dispatched usage stands for. `literals` is the literal path the dispatcher
/// parsed (the command name and its subcommands); the typed arguments carry the words the old
/// glue split out of the raw line.
fn action_of(literals: &[String], arguments: &[CommandArgument]) -> Action {
    match literals {
        [root] if is_root(root) => Action::Help,
        [root, sub] if is_root(root) => match sub.as_str() {
            "help" => Action::Help,
            "stats" => Action::Stats,
            "assemble" => match (
                word_argument(arguments, "gear"),
                word_argument(arguments, "head"),
                word_argument(arguments, "rod"),
            ) {
                (Some(gear), Some(head), Some(rod)) => Action::Assemble { gear, head, rod },
                _ => Action::Unknown,
            },
            _ => Action::Unknown,
        },
        _ => Action::Unknown,
    }
}

/// Whether `literal` is this mod's command root (upstream accepts `sg` as well).
fn is_root(literal: &str) -> bool {
    literal == "silentgear" || literal == "sg"
}

/// The word argument `name`, if the dispatcher resolved one.
fn word_argument(arguments: &[CommandArgument], name: &str) -> Option<String> {
    match arguments
        .iter()
        .rev()
        .find(|argument| argument.name == name)
        .map(|argument| &argument.value)
    {
        Some(CommandArgumentValue::String(word)) => Some(word.clone()),
        _ => None,
    }
}

/// The player a command came from; the console and command blocks have none.
fn command_player(source: &CommandSource) -> Option<PlayerRef> {
    match source {
        CommandSource::Player { player, .. } => Some(*player),
        _ => None,
    }
}

/// The held stack's stat block, as `/silentgear stats` answers it.
fn stats_text(host: &dyn ServerHost, player: PlayerRef) -> String {
    let Some(held) = held_item(host, player) else {
        return "Hold a Silent Gear tool first.".to_owned();
    };
    match gear_record(&held) {
        Some(record) => stats_line_with_damage(&record, held.damage()),
        None => format!("{} is not assembled Silent Gear.", held.item().id(host)),
    }
}

/// The stack in the player's main hand.
fn held_item(host: &dyn ServerHost, player: PlayerRef) -> Option<ItemStack> {
    let living = host.living(player.entity())?;
    living.hand_item(host, Hand::Main)
}

/// Builds the pickaxe from the blueprint and the parts in the player's inventory: one of each is
/// consumed (main slots first, then the off hand), the folded stats are refused where the carried
/// item cannot honour them, and the assembled stack goes back with its component.
fn assemble_command(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    gear: &str,
    head: &str,
    rod: &str,
) -> Result<String, String> {
    if gear != "pickaxe" {
        return Err(format!("this slice builds only the pickaxe, not {gear}."));
    }
    let Some(head_material) = material(head) else {
        return Err(format!("unknown material {head}."));
    };
    let Some(rod_material) = material(rod) else {
        return Err(format!("unknown material {rod}."));
    };

    // The parts and the blueprint must be carried; the wrong part is named so the e2e's control
    // reads as a refusal, not a silent no-op.
    let carried = carried_items(&*host, player);
    if slot_holding(&carried, BLUEPRINT).is_none() {
        return Err(format!("carry a {BLUEPRINT} first."));
    }
    if slot_holding(&carried, HEAD).is_none() {
        if slot_holding(&carried, AXE_HEAD).is_some() {
            return Err(format!("a {AXE_HEAD} is not a {HEAD}."));
        }
        return Err(format!("carry a {HEAD} first."));
    }
    if slot_holding(&carried, ROD).is_none() {
        return Err(format!("carry a {ROD} first."));
    }

    let parts = [
        (PartType::Main, head_material),
        (PartType::Rod, rod_material),
    ];
    let stats = assemble(&parts).map_err(|error| error.to_string())?;
    let record = GearRecord {
        gear_type: PICKAXE.to_owned(),
        parts: vec![
            (PartType::Main, material_id(head_material.id)),
            (PartType::Rod, material_id(rod_material.id)),
        ],
        stats,
    };
    supported(&record).map_err(|error| error.to_string())?;

    let key = Identifier::parse(PICKAXE)
        .ok()
        .and_then(|id| host.item_key(&id))
        .ok_or_else(|| format!("this world has no {PICKAXE}."))?;
    let mut stack = ItemStack::of(key);
    if !write_gear(&mut stack, &record) {
        return Err("the gear component could not be written.".to_owned());
    }

    let mut taken: Vec<ItemStack> = Vec::with_capacity(3);
    for item in [BLUEPRINT, HEAD, ROD] {
        let Some(one) = take_one(host, player, item) else {
            refund(host, player, taken);
            return Err(format!("carry a {item} first."));
        };
        taken.push(one);
    }
    if let Err(error) = player.give_stack(host, stack) {
        refund(host, player, taken);
        return Err(format!("the pickaxe could not be given ({error:?})."));
    }
    Ok(format!(
        "Assembled {PICKAXE} from {head} and {rod}: {}",
        stats_line_with_damage(&record, 0)
    ))
}

/// `silentgear:<material>` for a material's short id.
fn material_id(short: &str) -> String {
    format!("silentgear:{short}")
}

/// The player's main inventory and off hand as `(slot, item id)`.
fn carried_items(host: &dyn ServerHost, player: PlayerRef) -> Vec<(u32, String)> {
    let Some(slots) = player.inventory_slots(host) else {
        return Vec::new();
    };
    slots
        .into_iter()
        .filter(|(slot, _)| carried_slot(*slot))
        .map(|(slot, stack)| (slot, stack.item().id(host)))
        .collect()
}

/// Whether `slot` is one the assembly reads: the main inventory (0 to 35) or the off hand (40).
/// A part in an armour slot is not carried for assembly, as the retired port's scan of
/// `Inventory.items` and the off hand did not see one.
fn carried_slot(slot: u32) -> bool {
    slot <= MAIN_INVENTORY_END || slot == OFF_HAND
}

/// The lowest-numbered slot holding `item` in the carried slots, so the main inventory is used
/// before the off hand (slot 40 is the highest number).
fn slot_holding(carried: &[(u32, String)], item: &str) -> Option<u32> {
    carried
        .iter()
        .filter(|(_, id)| id == item)
        .map(|(slot, _)| *slot)
        .min()
}

/// Takes one `item` out of the player's inventory: the stack's count drops by one in place, so
/// the rest of it stays where it was.
fn take_one(host: &mut dyn ServerHost, player: PlayerRef, item: &str) -> Option<ItemStack> {
    let slots = player.inventory_slots(&*host)?;
    let (slot, stack) = slots
        .into_iter()
        .filter(|(slot, _)| carried_slot(*slot))
        .filter(|(_, stack)| stack.item().id(&*host) == item)
        .min_by_key(|(slot, _)| *slot)?;
    let split = stack.split(1);
    let taken = split.taken?;
    // A refused write leaves the slot as it was, so nothing was consumed and the caller must not
    // refund this stack.
    player
        .set_inventory_slot(host, slot, split.remainder)
        .ok()?;
    Some(taken)
}

/// Gives consumed items back where the assembly failed after taking them.
fn refund(host: &mut dyn ServerHost, player: PlayerRef, stacks: Vec<ItemStack>) {
    for stack in stacks {
        let _ = player.give_stack(host, stack);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_owned()).collect()
    }

    fn argument(name: &str, value: &str) -> CommandArgument {
        CommandArgument::new(
            name.to_owned(),
            CommandArgumentValue::String(value.to_owned()),
        )
    }

    #[test]
    fn the_registered_usages_parse_back_to_their_actions() {
        let literals = strings(&["silentgear", "stats"]);
        assert_eq!(action_of(&literals, &[]), Action::Stats);
        let literals = strings(&["silentgear", "help"]);
        assert_eq!(action_of(&literals, &[]), Action::Help);
        let literals = strings(&["silentgear", "assemble"]);
        let arguments = [
            argument("gear", "pickaxe"),
            argument("head", "iron"),
            argument("rod", "wood"),
        ];
        assert_eq!(
            action_of(&literals, &arguments),
            Action::Assemble {
                gear: "pickaxe".to_owned(),
                head: "iron".to_owned(),
                rod: "wood".to_owned(),
            }
        );
    }

    #[test]
    fn an_unregistered_or_incomplete_usage_is_unknown() {
        assert_eq!(
            action_of(&strings(&["silentgear", "salvage"]), &[]),
            Action::Unknown
        );
        assert_eq!(action_of(&strings(&["other"]), &[]), Action::Unknown);
        // A missing argument (the dispatcher omits absent optional ones) is not an assembly.
        let literals = strings(&["silentgear", "assemble"]);
        let arguments = [argument("gear", "pickaxe"), argument("head", "iron")];
        assert_eq!(action_of(&literals, &arguments), Action::Unknown);
    }

    #[test]
    fn the_short_root_is_accepted_like_upstream() {
        assert_eq!(action_of(&strings(&["sg", "stats"]), &[]), Action::Stats);
    }

    #[test]
    fn the_lowest_slot_wins_and_the_off_hand_is_last() {
        let slots = [
            (OFF_HAND, "silentgear:rod".to_owned()),
            (7, "silentgear:rod".to_owned()),
            (2, "silentgear:pickaxe_head".to_owned()),
        ];
        assert_eq!(slot_holding(&slots, "silentgear:rod"), Some(7));
        assert_eq!(slot_holding(&slots, "silentgear:pickaxe_head"), Some(2));
        assert_eq!(slot_holding(&slots, "silentgear:blueprint"), None);
        // The off hand is a fallback, not a competitor of a main slot.
        let only_off_hand = [(OFF_HAND, "silentgear:rod".to_owned())];
        assert_eq!(
            slot_holding(&only_off_hand, "silentgear:rod"),
            Some(OFF_HAND)
        );
    }

    #[test]
    fn the_assembly_reads_the_main_inventory_and_the_off_hand_only() {
        for slot in [0, 8, MAIN_INVENTORY_END, OFF_HAND] {
            assert!(carried_slot(slot), "slot {slot} is carried");
        }
        for slot in [36, 39, 41, 42] {
            assert!(!carried_slot(slot), "slot {slot} is not carried");
        }
    }

    #[test]
    fn the_gear_component_round_trips_through_nbt() {
        let record = GearRecord {
            gear_type: PICKAXE.to_owned(),
            parts: vec![
                (PartType::Main, "silentgear:iron".to_owned()),
                (PartType::Rod, "silentgear:wood".to_owned()),
            ],
            stats: runtime::ITEM_STATS,
        };
        let tag = fields_to_tag(&record.encode());
        assert!(
            matches!(tag, Tag::Compound(_)),
            "a record encodes to a compound"
        );
        let fields = tag_to_fields(&tag).expect("a compound reads back");
        assert_eq!(GearRecord::decode(&fields), Ok(record));
    }

    #[test]
    fn a_value_that_is_not_a_compound_is_not_the_component() {
        assert!(GearTag.decode(&Tag::Int(7)).is_none());
        assert!(GearTag.decode(&Tag::Compound(Default::default())).is_some());
        assert_eq!(GearTag.id(), COMPONENT);
    }

    #[test]
    fn the_material_prefix_matches_the_component_format() {
        assert_eq!(material_id("iron"), "silentgear:iron");
    }
}
