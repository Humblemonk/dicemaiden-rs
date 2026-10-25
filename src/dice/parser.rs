//! Dice expression parser: text → `Vec<DiceRoll>`.
//!
//! The public entry point is [`parse_dice_string`].  It accepts a free-form
//! string typed by a Discord user and returns a list of [`DiceRoll`] values
//! that the roller can execute.
//!
//! # Parsing pipeline
//!
//! ```text
//! raw input
//!   │
//!   ├─ alias expansion (aliases::expand_alias)
//!   ├─ semicolon split → multiple independent rolls
//!   ├─ roll-set detection  "N <expr>"  (e.g. "6 4d6 k3")
//!   └─ parse_single_dice_expression
//!         ├─ label extraction   "(Attack) …"
//!         ├─ comment extraction "… ! Fire damage"
//!         ├─ flags (p, s, nr, ul)
//!         ├─ dice core  NdS  or  d%
//!         └─ split_combined_modifiers → Vec<Modifier>
//! ```
//!
//! # Prefix-matching hazards
//!
//! Modifiers are matched in the order of the `SPLIT_MODIFIER_PATTERNS`,
//! `COMBINED_MODIFIER_PATTERNS` and `MODIFIER_START_PATTERNS` lists.
//! **Multi-character prefixes must appear before their single-character
//! counterparts** — e.g. `ie`/`irg`/`irx`/`ir` before `e`/`r`, `km`/`kl` before
//! `k`, `tl` before `t`.  Before adding a new modifier prefix, check
//! `roll_syntax.md` for conflicts.
//!
//! # Limits
//!
//! | Constraint          | Value       |
//! |---------------------|-------------|
//! | Max input length    | 1 000 chars |
//! | Max dice per pool   | 500         |
//! | Max sides per die   | 1 000       |
//! | Roll-set count      | 2 – 20      |
//! | Multi-roll segments | up to 4     |
//!
//! All regex patterns are compiled once, on first use, via `once_cell::Lazy`.

use super::{
    CONAN_MAX_FOCUS, CONAN_MAX_POOL, DiceRoll, ESSENCE20_RANKS, HeroSystemType, LaserFeelingsType,
    MAX_DICE, MOTHERSHIP_MAX_TARGET, Modifier, WFRP_MAX_TARGET,
};
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use regex::Regex;

const MAX_INPUT_LENGTH: usize = 1000;
const MAX_SIDES: u32 = 1000;

// Every pattern is compiled once, on first use. Never compile one per call:
// see the regex note in AGENTS.md.
static SET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)\s+(.+)$").expect("Failed to compile SET_REGEX"));

static DICE_ONLY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)?d(\d+|%)$").expect("Failed to compile DICE_ONLY_REGEX"));

static LABEL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\(([^)]*)\)\s*").expect("Failed to compile LABEL_REGEX"));

static COMMENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!\s*(.*)$").expect("Failed to compile COMMENT_REGEX"));

// An advantage/disadvantage alias with one trailing number: `+d20+5`, `-d% * 2`
static ADV_WITH_SIMPLE_MOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-])d(\d+|%)\s*([+\-*/])\s*(\d+)$")
        .expect("Failed to compile ADV_WITH_SIMPLE_MOD_REGEX")
});

static NUMBER_DIV_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+\s*/\s*(d\d+.*|d%.*|\d*d\d+.*)$")
        .expect("Failed to compile NUMBER_DIV_DICE_REGEX")
});

static STANDARD_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d*d\d+|d%|\d*d%)").expect("Failed to compile STANDARD_DICE_REGEX")
});

static ADVANTAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[+-]d(\d+|%)").expect("Failed to compile ADVANTAGE_REGEX"));

static WHITESPACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+").expect("Failed to compile WHITESPACE_REGEX"));

/// Internal operator token for `number / dice`, emitted by the tokenizer ahead of
/// the number. It contains a space so no whitespace-split user token can equal it.
const NUMBER_DIVIDED_BY_ROLL_OP: &str = "÷ roll";

static NUMBER_OP_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)\s*([+\-*/])\s+(d\d+.*|d%.*|\d*d\d+.*)$")
        .expect("Failed to compile NUMBER_OP_DICE_REGEX")
});

static DICE_WITH_REMAINDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d*d\d+|d%|d\d+)(.*)$").expect("Failed to compile DICE_WITH_REMAINDER_REGEX")
});

static NUMBER_SLASH_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)/(\S+)(.*)$").expect("Failed to compile NUMBER_SLASH_DICE_REGEX")
});

static BASIC_DICE_SPLIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d*d\d+)(.*)$").expect("Failed to compile BASIC_DICE_SPLIT_REGEX"));

static TARGET_DS_SIDES_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^t(\d+)ds(\d+)$").expect("Failed to compile TARGET_DS_SIDES_REGEX"));

static TARGET_DS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^t(\d+)ds$").expect("Failed to compile TARGET_DS_REGEX"));

static TARGET_LOWER_DS_SIDES_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^tl(\d+)ds(\d+)$").expect("Failed to compile TARGET_LOWER_DS_SIDES_REGEX")
});

static TARGET_LOWER_DS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^tl(\d+)ds$").expect("Failed to compile TARGET_LOWER_DS_REGEX"));

/// `tn12`, `tn12f3`, `tn12f3c19` - Conan / 2d20 target number, Focus and
/// Complication range. Matched as one whole token, the way `tl6ds5` is, so the
/// `f` and `c` inside it can never be mistaken for the `f#` failure or `c`
/// cancel modifiers.
static TWO_D20_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^tn(\d+)(?:f(\d+))?(?:c(\d+))?$").expect("Failed to compile TWO_D20_REGEX")
});

static WWC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^wwc(\d+)$").expect("Failed to compile WWC_REGEX"));

/// Dice-with-modifier shapes checked by `is_valid_dice_expression`.
/// Order is not significant — the check is a plain "any match".
static DICE_WITH_MODIFIERS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"^\d*d\d+\s+[kdie]+\d*.*$",    // "4d6 k3", "2d6 e6", "1d20 d1"
        r"^\d*d\d+\s*[+\-*/]\s*\d+.*$", // "4d6 + 3", "2d6 - 1", "1d20 * 2"
        r"^\d*d%\s+[kdie]+\d*.*$",      // "d% k1", "2d% e6"
        r"^\d*d%\s*[+\-*/]\s*\d+.*$",   // "d% + 10", "2d% - 5"
    ]
    .iter()
    .map(|pattern| {
        Regex::new(pattern).expect("Failed to compile DICE_WITH_MODIFIERS_PATTERNS entry")
    })
    .collect()
});

/// Parse the optional threshold on an explode modifier: `e`/`ie` explode on the
/// die's maximum, `e6`/`ie6` on the given value. `part` is only used to quote
/// the original token back in the error.
fn parse_explode_threshold(stripped: &str, part: &str) -> Result<Option<u32>> {
    if stripped.is_empty() {
        return Ok(None);
    }

    let threshold: u32 = stripped
        .parse()
        .map_err(|_| anyhow!("Invalid explode value in '{}'", part))?;
    if threshold == 0 {
        return Err(anyhow!("Cannot explode on 0"));
    }
    Ok(Some(threshold))
}

/// Parse and range-check a Mothership stat value (`ms45`, `ms45a`, `ms45d`).
fn parse_mothership_stat(stat_str: &str, part: &str) -> Result<u32> {
    let stat: u32 = stat_str
        .parse()
        .map_err(|_| anyhow!("Invalid Mothership stat value in '{}'", part))?;
    if !(1..=MOTHERSHIP_MAX_TARGET).contains(&stat) {
        return Err(anyhow!(
            "Mothership target must be 1-{MOTHERSHIP_MAX_TARGET}, got {stat}"
        ));
    }
    Ok(stat)
}

/// Expand the advantage/disadvantage alias in a roll-set expression, keeping any
/// trailing simple numeric modifier (`+d20-1`, `+d20 * 2`). Returns `expression`
/// unchanged when it is neither an alias nor an advantage pattern.
fn expand_roll_set_expression(expression: &str) -> String {
    if let Some(expanded) = super::aliases::expand_alias(expression) {
        // Direct alias match (like "+d20" -> "2d20 k1")
        return expanded;
    }

    // Advantage/disadvantage with modifiers (like "+d20-1" or "+d20 * 2")
    let Some(captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(expression) else {
        return expression.to_string();
    };

    let advantage_sign = &captures[1];
    let sides = &captures[2];
    let operator = &captures[3];
    let number = &captures[4];

    // Expand the advantage/disadvantage part, then recombine with the modifier
    let adv_alias = format!("{advantage_sign}d{sides}");
    match super::aliases::expand_alias(&adv_alias) {
        Some(expanded_adv) => format!("{expanded_adv} {operator} {number}"),
        None => expression.to_string(),
    }
}

/// Clone `dice` into the `count` labelled copies that make up a roll set.
fn build_roll_set(dice: &DiceRoll, count: u32) -> Vec<DiceRoll> {
    (0..count)
        .map(|i| {
            let mut set_dice = dice.clone();
            set_dice.label = Some(format!("Set {}", i + 1));
            set_dice
        })
        .collect()
}

/// Parse `input` as a roll set (`4 4d6`), transferring `metadata` (flags, label,
/// comment) onto each set when given.
///
/// `Ok(None)` means "not a roll set", and the caller falls through to
/// single-expression parsing — that fall-through is what lets an input like
/// `6 6` still parse as plain arithmetic. An out-of-range set count is an
/// error rather than a fall-through, so the user is told why it was rejected.
fn try_parse_roll_set(input: &str, metadata: Option<&DiceRoll>) -> Result<Option<Vec<DiceRoll>>> {
    let Some(captures) = SET_REGEX.captures(input) else {
        return Ok(None);
    };

    let expression = &captures[2];

    // `3 dndstats` would nest one roll set inside another; it used to roll three
    // single rolls of `6 + 4d6 k3` instead. Checked without any label, comment
    // or flags, which kept `3 dndstats ! hi` from matching the alias.
    let mut scratch = create_default_dice_roll();
    let bare = parse_flags(&mut scratch, expression);
    let bare = parse_label(&mut scratch, bare);
    let bare = parse_comment(&mut scratch, bare).trim();

    // A set's own parse errors are swallowed below (it falls back to other
    // readings), so refusals surface here: `3 ms20 - 30` said "Invalid dice
    // expression: 3".
    if let Some(refusal) = super::aliases::alien_push_refusal(bare)
        .or_else(|| super::aliases::mothership_target_refusal(bare))
    {
        return Err(anyhow!(refusal));
    }
    if let Some(expanded) = super::aliases::expand_alias(bare)
        && let Some(inner) = SET_REGEX.captures(&expanded)
        && is_valid_roll_set_expression(&inner[2])
    {
        return Err(anyhow!(
            "`{}` already rolls {} sets; roll it on its own",
            bare,
            &inner[1]
        ));
    }

    if !is_valid_roll_set_expression(expression) {
        return Ok(None);
    }

    let Ok(count) = captures[1].parse::<u32>() else {
        return Ok(None);
    };
    if !(2..=20).contains(&count) {
        return Err(anyhow!("Set count must be between 2 and 20"));
    }

    match parse_single_dice_expression(&expand_roll_set_expression(expression)) {
        Ok(mut dice) => {
            if let Some(meta) = metadata {
                transfer_dice_metadata(meta, &mut dice);
            }
            Ok(Some(build_roll_set(&dice, count)))
        }
        // Not parseable as a set member; fall through to single-expression parsing
        Err(_) => Ok(None),
    }
}

pub fn parse_dice_string(input: &str) -> Result<Vec<DiceRoll>> {
    let rolls = parse_rolls(input)?;
    for roll in &rolls {
        validate_number_divided_by_roll(roll)?;
        validate_dice_math_on_success_pool(roll)?;
        validate_single_marvel_modifier(roll)?;
        validate_no_math_on_dice_count(roll)?;
    }
    Ok(rolls)
}

/// Math the roller would drop on a dice-counting system (see
/// Modifier::counts_dice_without_math) is refused, not silently ignored:
/// `wng dn2 4d6 + 2`, `alien4 + 2`, `fitd3 + 1`. Adding dice is allowed, since
/// the added dice join the pool those systems count; refusing them (as a 2.0.0
/// draft briefly did) blocked the rules' own "+1d6".
fn validate_no_math_on_dice_count(roll: &DiceRoll) -> Result<()> {
    let Some(counts) = roll
        .modifiers
        .iter()
        .find_map(Modifier::counts_dice_without_math)
    else {
        return Ok(());
    };
    let dropped = roll
        .modifiers
        .iter()
        .find(|m| modifier_kind(m) == ModifierKind::Math && !matches!(m, Modifier::AddDice(_)));
    match dropped {
        None => Ok(()),
        Some(Modifier::Add(bonus)) if (1..=MAX_DICE as i32).contains(bonus) => Err(anyhow!(
            "{counts}, so `+ {bonus}` doesn't apply; add dice instead (`+ {bonus}d6`)"
        )),
        Some(_) => Err(anyhow!(
            "{counts}, so math doesn't apply; add dice (`+ 1d6`) or roll fewer instead"
        )),
    }
}

/// The part of a `wng` token after the wrath count: `""`, `t`, `dn3`, `dn3t`,
/// or a bare difficulty (`3`, `3t`). Returns the difficulty and total mode.
fn parse_wrath_glory_mode(rest: &str) -> Option<(Option<u32>, bool)> {
    let (rest, use_total) = match rest.strip_suffix('t') {
        Some(without_t) => (without_t, true),
        None => (rest, false),
    };
    let (rest, has_dn) = match rest.strip_prefix("dn") {
        Some(number) => (number, true),
        None => (rest, false),
    };
    if rest.is_empty() {
        // `dn` needs a number after it
        return (!has_dn).then_some((None, use_total));
    }
    rest.parse().ok().map(|dn| (Some(dn), use_total))
}

/// The Wrath & Glory mode named by the first word of a `wng` roll's comment:
/// `wng 4d6 !soak`, `wng 4d6 ! SOAK hits`.
fn wrath_glory_mode(roll: &str, comment: Option<&str>) -> Option<&'static str> {
    if !roll.starts_with("wng ") {
        return None;
    }
    let keyword = comment?.split_whitespace().next()?.to_lowercase();
    ["soak", "exempt", "dmg"]
        .into_iter()
        .find(|mode| *mode == keyword)
}

/// The roller reads one Marvel Multiverse modifier, so a second raw token was
/// ignored: `3d6 mme2 mmt1` dropped the trouble. Only raw `mm`/`mme`/`mmt`
/// tokens can repeat; the documented `mm 2e 1t` already combines into one.
fn validate_single_marvel_modifier(roll: &DiceRoll) -> Result<()> {
    let marvel_modifiers = roll
        .modifiers
        .iter()
        .filter(|m| matches!(m, Modifier::MarvelMultiverse(..)))
        .count();
    if marvel_modifiers > 1 {
        return Err(anyhow!(
            "Use one Marvel Multiverse modifier, like `mm 2e 1t`"
        ));
    }
    Ok(())
}

/// On success counting (`t#`, `f#`, `b#`), a dice operand's dice join the pool
/// and are counted like the rest. That is what `+` means (`4d10 t7 + 1d10`, and
/// the D6 Legends wild die), but the roller had no other meaning to give `-`,
/// `*` or `/`: `2d1 t1 - 1d1` counted 3 successes. No game system expands to
/// that, so it is rejected rather than counted.
fn validate_dice_math_on_success_pool(roll: &DiceRoll) -> Result<()> {
    let counts_successes = roll.modifiers.iter().any(Modifier::is_target);
    let has_non_additive_dice = roll.modifiers.iter().any(|m| {
        matches!(
            m,
            Modifier::SubtractDice(_) | Modifier::MultiplyDice(_) | Modifier::DivideDice(_)
        )
    });
    if counts_successes && has_non_additive_dice {
        return Err(anyhow!(
            "Dice can only be added to a success-counting roll (`+ 1d10` adds a die to the \
             pool); use a number with `-`, `*` or `/`"
        ));
    }
    Ok(())
}

/// `N/dice` divides N by the dice total, so it only means something on a plain
/// roll whose result is that total. On success counting or a game system there
/// is no such total: the roller used to turn `N/` into `+N` there, and since
/// 1.8.1 ignored it, so it is rejected instead.
fn validate_number_divided_by_roll(roll: &DiceRoll) -> Result<()> {
    let Some(position) = roll
        .modifiers
        .iter()
        .position(|m| matches!(m, Modifier::NumberDividedByRoll(_)))
    else {
        return Ok(());
    };
    if position == 0 && roll.modifiers[1..].iter().all(is_plain_dice_modifier) {
        return Ok(());
    }
    Err(anyhow!(
        "A number divided by dice (like `100/2d6`) only works on a plain dice roll, \
         not with success counting or a game system"
    ))
}

/// How a modifier affects a roll's result, for the leading-number and `N/dice`
/// rules and for the roller's "does this roll have math" check. Exhaustive on
/// purpose: a new modifier must be classified here, not fall through. The
/// roller once kept its own list of math modifiers; it missed `*`/`/` dice.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ModifierKind {
    /// Arithmetic with a number or a dice operand (`+2`, `*3`, `+1d6`)
    Math,
    /// Changes which dice count but leaves the result a dice total (`k3`, `e`)
    PlainDice,
    /// Success counting or a game system: the result is not the dice total
    System,
}

pub(crate) fn modifier_kind(modifier: &Modifier) -> ModifierKind {
    match modifier {
        Modifier::Add(_)
        | Modifier::Subtract(_)
        | Modifier::Multiply(_)
        | Modifier::Divide(_)
        | Modifier::AddDice(_)
        | Modifier::SubtractDice(_)
        | Modifier::MultiplyDice(_)
        | Modifier::DivideDice(_) => ModifierKind::Math,
        Modifier::Explode(_)
        | Modifier::ExplodeIndefinite(_)
        | Modifier::Implode(_)
        | Modifier::Drop(_)
        | Modifier::Advantage(_)
        | Modifier::Disadvantage(_)
        | Modifier::KeepHigh(_)
        | Modifier::KeepLow(_)
        | Modifier::KeepMiddle(_)
        | Modifier::Reroll(_)
        | Modifier::RerollIndefinite(_)
        | Modifier::RerollGreater(_)
        | Modifier::RerollGreaterIndefinite(_)
        | Modifier::RerollExact(_)
        | Modifier::RerollExactIndefinite(_) => ModifierKind::PlainDice,
        Modifier::NumberDividedByRoll(_)
        | Modifier::Target(_)
        | Modifier::TargetLower(_)
        | Modifier::TargetWithDoubleSuccess(..)
        | Modifier::TargetLowerWithDoubleSuccess(..)
        | Modifier::Failure(_)
        | Modifier::Botch(_)
        | Modifier::Cancel
        | Modifier::WrathGlory(..)
        | Modifier::Godbound(_)
        | Modifier::HeroSystem(_)
        | Modifier::Fudge
        | Modifier::DarkHeresy
        | Modifier::SavageWorlds(_)
        | Modifier::D6System(..)
        | Modifier::Shadowrun(_)
        | Modifier::MarvelMultiverse(..)
        | Modifier::CyberpunkRed
        | Modifier::CyberpunkRedDamage
        | Modifier::Wfrp(_)
        | Modifier::BrokenEmpires(..)
        | Modifier::BrokenEmpiresFavor(_)
        | Modifier::Witcher
        | Modifier::CypherSystem(_)
        | Modifier::BraveNewWorld(_)
        | Modifier::ConanSkill(_)
        | Modifier::ConanCombat(_)
        | Modifier::TwoD20Test { .. }
        | Modifier::Silhouette(_)
        | Modifier::VampireMasquerade5(..)
        | Modifier::LaserFeelings(..)
        | Modifier::Alien
        | Modifier::AlienStress(_)
        | Modifier::ForgedDark
        | Modifier::ForgedDarkZero
        | Modifier::Daggerheart
        | Modifier::WildWords(_)
        | Modifier::Mothership(..)
        | Modifier::MutantsMasterminds
        | Modifier::PlotDie
        | Modifier::Essence20(..)
        | Modifier::DarkestHouse(_) => ModifierKind::System,
    }
}

/// Modifiers that leave a roll's result as its dice total.
fn is_plain_dice_modifier(modifier: &Modifier) -> bool {
    modifier_kind(modifier) != ModifierKind::System
}

fn parse_rolls(input: &str) -> Result<Vec<DiceRoll>> {
    let input = input.trim();

    // Input length validation for DoS protection
    if input.len() > MAX_INPUT_LENGTH {
        return Err(anyhow!(
            "Input too long: {} characters (max {})",
            input.len(),
            MAX_INPUT_LENGTH
        ));
    }

    // Check for aliases that expand to roll sets
    if let Some(expanded) = super::aliases::expand_alias(input) {
        if expanded.contains(' ') && !expanded.contains(';') {
            // Check if this is a roll set pattern
            if let Some(captures) = SET_REGEX.captures(&expanded) {
                // Validate that this is actually a roll set before proceeding
                if is_valid_roll_set_expression(&captures[2]) {
                    return create_roll_set(&captures);
                }
            }
        }
        // If alias doesn't expand to roll set, parse normally
        return Ok(vec![parse_single_dice_expression(&expanded)?]);
    }

    // Check for multi-roll (semicolon separated)
    if input.contains(';') {
        return parse_semicolon_separated_rolls(input);
    }

    // PRIORITY 1: Check for roll sets FIRST, but validate expression first
    if let Some(results) = try_parse_roll_set(input, None)? {
        return Ok(results);
    }

    // Parse flags, labels, and comments
    let mut temp_dice = create_default_dice_roll();
    let after_flags = parse_flags(&mut temp_dice, input);
    let after_label = parse_label(&mut temp_dice, after_flags);
    let after_comment = parse_comment(&mut temp_dice, after_label);
    let remaining_input = after_comment.trim();

    // PRIORITY 2: Check for roll sets AGAIN after processing flags, with same validation
    if let Some(results) = try_parse_roll_set(remaining_input, Some(&temp_dice))? {
        return Ok(results);
    }

    // An alias that expands to a roll set (`dndstats` -> `6 4d6 k3`) keeps that
    // shape behind a flag, label or comment. Only the bare alias was checked,
    // so `p dndstats` read the expansion as `6 + 4d6 k3`: one roll, not six.
    if let Some(expanded) = super::aliases::expand_alias(remaining_input)
        && let Some(results) = try_parse_roll_set(&expanded, Some(&temp_dice))?
    {
        return Ok(results);
    }

    // PRIORITY 3: Handle advantage/disadvantage with simple modifiers
    // But ONLY if we're not dealing with a roll set (no space + number at start)
    // Check for advantage/disadvantage with SIMPLE numeric modifiers ONLY
    // This handles cases like "+d20+5" or "-d20-3" but NOT "+d20 + d10"
    // Use remaining_input (post-comment-stripping) so that "+d20+5 ! comment" works correctly.
    if let Some(captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(remaining_input) {
        let advantage_sign = &captures[1];
        let sides = &captures[2];
        let operator = &captures[3];
        let number = &captures[4];

        let adv_alias = format!("{advantage_sign}d{sides}");
        if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
            let full_expression = format!("{expanded_adv} {operator} {number}");
            let mut result_dice = parse_single_dice_expression(&full_expression)?;
            transfer_dice_metadata(&temp_dice, &mut result_dice);
            return Ok(vec![result_dice]);
        }
    }

    // PRIORITY 4: Parse as single expression
    let mut result_dice = parse_single_dice_expression(input)?;
    transfer_dice_metadata(&temp_dice, &mut result_dice);
    Ok(vec![result_dice])
}

/// Whether `expr`, the part after a roll set's count (`6 4d6 k3`), looks like a
/// dice expression or alias rather than arithmetic such as `6 + 6`.
fn is_valid_roll_set_expression(expr: &str) -> bool {
    let expr = expr.trim();

    // Empty expressions are not valid
    if expr.is_empty() {
        return false;
    }

    // Reject expressions that start with "/ " immediately
    if expr.starts_with("/ ") {
        return false;
    }

    // Check if it's a known alias FIRST (before rejecting operators)
    // This catches +d20, -d20, +d%, -d%, sw8, 4cod, etc.
    // This fixes "3 sw8 + 5" because "sw8 + 5" has no direct alias
    // but "sw8" does, so we need better checking
    if super::aliases::expand_alias(expr).is_some() {
        return true;
    }

    // Check for D6 Legends patterns specifically
    if let Some(before_d6l) = expr.strip_suffix("d6l") {
        // Pattern like "5d6l" or "12d6l"
        // Remove "d6l"
        if let Ok(count) = before_d6l.parse::<u32>() {
            return count > 0; // Valid if positive number
        }
    }

    // Check if the expression starts with a game system alias
    // This handles cases like "sw8 + 5" where the full expression isn't an alias
    // but it starts with one
    let game_system_prefixes = ["sw", "cod", "wod", "sr", "gb", "cpr", "wit", "df", "hs"];
    for prefix in &game_system_prefixes {
        if let Some(after_prefix) = expr.strip_prefix(prefix) {
            // Check if it's followed by valid game system patterns
            if after_prefix
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
            {
                return true; // "sw8", "4cod", etc.
            }
            if *prefix == "gb" && (after_prefix.is_empty() || after_prefix.starts_with(' ')) {
                return true; // "gb" or "gb "
            }
        }
    }

    // THEN reject expressions that start with operators (but not advantage/disadvantage)
    if expr.starts_with(['*', '/', '=', '<', '>', '!', '&', '|', '^', '%']) {
        return false;
    }

    // Special handling for + and - : only reject if they're not followed by 'd'
    if expr.starts_with(['+', '-']) {
        let after_sign = &expr[1..];
        if !after_sign.starts_with('d') {
            return false;
        }
    }

    // Check for number/dice division patterns (like "20/d6", "100/2d4")
    if NUMBER_DIV_DICE_REGEX.is_match(expr) {
        return true;
    }

    // Check for standard dice expressions (like "2d6", "d20", "4d6+3")
    if STANDARD_DICE_REGEX.is_match(expr) {
        return true;
    }

    // Check for advantage/disadvantage patterns explicitly
    if ADVANTAGE_REGEX.is_match(expr) {
        return true;
    }

    // Check for dice expressions with modifiers
    // This fixes "3 (Stat Roll) 4d6 k3" and "6 (Stat) 4d6 k3 ! Character generation"
    // After label/comment parsing, these become "4d6 k3"
    if DICE_WITH_MODIFIERS_PATTERNS
        .iter()
        .any(|regex| regex.is_match(expr))
    {
        return true;
    }

    // Must contain 'd' to be a dice expression (for remaining cases)
    if !expr.contains('d') {
        return false;
    }

    // If we get here and it contains 'd', it's probably valid
    true
}

// Helper function to create roll sets, eliminating duplication
fn create_roll_set(captures: &regex::Captures) -> Result<Vec<DiceRoll>> {
    create_roll_set_with_metadata(captures, None)
}

// Helper function to parse semicolon-separated rolls
fn parse_semicolon_separated_rolls(input: &str) -> Result<Vec<DiceRoll>> {
    let parts: Vec<&str> = input.split(';').collect();
    if parts.len() > 4 {
        return Err(anyhow!("Maximum of 4 separate rolls allowed"));
    }

    let mut results = Vec::with_capacity(parts.len());
    for part in parts {
        let part = part.trim();
        let mut sub_results = parse_dice_string(part)?;
        for dice in &mut sub_results {
            dice.original_expression = Some(part.to_string());
        }
        results.extend(sub_results);
    }
    Ok(results)
}

fn parse_single_dice_expression(input: &str) -> Result<DiceRoll> {
    let mut dice = create_default_dice_roll();

    // Normalize and handle the input
    let normalized_input = normalize_whitespace(input.trim());
    let mut remaining = normalized_input.as_str();

    // Parse flags, labels, and comments
    remaining = parse_flags(&mut dice, remaining);
    remaining = parse_label(&mut dice, remaining);
    remaining = parse_comment(&mut dice, remaining);
    remaining = remaining.trim();

    // Handle D6 System alias expansion BEFORE general alias expansion
    // This prevents the "d6s5" -> "5d6 + 1d6ie" from being mis-parsed
    if remaining.starts_with("d6s")
        && let Some(expanded) = super::aliases::expand_alias(remaining)
    {
        // D6 System expansion: "d6s5" -> "1d1 d6s5"
        // This creates a dummy roll that triggers the D6System modifier
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;
        transfer_dice_metadata(&dice, &mut expanded_dice);
        return Ok(expanded_dice);
    }

    if let Some(refusal) = super::aliases::alien_push_refusal(remaining)
        .or_else(|| super::aliases::mothership_target_refusal(remaining))
    {
        return Err(anyhow!(refusal));
    }

    // Wrath & Glory's `!soak`, `!exempt` and `!dmg` are modes written as a
    // comment, as the Ruby bot read them. parse_comment has already taken the
    // comment, which left them unreachable, so the keyword goes back to the
    // alias. The text stays on the roll as its comment.
    if let Some(mode) = wrath_glory_mode(remaining, dice.comment.as_deref())
        && let Some(expanded) = super::aliases::expand_alias(&format!("{remaining} !{mode}"))
    {
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;
        transfer_dice_metadata(&dice, &mut expanded_dice);
        return Ok(expanded_dice);
    }

    // Check for simple advantage/disadvantage patterns (without additional modifiers)
    // Only do alias expansion, don't try to be clever about advantage detection here
    if let Some(expanded) = super::aliases::expand_alias(remaining) {
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;
        // Transfer flags and metadata
        transfer_dice_metadata(&dice, &mut expanded_dice);
        return Ok(expanded_dice);
    }

    // Parse the expression into parts
    let parts = parse_expression_to_parts(remaining)?;

    if parts.is_empty() {
        return Err(anyhow!("No dice expression found"));
    }

    // Check for advantage/disadvantage as the first part of a larger expression
    // ONLY do this if we have multiple parts (indicating it's a complex expression)
    if parts.len() > 2
        && (parts[0] == "+d20" || parts[0] == "-d20" || parts[0] == "+d%" || parts[0] == "-d%")
    {
        // This is an advantage/disadvantage pattern with additional modifiers
        // Expand the first part and then handle the rest
        if let Some(expanded_first) = super::aliases::expand_alias(&parts[0]) {
            // Parse the expanded advantage/disadvantage
            let mut adv_dice = parse_single_dice_expression(&expanded_first)?;

            // Apply remaining modifiers
            parse_all_modifiers(&mut adv_dice, &parts[1..])?;

            // Transfer metadata
            transfer_dice_metadata(&dice, &mut adv_dice);
            return Ok(adv_dice);
        }
    }

    // Check if this is an expression that starts with a number (like "4 + 4d10" or "5 - 2d1")
    if parts.len() > 2 && parts[0].parse::<i32>().is_ok() && !parts[0].contains('d') {
        // Find the first dice expression in the parts
        let mut dice_index = None;
        for (i, part) in parts.iter().enumerate() {
            if part.contains('d') || is_dice_expression(part) {
                dice_index = Some(i);
                break;
            }
        }

        if let Some(dice_idx) = dice_index {
            // Parse the dice part
            parse_base_dice(&mut dice, &parts[dice_idx])?;

            // Numbers in front of the dice fold left to right, the same order
            // as math after the dice: `10 + 2 + 2d6` is 12 + 2d6.
            let initial_number = fold_leading_numbers(&parts[..dice_idx])?;

            // Check if there's an operator before the dice
            let operator = if dice_idx > 1 {
                parts[dice_idx - 1].as_str()
            } else {
                "+"
            };
            let leading = match operator {
                "+" => vec![Modifier::Add(initial_number)],
                "-" => vec![Modifier::Multiply(-1), Modifier::Add(initial_number)],
                "*" => vec![Modifier::Multiply(initial_number)],
                "/" => vec![Modifier::NumberDividedByRoll(initial_number)],
                _ => return Err(unexpected_before_dice(&parts[..dice_idx])),
            };

            // Parse remaining modifiers after the dice
            if dice_idx + 1 < parts.len() {
                parse_all_modifiers(&mut dice, &parts[dice_idx + 1..])?;
            }

            // The leading number goes after the dice's own modifiers and before
            // any math that follows them, so `4 + 4d10 t7` and `4 + 4d10t7`
            // both add 4 to the success count, as the single-token form always
            // has. It also goes after the last target: in `4 + 4d10 + 1 t7`
            // the `+ 1` is per-die math before the target, but the 4 still
            // belongs on the success count. Only targets split math this way;
            // other systems (`mnm`, Conan) apply math to the total in written
            // order, so there the 4 stays in front of it. With only `f#`/`b#`
            // the roller drops math after them once there is math before them,
            // so that shape is rejected rather than losing the number.
            // `N/dice` must stay first: the roller reads it from there.
            let position = if operator == "/" {
                0
            } else {
                let after_target = if dice.modifiers.iter().any(Modifier::sets_success_count) {
                    dice.modifiers
                        .iter()
                        .rposition(Modifier::is_target)
                        .map_or(0, |i| i + 1)
                } else if dice
                    .modifiers
                    .iter()
                    .take_while(|m| !m.is_target())
                    .any(|m| modifier_kind(m) == ModifierKind::Math)
                    && dice.modifiers.iter().any(Modifier::is_target)
                {
                    return Err(anyhow!(
                        "A number in front of the dice can't be combined with math before \
                         `f#` or `b#` unless there is also a target (`t#`)"
                    ));
                } else {
                    0
                };
                dice.modifiers[after_target..]
                    .iter()
                    .position(|m| modifier_kind(m) == ModifierKind::Math)
                    .map_or(dice.modifiers.len(), |i| after_target + i)
            };
            dice.modifiers.splice(position..position, leading);

            return Ok(dice);
        }
    }

    // Standard parsing: first part should be dice
    parse_base_dice(&mut dice, &parts[0])?;

    // Parse modifiers
    parse_all_modifiers(&mut dice, &parts[1..])?;

    Ok(dice)
}

// Helper function to create default dice roll, eliminating duplication
fn create_default_dice_roll() -> DiceRoll {
    DiceRoll {
        count: 1,
        sides: 6,
        modifiers: Vec::new(),
        comment: None,
        label: None,
        private: false,
        simple: false,
        no_results: false,
        unsorted: false,
        original_expression: None,
    }
}

// Helper function to transfer metadata between dice rolls
fn transfer_dice_metadata(source: &DiceRoll, target: &mut DiceRoll) {
    target.private = source.private;
    target.simple = source.simple;
    target.no_results = source.no_results;
    target.unsorted = source.unsorted;
    target.comment = source.comment.clone();
    target.label = source.label.clone();
}

fn normalize_whitespace(input: &str) -> String {
    WHITESPACE_REGEX.replace_all(input.trim(), " ").to_string()
}

// Handle both spaced and combined expressions

fn parse_expression_to_parts(input: &str) -> Result<Vec<String>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    // Normalize whitespace first
    let normalized = normalize_whitespace(input);

    // Handle basic dice + modifiers case (like "1d6 t4f1ie6") before complex parsing
    if !normalized.contains(['+', '-', '*', '/']) && normalized.contains(' ') {
        let parts: Vec<String> = normalized
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if parts.len() == 2 && parts[0].contains('d') {
            return Ok(parts);
        }
    }

    // No advantage detection here: it misread roll sets. Advantage patterns are
    // handled only inside multi-operator expressions, below.

    // Special handling for expressions like "200 / 2d4" or "500 / d%" where number comes before dice
    if let Some(captures) = NUMBER_OP_DICE_REGEX.captures(&normalized) {
        let number = &captures[1];
        let operator = &captures[2];
        let dice_part = &captures[3];

        if operator == "/" {
            let dice_parts = parse_dice_and_additional_modifiers(dice_part)?;

            let mut result_parts = vec![
                dice_parts.dice_part,
                NUMBER_DIVIDED_BY_ROLL_OP.to_string(),
                number.to_string(),
            ];

            result_parts.extend(dice_parts.additional_modifiers);
            return Ok(result_parts);
        } else if dice_part.contains(|c: char| c.is_whitespace() || "+-*/".contains(c)) {
            // Tokenize the dice and anything after it the normal way, so that
            // `5 + 1d6 + 2` (or `5 + 1d6+2`) reaches the parser as separate
            // tokens, not with `1d6 + 2` as one unreadable token, which failed.
            let mut parts = vec![number.to_string(), operator.to_string()];
            parts.extend(parse_expression_to_parts(dice_part)?);
            return Ok(parts);
        } else {
            // A single dice token keeps its own modifiers ahead of the leading
            // number: `4 + 4d10t7` adds 4 to the success count, not to each
            // die. Splitting it would change that result.
            return Ok(vec![
                number.to_string(),
                operator.to_string(),
                dice_part.to_string(),
            ]);
        }
    }

    // Check if this is entirely without spaces and contains dice
    if !normalized.contains(' ') && normalized.contains('d') {
        return parse_combined_expression(&normalized);
    }

    // Check if this is a combined expression without spaces but with operators
    if !normalized.contains(' ')
        && normalized
            .chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/'))
    {
        return parse_combined_expression(&normalized);
    }

    // Handle mixed expressions (some spaces, some combined)
    let mut parts = Vec::new();
    let mut current_token = String::new();
    let mut chars = normalized.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' => {
                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '+' | '-' | '*' | '/' => {
                // Only treat + or - as part of dice expression if it's at the start AND followed by 'd'
                // AND we're in a complex multi-operator expression
                if current_token.is_empty()
                    && (ch == '+' || ch == '-')
                    && chars.peek() == Some(&'d')
                {
                    // Look ahead to see if this looks like a complex expression
                    let remaining_chars: String = chars.clone().collect();
                    if remaining_chars.contains(['+', '-', '*', '/']) {
                        // This is likely "+d20 + something" so treat +d20 as one token
                        current_token.push(ch);
                        continue;
                    }
                }

                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                parts.push(ch.to_string());
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        process_final_token(&mut parts, &current_token)?;
    }

    Ok(parts)
}

// A dice token split from the modifier tokens that follow it
struct DiceAndModifiers {
    dice_part: String,
    additional_modifiers: Vec<String>,
}

// Helper function to parse dice and additional modifiers
fn parse_dice_and_additional_modifiers(input: &str) -> Result<DiceAndModifiers> {
    // Look for the dice expression first
    if let Some(captures) = DICE_WITH_REMAINDER_REGEX.captures(input) {
        let dice_part = captures[1].to_string();
        let remaining = captures[2].trim();

        let additional_modifiers = if remaining.is_empty() {
            Vec::new()
        } else {
            // Parse the remaining modifiers
            parse_additional_modifiers(remaining)?
        };

        Ok(DiceAndModifiers {
            dice_part,
            additional_modifiers,
        })
    } else {
        // Fallback: treat the whole thing as a dice part
        Ok(DiceAndModifiers {
            dice_part: input.to_string(),
            additional_modifiers: Vec::new(),
        })
    }
}

// Helper function to parse additional modifiers after dice
fn parse_additional_modifiers(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current_token = String::new();

    for ch in input.chars() {
        match ch {
            '+' | '-' | '*' | '/' => {
                if !current_token.is_empty() {
                    parts.push(current_token.trim().to_string());
                    current_token.clear();
                }
                parts.push(ch.to_string());
            }
            ' ' => {
                if !current_token.is_empty() && current_token.trim() != "" {
                    parts.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        parts.push(current_token.trim().to_string());
    }

    Ok(parts)
}

// Helper function to process current token, reducing duplication
fn process_current_token(parts: &mut Vec<String>, current_token: &mut String) -> Result<()> {
    if current_token.contains('d') && has_attached_modifiers(current_token) {
        handle_combined_token_with_operator(parts, current_token)?;
    } else {
        parts.push(current_token.clone());
    }
    current_token.clear();
    Ok(())
}

// Helper function to handle combined tokens with trailing operators
fn handle_combined_token_with_operator(parts: &mut Vec<String>, current_token: &str) -> Result<()> {
    if current_token.ends_with(['+', '-', '*', '/']) {
        let op_pos = current_token.len() - 1;
        let main_part = &current_token[..op_pos];
        let op_part = &current_token[op_pos..];

        if main_part.contains('d') {
            let combined_parts = parse_combined_expression(main_part)?;
            parts.extend(combined_parts);
            parts.push(op_part.to_string());
        } else {
            parts.push(current_token.to_string());
        }
    } else {
        let combined_parts = parse_combined_expression(current_token)?;
        parts.extend(combined_parts);
    }
    Ok(())
}

// Helper function to process final token
fn process_final_token(parts: &mut Vec<String>, current_token: &str) -> Result<()> {
    if current_token.contains('d') && has_attached_modifiers(current_token) {
        handle_combined_token_with_operator(parts, current_token)?;
    } else {
        parts.push(current_token.to_string());
    }
    Ok(())
}

// Check if a dice expression has modifiers attached
fn has_attached_modifiers(input: &str) -> bool {
    // Check if it's just a simple dice expression
    if DICE_ONLY_REGEX.is_match(input) {
        return false;
    }

    // Check if it contains dice and additional characters that could be modifiers
    if let Some((_, modifiers_part)) = split_dice_and_modifiers(input) {
        !modifiers_part.is_empty()
    } else {
        false
    }
}

/// Fold `before_dice` (every token in front of the dice) into one number,
/// left to right: `10 + 2 +` gives 12 and `2 + 3 *` gives 5. The final token is
/// the operator joining the result to the dice, which the caller applies.
/// Anything that is not `number (op number)*` is rejected: it used to be
/// dropped silently (`100 / gb 3d8` rolled 3d8 + 100).
fn fold_leading_numbers(before_dice: &[String]) -> Result<i32> {
    let numbers = match before_dice {
        [number] => std::slice::from_ref(number),
        [numbers @ .., _operator] => numbers,
        [] => return Err(anyhow!("Expected a number before the dice")),
    };
    let parse = |token: &String| {
        token
            .parse::<i32>()
            .map_err(|_| unexpected_before_dice(before_dice))
    };

    let mut value = parse(&numbers[0])?;
    for pair in numbers[1..].chunks(2) {
        let [operator, operand] = pair else {
            return Err(unexpected_before_dice(before_dice));
        };
        let operand = parse(operand)?;
        value = match operator.as_str() {
            "+" => value.checked_add(operand),
            "-" => value.checked_sub(operand),
            "*" => value.checked_mul(operand),
            "/" if operand == 0 => return Err(anyhow!("Cannot divide by zero")),
            "/" => value.checked_div(operand),
            _ => return Err(unexpected_before_dice(before_dice)),
        }
        .ok_or_else(super::roller::result_too_large)?;
    }
    Ok(value)
}

fn unexpected_before_dice(before_dice: &[String]) -> anyhow::Error {
    anyhow!(
        "Unexpected `{}` between `{}` and the dice",
        before_dice[1..].join(" "),
        before_dice[0]
    )
}

// Parse combined expressions like "4d6e6k3+2" or "200/d%" or "100/2d1+5"
fn parse_combined_expression(input: &str) -> Result<Vec<String>> {
    // Check for number/dice pattern first (e.g., "200/2d4", "500/d%", "100/2d1+5")
    // But be more careful about capturing only the dice part, not trailing spaces/operators
    if let Some(captures) = NUMBER_SLASH_DICE_REGEX.captures(input) {
        let number = &captures[1];
        let dice_part = &captures[2];
        let trailing_part = captures[3].trim();

        // Check if the dice part actually contains dice (basic validation)
        if dice_part.contains('d') {
            // Parse the dice part and any modifiers attached to it
            let parsed = parse_dice_and_additional_modifiers(dice_part)?;

            // Create the base parts for division
            let mut parts = vec![
                parsed.dice_part,
                NUMBER_DIVIDED_BY_ROLL_OP.to_string(),
                number.to_string(), // The number to divide by dice result
            ];

            // Add any modifiers that were attached to the dice
            parts.extend(parsed.additional_modifiers);

            // Handle any trailing part (like " +5" in "100/2d1 +5")
            if !trailing_part.is_empty() {
                let trailing_modifiers = parse_additional_modifiers(trailing_part)?;
                parts.extend(trailing_modifiers);
            }

            return Ok(parts);
        }
    }

    let mut parts = Vec::new();
    let mut current_token = String::new();

    for ch in input.chars() {
        match ch {
            '+' | '-' | '*' | '/' => {
                // Mathematical operators - finish current token and add operator
                if !current_token.is_empty() {
                    parts.push(current_token.clone());
                    current_token.clear();
                }
                parts.push(ch.to_string());
            }
            _ => {
                // Regular character - add to current token
                current_token.push(ch);
            }
        }
    }

    // Add final token if any
    if !current_token.is_empty() {
        parts.push(current_token);
    }

    // If we didn't find any operators, try the old logic for dice with modifiers
    if parts.len() == 1 && parts[0].contains('d') {
        let input = parts[0].clone(); // Clone to avoid borrow checker issues
        parts.clear();

        // First, extract any mathematical operators at the end
        let (main_part, math_part) = split_math_operators(&input);

        // Parse the main dice + modifiers part
        if let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(main_part) {
            parts.push(dice_part);

            // Split the modifiers part into individual modifiers
            let modifier_parts = split_combined_modifiers(&modifiers_part)?;
            parts.extend(modifier_parts);
        } else {
            // Not a dice expression, treat as single part
            parts.push(main_part.to_string());
        }

        // Add any mathematical parts
        parts.extend(math_part);
    }

    Ok(parts)
}

// Split input into main part and mathematical operators
fn split_math_operators(input: &str) -> (&str, Vec<String>) {
    let mut split_pos = input.len();
    let mut math_parts = Vec::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = chars.len();

    // Work backwards to find mathematical operations
    while i > 0 {
        i -= 1;
        let ch = chars[i];

        match ch {
            '+' | '-' | '*' | '/' => {
                // Found operator, extract it and the operand that follows
                let op = ch.to_string();
                let start = i + 1;
                let mut end = start;

                // Find the complete operand (could be number or dice expression)
                while end < chars.len() {
                    let operand_char = chars[end];
                    if operand_char.is_ascii_digit() || operand_char == 'd' {
                        end += 1;
                    } else if operand_char.is_ascii_alphabetic() && end > start {
                        // Could be part of a modifier like "e6" in "1d4e4"
                        end += 1;
                    } else {
                        break;
                    }
                }

                if end > start {
                    let operand = chars[start..end].iter().collect::<String>();
                    math_parts.insert(0, operand);
                }
                math_parts.insert(0, op);
                split_pos = i;

                // Continue looking for more operators
                continue;
            }
            _ if ch.is_ascii_digit() || ch.is_ascii_alphabetic() => {
                // Part of an operand, continue
                continue;
            }
            _ => {
                // Not part of math operation, stop here
                break;
            }
        }
    }

    (&input[..split_pos], math_parts)
}

// Split dice expression from modifiers: "4d6e6k3" -> ("4d6", "e6k3")
fn split_dice_and_modifiers(input: &str) -> Option<(String, String)> {
    // Don't split D6 System expressions like "d6s5"
    // These should be treated as single modifiers, not dice + modifiers
    if input.starts_with("d6s") {
        return None; // Don't split D6 System expressions
    }

    // Match basic dice pattern
    if let Some(captures) = BASIC_DICE_SPLIT_REGEX.captures(input) {
        let dice_part = captures[1].to_string();
        let modifiers_part = captures[2].to_string();

        if modifiers_part.is_empty() {
            Some((dice_part, String::new()))
        } else {
            Some((dice_part, modifiers_part))
        }
    } else {
        None
    }
}

/// Modifier prefixes recognised by [`split_combined_modifiers`], in match order.
///
/// **Order is load-bearing.** Multi-character prefixes must be tried before the
/// single-character prefixes they shadow (`ie`/`irg`/`irx`/`ir` before `i`, `km`/`kl`
/// before `k`, `tl` before `t`, `rg`/`rx` before `r`, `esss?` before `e`, `cpd` before
/// `c`, `wwc` before `ww`), or a longer modifier is silently truncated into a
/// shorter one. Do not reorder.
static SPLIT_MODIFIER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"^(ie\d*)",                      // indefinite explode — ie, ie6 (BEFORE regular explode)
        r"^(adv\d+)",                     // advantage — adv1 (BEFORE alien)
        r"^(dis\d+)",                     // disadvantage — dis1 (BEFORE drop and d6s)
        r"^(irg\d+)",                     // indefinite reroll greater — irg5 (BEFORE rg)
        r"^(irx\d+)",                     // indefinite reroll exact — irx6 (BEFORE rx and i)
        r"^(ir\d+)",                      // indefinite reroll — ir1 (BEFORE regular reroll)
        r"^(i\d*)",                       // implode — i, i2 (AFTER ie/irg/irx/ir)
        r"^(tdhc?)",                      // darkest house — tdhc, tdh (BEFORE target and drop)
        r"^(km\d+)",                      // keep middle — km3 (BEFORE regular keep)
        r"^(kl\d+)",                      // keep low — kl2 (BEFORE regular keep)
        r"^(tl\d+)",                      // target lower — tl5 (BEFORE regular target)
        r"^(rg\d+)",                      // reroll greater — rg5
        r"^(rx\d+)",                      // reroll exact — rx6 (BEFORE regular reroll)
        r"^(k\d+)",                       // keep high — k3
        r"^(d\d+)",                       // drop — d1
        r"^(r\d+)",                       // reroll — r1
        r"^(t\d+)",                       // target — t4, t7
        r"^(favor\d+)",                   // Broken Empires Favor (BEFORE failure)
        r"^(f\d+)",                       // failure — f1
        r"^(esss?\d+)",                   // essence20 — ess4, esss4 (BEFORE explode)
        r"^(e\d*)",                       // explode — e, e6 (AFTER indefinite explode)
        r"^(b\d*)",                       // botch — b, b1
        r"^(cpd)",                        // cyberpunk red damage — cpd (BEFORE cancel's bare `c`)
        r"^(c)",                          // cancel — c
        r"^(wfrp\d+)",                    // warhammer fantasy — wfrp67 (BEFORE wng/wit/ww)
        r"^(tbe\d+(?:e\d+)?)",            // The Broken Empires — tbe65e4 (keeps e# as Expertise)
        r"^(wng\d*t?)",                   // wrath & glory — wng patterns
        r"^(gb|gbs)",                     // godbound — gb, gbs
        r"^(hs[nkh])",                    // hero system — hsn, hsk, hsh
        r"^(dh)",                         // dark heresy — dh
        r"^(fudge|df)",                   // fudge — fudge, df
        r"^(d6s\d+(?:\+\d+)?(?:\-\d+)?)", // d6 system — d6s patterns
        r"^(cpr)",                        // cyberpunk red — cpr
        r"^(wit)",                        // witcher — wit
        r"^(bnw)",                        // brave new world — bnw
        r"^(alien)",                      // alien base — alien (exact)
        r"^(aliens\d+)",                  // alien stress — aliens1, aliens2, etc.
        r"^(wwc\d+)",                     // wild words cut — wwc2, wwc3, etc. (before basic ww)
        r"^(ms\d+[ad]?|ms[ad]?|ms)",      // mothership
        r"^(ww)",                         // wild words — ww (basic)
        r"^(plot)",                       // plotweaver — plot
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("Failed to compile SPLIT_MODIFIER_PATTERNS entry"))
    .collect()
});

// Split combined modifiers like "e6k3r1" into ["e6", "k3", "r1"]
fn split_combined_modifiers(input: &str) -> Result<Vec<String>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    // Special handling for Mothership patterns - don't strip trailing +/-
    // because they're part of the modifier, not operators
    if input.starts_with("ms") {
        // This is a Mothership modifier, return it as-is
        return Ok(vec![input.to_string()]);
    }

    // First, check if there's a trailing operator and separate it
    let (modifiers_part, _trailing_op) = if input.ends_with(['+', '-', '*', '/']) {
        let op_pos = input.len() - 1;
        (&input[..op_pos], Some(&input[op_pos..]))
    } else {
        (input, None)
    };

    let mut modifiers = Vec::new();
    let mut remaining = modifiers_part;

    // Use progress tracking instead of arbitrary iteration limit
    let mut last_length = remaining.len();
    let mut no_progress_count = 0;
    const MAX_NO_PROGRESS: usize = 3;

    while !remaining.is_empty() {
        let mut found_match = false;

        for regex in SPLIT_MODIFIER_PATTERNS.iter() {
            if let Some(captures) = regex.captures(remaining)
                && let Some(matched) = captures.get(1)
            {
                let modifier = matched.as_str().to_string();
                modifiers.push(modifier.clone());

                // Move past this modifier
                remaining = &remaining[matched.end()..];
                found_match = true;

                break;
            }
        }

        if !found_match {
            return Err(anyhow!(
                "Unable to parse combined modifier: '{}' at position: '{}'. Expected valid modifier pattern.",
                input,
                remaining
            ));
        }

        // Progress tracking
        let current_length = remaining.len();
        if current_length == last_length {
            no_progress_count += 1;
            if no_progress_count > MAX_NO_PROGRESS {
                return Err(anyhow!(
                    "Infinite loop detected while parsing modifiers '{}' at position: '{}'",
                    input,
                    remaining
                ));
            }
        } else {
            no_progress_count = 0;
            last_length = current_length;
        }
    }

    if modifiers.is_empty() {
        return Err(anyhow!("No valid modifiers found in: {}", input));
    }

    Ok(modifiers)
}

/// True when `input`, or any `;`-separated part of it, starts with the `p`
/// flag: the same rule that marks a roll private. A roll that fails to parse
/// has no results to read the flag from, so its error uses this instead.
pub fn is_private_expression(input: &str) -> bool {
    input.split(';').any(|part| {
        let mut flags = create_default_dice_roll();
        parse_flags(&mut flags, part);
        flags.private
    })
}

fn parse_flags<'a>(dice: &mut DiceRoll, mut remaining: &'a str) -> &'a str {
    let flags = ["p", "s", "nr", "ul"];

    let mut changed = true;
    while changed {
        changed = false;
        remaining = remaining.trim_start();

        for &flag in &flags {
            if remaining.starts_with(flag) {
                // Check if it's a complete flag (followed by space or end of string)
                let after_flag = &remaining[flag.len()..];
                if after_flag.is_empty()
                    || after_flag.starts_with(' ')
                    || after_flag.starts_with('\t')
                {
                    match flag {
                        "p" => dice.private = true,
                        "s" => dice.simple = true,
                        "nr" => dice.no_results = true,
                        "ul" => dice.unsorted = true,
                        _ => {}
                    }
                    remaining = remaining[flag.len()..].trim_start();
                    changed = true;
                    break;
                }
            }
        }
    }

    remaining
}

fn parse_label<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if let Some(captures) = LABEL_REGEX.captures(remaining) {
        let label_content = &captures[1];
        let trimmed = label_content.trim();
        dice.label = Some(trimmed.to_string());
        return captures
            .get(0)
            .map_or(remaining, |label| &remaining[label.end()..]);
    }
    remaining
}

fn parse_comment<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if remaining.trim() == "!" {
        dice.comment = Some("".to_string());
        return "";
    }

    if let Some(captures) = COMMENT_REGEX.captures(remaining) {
        let comment_content = captures[1].trim();

        // Handle semicolon in comments - take everything before first semicolon
        if comment_content.contains(';') {
            if let Some(first_part) = comment_content.split(';').next() {
                let trimmed = first_part.trim();
                dice.comment = Some(trimmed.to_string());
            } else {
                dice.comment = Some("".to_string());
            }
        } else {
            dice.comment = Some(comment_content.to_string());
        }

        captures
            .get(0)
            .map_or(remaining, |comment| remaining[..comment.start()].trim())
    } else {
        remaining
    }
}

fn parse_base_dice(dice: &mut DiceRoll, part: &str) -> Result<()> {
    // Try alias expansion first (this handles +d20, -d20, etc.)
    if let Some(expanded) = super::aliases::expand_alias(part) {
        let expanded_dice = parse_single_dice_expression(&expanded)?;
        dice.count = expanded_dice.count;
        dice.sides = expanded_dice.sides;
        dice.modifiers.extend(expanded_dice.modifiers);
        return Ok(());
    }

    // Check if this is a simple dice expression
    if DICE_ONLY_REGEX.is_match(part) {
        return parse_simple_dice_part(dice, part);
    }

    // If it's not a simple dice expression, it might be a combined expression
    // that needs to be split further
    if part.contains('d')
        && let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(part)
    {
        // Parse the dice part
        parse_simple_dice_part(dice, &dice_part)?;

        // Parse the modifiers part if any
        parse_modifiers_from_part(dice, &modifiers_part)?;
        return Ok(());
    }

    Err(anyhow!("Invalid dice expression: {}", part))
}

// Helper function to parse modifiers from a part, reducing duplication
fn parse_modifiers_from_part(dice: &mut DiceRoll, modifiers_part: &str) -> Result<()> {
    if !modifiers_part.is_empty() {
        let modifier_parts = split_combined_modifiers(modifiers_part)?;
        for modifier_part in modifier_parts {
            let modifier = parse_single_modifier(&modifier_part)?;
            dice.modifiers.push(modifier);
        }
    }
    Ok(())
}

fn parse_simple_dice_part(dice: &mut DiceRoll, part: &str) -> Result<()> {
    let (count, sides) = parse_dice_term(part)?;
    dice.count = count;
    dice.sides = sides;
    Ok(())
}

// Every `NdS` term, main roll or `+ 2d6` operand, goes through these limits. Operands once
// skipped them: `1d6 + 1d4000000000` panicked the roller and `99999999999d6` rolled one die.
// DICE_ONLY_REGEX guarantees digits, so a failed parse can only mean the number overflowed.
fn parse_dice_term(part: &str) -> Result<(u32, u32)> {
    let captures = DICE_ONLY_REGEX
        .captures(part)
        .ok_or_else(|| anyhow!("Invalid dice expression: {}", part))?;

    let count = match captures.get(1) {
        Some(m) => m
            .as_str()
            .parse()
            .map_err(|_| anyhow!("Maximum {MAX_DICE} dice allowed"))?,
        None => 1,
    };
    let sides = if &captures[2] == "%" {
        100
    } else {
        captures[2]
            .parse()
            .map_err(|_| anyhow!("Maximum {MAX_SIDES} sides allowed"))?
    };

    if count == 0 {
        return Err(anyhow!("Cannot roll 0 dice"));
    }
    if count > MAX_DICE {
        return Err(anyhow!("Maximum {MAX_DICE} dice allowed"));
    }
    if sides < 1 {
        return Err(anyhow!("Dice must have at least 1 side"));
    }
    if sides > MAX_SIDES {
        return Err(anyhow!("Maximum {MAX_SIDES} sides allowed"));
    }

    Ok((count, sides))
}

fn parse_all_modifiers(dice: &mut DiceRoll, parts: &[String]) -> Result<()> {
    let mut i = 0;
    while i < parts.len() {
        let part = &parts[i];

        // Try parsing operator + operand pairs
        if i + 1 < parts.len()
            && let Some(consumed) = try_parse_operator_pair(dice, &parts[i], &parts[i + 1])?
        {
            i += consumed;

            // Check if we just created an AddDice and if following tokens are modifiers
            if consumed == 2 && parts[i - 2] == "+" && parts[i - 1].contains('d') {
                // Check if following tokens are modifiers for the dice we just added
                let mut modifier_tokens = Vec::new();
                let mut j = i;
                while j < parts.len() && is_modifier_start(&parts[j]) {
                    modifier_tokens.push(parts[j].clone());
                    j += 1;
                }

                if !modifier_tokens.is_empty() {
                    // Handle combined modifiers properly
                    if let Some(Modifier::AddDice(dice_roll)) = dice.modifiers.last_mut() {
                        for token in &modifier_tokens {
                            // Check if this token is combined modifiers
                            if is_combined_modifiers_token(token) {
                                // Split combined modifiers and add each one
                                let combined_parts = split_combined_modifiers(token)?;
                                for combined_part in combined_parts {
                                    let modifier = parse_single_modifier(&combined_part)?;
                                    dice_roll.modifiers.push(modifier);
                                }
                            } else {
                                // Single modifier
                                let modifier = parse_single_modifier(token)?;
                                dice_roll.modifiers.push(modifier);
                            }
                        }
                    }
                    i += modifier_tokens.len(); // Skip the modifier tokens we just processed
                }
            }
            continue;
        }

        // Check if this part is combined modifiers before parsing as single
        if is_combined_modifiers_token(part) {
            let modifier_parts = split_combined_modifiers(part)?;

            for modifier_part in modifier_parts {
                let modifier = parse_single_modifier(&modifier_part)?;
                dice.modifiers.push(modifier);
            }
        } else {
            // Parse single modifier
            let modifier = parse_single_modifier(part)?;
            dice.modifiers.push(modifier);
        }
        i += 1;
    }
    Ok(())
}

/// Modifier prefixes recognised by [`is_combined_modifiers_token`], in match order.
///
/// **Order is load-bearing.** Multi-character prefixes (`ie`, `adv`, `dis`, `irg`,
/// `irx`, `ir`, `km`, `kl`, `tl`, `rg`, `rx`) must be tried before the single-character prefixes
/// they shadow (`i`, `e`, `k`, `t`, `r`, `d`), or a longer modifier is silently
/// truncated into a shorter one. Do not reorder.
static COMBINED_MODIFIER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"^(ie\d*)",    // Indefinite explode first (longer pattern)
        r"^(adv\d+)",   // Advantage (before explode)
        r"^(dis\d+)",   // Disadvantage (before drop)
        r"^(irg\d+)",   // Indefinite reroll greater
        r"^(irx\d+)",   // Indefinite reroll exact
        r"^(ir\d+)",    // Indefinite reroll
        r"^(i\d*)",     // Implode (AFTER ie/irg/irx/ir)
        r"^(km\d+)",    // Keep middle
        r"^(kl\d+)",    // Keep low
        r"^(tl\d+)",    // Target lower (must come before regular target)
        r"^(rg\d+)",    // Reroll greater
        r"^(rx\d+)",    // Reroll exact
        r"^(e\d*)",     // Explode
        r"^(k\d+)",     // Keep high
        r"^(d\d+)",     // Drop
        r"^(r\d+)",     // Reroll
        r"^(t\d+)",     // Target - KEY FOR D6 LEGENDS
        r"^(favor\d+)", // Broken Empires Favor (before failure)
        r"^(f\d+)",     // Failure - KEY FOR D6 LEGENDS
        r"^(b\d*)",     // Botch
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("Failed to compile COMBINED_MODIFIER_PATTERNS entry"))
    .collect()
});

fn is_combined_modifiers_token(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    // Handle D6 system expressions specially - don't treat as combined modifiers
    if input.contains('d') {
        // Don't treat standalone dice expressions as combined modifiers
        if is_standalone_dice_expression(input) {
            return false;
        }

        // Don't treat D6 system expressions as combined modifiers
        if input.starts_with("d6s") {
            return false;
        }
    }

    // Check if it starts with common modifier patterns and has more after the first one
    for regex in COMBINED_MODIFIER_PATTERNS.iter() {
        if let Some(captures) = regex.captures(input) {
            let first_modifier = &captures[1];
            let match_length = first_modifier.len();

            // Safety check for zero-length matches
            if match_length == 0 {
                continue;
            }

            // If the match is the entire string, it's a single modifier, not combined
            if match_length == input.len() {
                return false;
            }

            // If there's more after the first modifier, check if it looks like more modifiers
            let remaining = &input[match_length..];
            let result = is_modifier_start(remaining);

            return result;
        }
    }

    false
}

/// Modifier prefixes recognised by [`starts_with_modifier_pattern`], in match order.
///
/// **Order is load-bearing** for the same reason as [`SPLIT_MODIFIER_PATTERNS`]:
/// longer prefixes must precede the shorter ones they shadow. Do not reorder.
static MODIFIER_START_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"^ie\d*",    // Indefinite explode: ie, ie6
        r"^adv\d+",   // Advantage: adv1 (before explode)
        r"^dis\d+",   // Disadvantage: dis1 (before drop)
        r"^irg\d+",   // Indefinite reroll greater: irg5
        r"^irx\d+",   // Indefinite reroll exact: irx6
        r"^ir\d+",    // Indefinite reroll: ir1
        r"^i\d*",     // Implode: i, i2 (AFTER ie/irg/irx/ir)
        r"^km\d+",    // Keep middle: km3
        r"^kl\d+",    // Keep low: kl2
        r"^tl\d+",    // Target lower: tl5
        r"^tn\d+",    // Conan 2d20 target number: tn12, tn12f3c19 (before t)
        r"^rg\d+",    // Reroll greater: rg5
        r"^rx\d+",    // Reroll exact: rx6
        r"^k\d+",     // Keep high: k3
        r"^d\d+",     // Drop: d1
        r"^r\d+",     // Reroll: r1
        r"^t\d+",     // Target: t4, t7 - CRITICAL FOR D6 LEGENDS
        r"^favor\d+", // Broken Empires Favor (before failure)
        r"^f\d+",     // Failure: f1 - CRITICAL FOR D6 LEGENDS
        r"^esss?\d+", // Essence20 skill die: ess4, esss4 (before explode)
        r"^e\d*",     // Explode: e, e6
        r"^b\d*",     // Botch: b, b1
        r"^c$",       // Cancel: c (exact match)
        // System modifiers
        r"^wfrp\d+",         // Warhammer Fantasy 4e: wfrp67
        r"^tbe\d+(?:e\d+)?", // The Broken Empires: tbe65e4 (e# is Expertise)
        r"^wng",             // Wrath & Glory patterns
        r"^gb$",             // Godbound (exact)
        r"^gbs$",            // Godbound straight (exact)
        r"^hs[nkh]",         // Hero System
        r"^dh$",             // Dark Heresy (exact)
        r"^fudge$",          // Fudge (exact)
        r"^df$",             // Fudge dice (exact)
        r"^d6s\d+",          // D6 System
        r"^cpr$",            // Cyberpunk Red (exact)
        r"^cpd$",            // Cyberpunk Red damage (exact)
        r"^wit$",            // Witcher (exact)
        r"^alien$",          // Alien base modifier (exact)
        r"^aliens\d+",       // Alien stress modifiers: aliens1, aliens2, etc.
        r"^fitd$",           // Forged in the Dark (exact)
        r"^fitd0$",          // FitD zero dice (exact)
        r"^plot$",           // Plotweaver/Cosmere RPG plot die (exact)
        r"^tdhc?$",          // The Darkest House die: tdh, tdhc (exact)
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("Failed to compile MODIFIER_START_PATTERNS entry"))
    .collect()
});

fn is_modifier_start(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    // Special case: Don't treat single letters as modifiers if they could be part of game system aliases
    if input.len() == 1 {
        // Cancel and implode are the only single-letter modifiers
        return matches!(input, "c" | "i");
    }

    // Check if it starts with any known modifier pattern (not requiring it to be complete)

    // Check if the input starts with any of these patterns
    MODIFIER_START_PATTERNS
        .iter()
        .any(|regex| regex.is_match(input))
}

fn try_parse_operator_pair(
    dice: &mut DiceRoll,
    first: &str,
    second: &str,
) -> Result<Option<usize>> {
    match first {
        NUMBER_DIVIDED_BY_ROLL_OP => {
            let num = second
                .parse::<i32>()
                .map_err(|_| anyhow!("Number too large to divide by dice: {second}"))?;
            dice.modifiers.push(Modifier::NumberDividedByRoll(num));
            return Ok(Some(2));
        }
        "+" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Add(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::AddDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // This might be a complex dice expression like "1d4e4" or just "d10"
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::AddDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "-" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Subtract(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::SubtractDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // This might be a complex dice expression like "1d4e4" or just "d10"
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::SubtractDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "*" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Multiply(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                // Handle dice multiplication
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::MultiplyDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // Handle complex dice expressions for multiplication
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::MultiplyDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "/" => {
            if let Ok(num) = second.parse::<i32>() {
                if num == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                dice.modifiers.push(Modifier::Divide(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                // Handle dice division
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::DivideDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // Handle complex dice expressions for division
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::DivideDice(dice_roll));
                return Ok(Some(2));
            }
        }
        _ => {}
    }

    Ok(None)
}

// Parse complex dice expressions that might have modifiers attached
fn parse_complex_dice_expression(input: &str) -> Result<DiceRoll> {
    if is_dice_expression(input) {
        // Simple dice expression
        return parse_dice_expression_only(input);
    }

    // Complex expression with modifiers - parse as a mini dice expression
    let mut dice = create_default_dice_roll();

    if let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(input) {
        // Parse the dice part
        parse_simple_dice_part(&mut dice, &dice_part)?;

        // Parse the modifiers part if any
        parse_modifiers_from_part(&mut dice, &modifiers_part)?;
        Ok(dice)
    } else {
        Err(anyhow!("Invalid complex dice expression: {}", input))
    }
}

// Edge/trouble count for `mme<N>`/`mmt<N>`. A bare `mme` is one edge, as `mm e` is
// and as the rules define an edge; it used to mean none. Each count is a reroll loop in
// the roller, so it shares the dice cap: `mme2147483647` would have hung a worker, and an
// overflowing count silently became 0.
fn parse_marvel_count(digits: &str, part: &str, what: &str) -> Result<i32> {
    if digits.is_empty() {
        return Ok(1);
    }
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(anyhow!("Invalid Marvel Multiverse modifier '{}'", part));
    }
    match digits.parse::<u32>() {
        Ok(count) if count <= MAX_DICE => Ok(count as i32),
        _ => Err(anyhow!("Maximum {MAX_DICE} {what} allowed")),
    }
}

// Shared level parsing for the advantage/disadvantage modifiers. The upper
// bound mirrors the 500 dice cap, since each level rolls one extra die.
fn parse_advantage_level(digits: &str, part: &str) -> Result<u32> {
    let level: u32 = digits
        .parse()
        .map_err(|_| anyhow!("Invalid advantage/disadvantage level in '{}'", part))?;

    if level > MAX_DICE {
        return Err(anyhow!(
            "Maximum advantage/disadvantage level is {MAX_DICE}"
        ));
    }

    Ok(level)
}

// Essence20 skill dice are carried as a rank number into `ESSENCE20_RANKS`
// (1 = d2 … 8 = 3d6) so the modifier token never contains a `d`.
fn parse_essence20_rank(digits: &str, part: &str, specialization: bool) -> Result<Modifier> {
    let rank: u32 = digits
        .parse()
        .map_err(|_| anyhow!("Invalid Essence20 skill rank in '{}'", part))?;

    let highest_rank = ESSENCE20_RANKS.len() as u32;
    if !(1..=highest_rank).contains(&rank) {
        return Err(anyhow!(
            "Essence20 skill rank must be 1-{highest_rank}, got {rank}"
        ));
    }

    Ok(Modifier::Essence20(rank, specialization))
}

/// Build a [`Modifier::TwoD20Test`] from a captured `tn#[f#][c#]` token.
///
/// Omitted `f#` gives a Focus of 0 (no criticals) and omitted `c#` gives a
/// Complication range of 20 only, which is an ordinary trained Conan test. An
/// untrained test widens the range to 19-20, written `c19`.
fn parse_two_d20_test(captures: &regex::Captures<'_>) -> Result<Modifier> {
    let number = |index: usize, default: u32| -> Result<u32> {
        match captures.get(index) {
            Some(matched) => matched
                .as_str()
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid number in target number modifier")),
            None => Ok(default),
        }
    };

    let target = number(1, 0)?;
    let focus = number(2, 0)?;
    let complication = number(3, 20)?;

    if !(1..=20).contains(&target) {
        return Err(anyhow!("Target number must be 1-20, got {}", target));
    }
    if focus > CONAN_MAX_FOCUS {
        return Err(anyhow!(
            "Focus must be 0-{}, got {}",
            CONAN_MAX_FOCUS,
            focus
        ));
    }
    if focus > target {
        return Err(anyhow!(
            "Focus {} cannot exceed target number {}",
            focus,
            target
        ));
    }
    if !(2..=20).contains(&complication) {
        return Err(anyhow!(
            "Complication range must be 2-20, got {}",
            complication
        ));
    }

    Ok(Modifier::TwoD20Test {
        target,
        focus,
        complication,
    })
}

fn parse_single_modifier(part: &str) -> Result<Modifier> {
    // Reject standalone 'l' - it should only appear in d6l aliases
    if part == "l" {
        return Err(anyhow!("Unrecognized modifier pattern: 'l' in 'l'"));
    }

    // Advantage/disadvantage before anything that could claim the 'd' or 'a'
    // prefix (drop, d6s, standalone dice expressions, alien)
    if let Some(stripped) = part.strip_prefix("adv") {
        return Ok(Modifier::Advantage(parse_advantage_level(stripped, part)?));
    }

    if let Some(stripped) = part.strip_prefix("dis") {
        return Ok(Modifier::Disadvantage(parse_advantage_level(
            stripped, part,
        )?));
    }

    // Essence20 skill dice must be matched before the single-character `e`
    // explode prefix would claim them.  `esss#` (specialization) first, since
    // `ess#` is a prefix of it.
    if let Some(stripped) = part.strip_prefix("esss") {
        return parse_essence20_rank(stripped, part, true);
    }

    if let Some(stripped) = part.strip_prefix("ess") {
        return parse_essence20_rank(stripped, part, false);
    }

    // Check for Alien RPG modifiers first
    if part == "alien" {
        return Ok(Modifier::Alien);
    }

    if let Some(stripped) = part.strip_prefix("aliens") {
        let stress_level: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid stress level in Alien stress modifier '{}'", part))?;

        if stress_level == 0 {
            return Err(anyhow!("Alien stress level cannot be 0"));
        }
        if stress_level > 10 {
            return Err(anyhow!("Alien stress level cannot exceed 10"));
        }

        return Ok(Modifier::AlienStress(stress_level));
    }

    // Forged in the Dark modifiers
    if part == "fitd" {
        return Ok(Modifier::ForgedDark);
    }

    if part == "fitd0" {
        return Ok(Modifier::ForgedDarkZero);
    }

    // System modifiers
    match part {
        "dh" => return Ok(Modifier::DarkHeresy),
        "dheart" => return Ok(Modifier::Daggerheart),
        "hsn" => return Ok(Modifier::HeroSystem(HeroSystemType::Normal)),
        "hsk" => return Ok(Modifier::HeroSystem(HeroSystemType::Killing)),
        "hsh" => return Ok(Modifier::HeroSystem(HeroSystemType::Hit)),
        "fudge" | "df" => return Ok(Modifier::Fudge),
        "gb" => return Ok(Modifier::Godbound(false)),
        "gbs" => return Ok(Modifier::Godbound(true)),
        "cpr" => return Ok(Modifier::CyberpunkRed),
        "cpd" => return Ok(Modifier::CyberpunkRedDamage),
        "wit" => return Ok(Modifier::Witcher),
        "bnw" => return Ok(Modifier::BraveNewWorld(0)),
        "mnm" => return Ok(Modifier::MutantsMasterminds),
        "c" => return Ok(Modifier::Cancel),
        "ww" => return Ok(Modifier::WildWords(None)),
        "plot" => return Ok(Modifier::PlotDie),
        "tdh" => return Ok(Modifier::DarkestHouse(false)),
        "tdhc" => return Ok(Modifier::DarkestHouse(true)),
        _ => {}
    }

    // Cypher System handling (cs1, cs3, cs10, etc.)
    if let Some(stripped) = part.strip_prefix("cs") {
        let level = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Cypher System level in '{}'", part))?;
        if !(1..=10).contains(&level) {
            return Err(anyhow!(
                "Cypher System difficulty level must be 1-10, got {}",
                level
            ));
        }
        return Ok(Modifier::CypherSystem(level));
    }

    // Warhammer Fantasy 4e target: wfrp67. The alias folds difficulty modifiers
    // into this number, so a target above 100 is legitimate here even though a
    // bare Characteristic or Skill never is.
    if let Some(stripped) = part.strip_prefix("wfrp")
        && !stripped.is_empty()
        && stripped.chars().all(|c| c.is_ascii_digit())
    {
        let target: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid WFRP target in '{}'", part))?;
        if target > WFRP_MAX_TARGET {
            return Err(anyhow!(
                "Maximum WFRP target is {WFRP_MAX_TARGET}, got {target}"
            ));
        }
        return Ok(Modifier::Wfrp(target));
    }

    if let Some(stripped) = part.strip_prefix("tbe")
        && !stripped.is_empty()
    {
        let (skill_text, expertise_text) = stripped
            .split_once('e')
            .map_or((stripped, None), |(skill, expertise)| {
                (skill, Some(expertise))
            });
        if !skill_text.chars().all(|c| c.is_ascii_digit())
            || expertise_text
                .is_some_and(|value| value.is_empty() || !value.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(anyhow!("Invalid Broken Empires skill in '{}'", part));
        }

        let skill: u32 = skill_text
            .parse()
            .map_err(|_| anyhow!("Invalid Broken Empires skill in '{}'", part))?;
        if skill == 0 {
            return Err(anyhow!("Broken Empires skill must be greater than 0"));
        }
        let expertise: u32 = expertise_text
            .unwrap_or("0")
            .parse()
            .map_err(|_| anyhow!("Invalid Broken Empires Expertise in '{}'", part))?;
        if expertise_text.is_some() && expertise == 0 {
            return Err(anyhow!("Broken Empires Expertise must be positive"));
        }
        if expertise > i32::MAX as u32 {
            return Err(anyhow!("Broken Empires Expertise is too large"));
        }
        return Ok(Modifier::BrokenEmpires(skill, expertise));
    }

    if let Some(stripped) = part.strip_prefix("favor") {
        let points: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Broken Empires Favor in '{}'", part))?;
        if !(1..=3).contains(&points) {
            return Err(anyhow!(
                "Broken Empires Favor spent per roll must be 1-3, got {points}"
            ));
        }
        return Ok(Modifier::BrokenEmpiresFavor(points));
    }

    // Conan / 2d20 target number: tn12, tn12f3, tn12f3c19
    if let Some(captures) = TWO_D20_REGEX.captures(part) {
        return parse_two_d20_test(&captures);
    }

    // Conan skill roll handling (conan, conan1 through conan9)
    if part == "conan" {
        return Ok(Modifier::ConanSkill(2)); // Default 2d20
    }
    if let Some(stripped) = part.strip_prefix("conan")
        && let Ok(dice_count) = stripped.parse::<u32>()
    {
        if (1..=CONAN_MAX_POOL).contains(&dice_count) {
            return Ok(Modifier::ConanSkill(dice_count));
        } else {
            return Err(anyhow!(
                "Conan skill rolls support 1-{} dice, got {}",
                CONAN_MAX_POOL,
                dice_count
            ));
        }
    }

    // Conan combat dice handling (cd, cd4, cd10, etc.)
    if part == "cd" {
        return Ok(Modifier::ConanCombat(1)); // Default 1d6
    }
    if let Some(stripped) = part.strip_prefix("cd")
        && let Ok(dice_count) = stripped.parse::<u32>()
    {
        if dice_count > 0 && dice_count <= 100 {
            // Reasonable limit
            return Ok(Modifier::ConanCombat(dice_count));
        } else {
            return Err(anyhow!(
                "Conan combat dice count must be 1-100, got {}",
                dice_count
            ));
        }
    }

    if is_multi_sided_dice_expression(part) {
        // This is additional dice that should be added to the roll
        let dice_roll = parse_dice_expression_only(part)?;
        return Ok(Modifier::AddDice(dice_roll));
    }

    // Check for invalid characters before parsing numbers
    if part.contains(['+', '-', '*', '/']) {
        return Err(anyhow!("Invalid modifier '{}' - contains operator", part));
    }

    // Handle exploding dice
    if let Some(stripped) = part.strip_prefix("ie") {
        return Ok(Modifier::ExplodeIndefinite(parse_explode_threshold(
            stripped, part,
        )?));
    }

    if let Some(stripped) = part.strip_prefix('e') {
        return Ok(Modifier::Explode(parse_explode_threshold(stripped, part)?));
    }

    // Handle D6 System BEFORE drop modifier to avoid conflicts
    // D6 System handling (Handle BEFORE drop modifier since both start with 'd')
    // Pips (`d6s5+2`) never arrive here: the operator check above rejects a part
    // holding `+`/`-`, and tokenizing has already split them off as Add/Subtract.
    if let Some(stripped) = part.strip_prefix("d6s") {
        let count: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid D6 System count in '{}'", part))?;
        // The roller rolls `count` dice in one go; uncapped, `d6s4294967295` hung a worker.
        if count > MAX_DICE {
            return Err(anyhow!("Maximum {MAX_DICE} dice allowed"));
        }

        return Ok(Modifier::D6System(count));
    }

    // Handle drop modifier (AFTER D6 System to avoid conflicts)
    if let Some(stripped) = part.strip_prefix('d') {
        // Make sure this isn't a D6 System expression that somehow got through
        if stripped.starts_with("6s") {
            return Err(anyhow!(
                "D6 System expressions should be handled earlier: {}",
                part
            ));
        }

        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid drop value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot drop 0 dice"));
        }
        return Ok(Modifier::Drop(num));
    }

    // Check indefinite reroll greater (irg) BEFORE regular reroll greater (rg)
    if let Some(stripped) = part.strip_prefix("irg") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid indefinite reroll greater value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollGreaterIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix("rg") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll greater value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollGreater(num));
    }

    // Exact rerolls (irx, rx) BEFORE `ir` and `r`, which would reject the `x`
    if let Some(stripped) = part.strip_prefix("irx") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid indefinite reroll exact value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll 0s - no die shows 0"));
        }
        return Ok(Modifier::RerollExactIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix("rx") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll exact value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll 0s - no die shows 0"));
        }
        return Ok(Modifier::RerollExact(num));
    }

    // Continue with other modifiers...
    if let Some(stripped) = part.strip_prefix("ir") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('r') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::Reroll(num));
    }

    // Imploding dice - MUST stay after `ie`, `irg`, `irx` and `ir`, which all start
    // with the same 'i' prefix and would otherwise be claimed here.
    if let Some(stripped) = part.strip_prefix('i') {
        let num = if stripped.is_empty() {
            None
        } else {
            Some(
                stripped
                    .parse()
                    .map_err(|_| anyhow!("Invalid implode value in '{}'", part))?,
            )
        };
        if let Some(val) = num
            && val == 0
        {
            return Err(anyhow!("Cannot implode on 0"));
        }
        return Ok(Modifier::Implode(num));
    }

    if let Some(stripped) = part.strip_prefix("km") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep middle value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepMiddle(num));
    }

    if let Some(stripped) = part.strip_prefix("kl") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep low value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepLow(num));
    }

    if let Some(stripped) = part.strip_prefix('k') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepHigh(num));
    }

    // Target with doubles t{num}ds{num} must be checked BEFORE other targets
    if let Some(captures) = TARGET_DS_SIDES_REGEX.captures(part) {
        let target = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid target value in TargetWithDoubleSuccess '{}'", part))?;
        let double_value = captures[2]
            .parse()
            .map_err(|_| anyhow!("Invalid double value in TargetWithDoubleSuccess '{}'", part))?;

        // Validation for new syntax only
        if target == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }
        if double_value == 0 {
            return Err(anyhow!("Double success must be greater than 0"));
        }
        if double_value < target {
            return Err(anyhow!(
                "Double success ({}) must be >= target value ({})",
                double_value,
                target
            ));
        }

        return Ok(Modifier::TargetWithDoubleSuccess(target, double_value));
    }

    // Check for t{target}ds pattern (default double success = target)
    if let Some(captures) = TARGET_DS_REGEX.captures(part) {
        let target = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid target value in TargetWithDoubleSuccess '{}'", part))?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }

        // Default: double success value = target value
        return Ok(Modifier::TargetWithDoubleSuccess(target, target));
    }

    // Parse target lower with double success (explicit)
    if let Some(captures) = TARGET_LOWER_DS_SIDES_REGEX.captures(part) {
        let target = captures[1].parse().map_err(|_| {
            anyhow!(
                "Invalid target value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;
        let double_value = captures[2].parse().map_err(|_| {
            anyhow!(
                "Invalid double success value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }
        if double_value == 0 {
            return Err(anyhow!("Double success value must be greater than 0"));
        }
        if double_value > target {
            return Err(anyhow!(
                "Double success value ({}) must be <= target lower value ({})",
                double_value,
                target
            ));
        }

        return Ok(Modifier::TargetLowerWithDoubleSuccess(target, double_value));
    }

    // Parse target lower with double success (default)
    if let Some(captures) = TARGET_LOWER_DS_REGEX.captures(part) {
        let target = captures[1].parse().map_err(|_| {
            anyhow!(
                "Invalid target value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }

        // Default: double success value = target value
        return Ok(Modifier::TargetLowerWithDoubleSuccess(target, target));
    }

    // Target Lower (tl) must be checked BEFORE Target (t) to avoid conflicts
    if let Some(stripped) = part.strip_prefix("tl") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid target lower value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }
        return Ok(Modifier::TargetLower(num));
    }

    if let Some(stripped) = part.strip_prefix('t') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid target value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }
        return Ok(Modifier::Target(num));
    }

    if let Some(stripped) = part.strip_prefix('f') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid failure value in '{}'", part))?;
        return Ok(Modifier::Failure(num));
    }

    if let Some(stripped) = part.strip_prefix('b') {
        let num = if stripped.is_empty() {
            None
        } else {
            Some(
                stripped
                    .parse()
                    .map_err(|_| anyhow!("Invalid botch value in '{}'", part))?,
            )
        };
        return Ok(Modifier::Botch(num));
    }

    // Wrath & Glory handling
    if let Some(stripped) = part.strip_prefix("wng") {
        if stripped.is_empty() {
            return Ok(Modifier::WrathGlory(None, false, 1)); // Default 1 wrath die
        }

        // Wrath dice count (`w2`), then difficulty (`dn3`) and total mode (`t`).
        // The count was read char by char and dropped whatever single character
        // followed it, so `wngw1t` lost its `t` and `wngw1x` passed as `w1`.
        let mut remaining = stripped;
        let mut wrath_count = 1u32;
        if let Some(w_stripped) = remaining.strip_prefix('w') {
            let digits = w_stripped.bytes().take_while(u8::is_ascii_digit).count();
            let (count_str, rest) = w_stripped.split_at(digits);
            remaining = rest;
            if !count_str.is_empty() {
                wrath_count = count_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid wrath dice count in '{}'", part))?;
            }
        }

        let (difficulty, use_total) = parse_wrath_glory_mode(remaining)
            .ok_or_else(|| anyhow!("Invalid Wrath & Glory modifier: {}", part))?;

        // 0 for rolls without a Wrath Die: damage, Determination and
        // Initiative ("This roll does not include a Wrath Die", p.178).
        if wrath_count > 5 {
            return Err(anyhow!("Wrath dice count must be 0-5, got {wrath_count}"));
        }
        return Ok(Modifier::WrathGlory(difficulty, use_total, wrath_count));
    }

    // Savage Worlds handling (AFTER D6 System but before other modifiers)
    if let Some(stripped) = part.strip_prefix("sw") {
        let sides = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Savage Worlds trait die in '{}'", part))?;
        if !(4..=12).contains(&sides) || sides % 2 != 0 {
            return Err(anyhow!(
                "Savage Worlds trait die must be d4, d6, d8, d10, or d12"
            ));
        }
        return Ok(Modifier::SavageWorlds(sides));
    }

    // Shadowrun handling
    if let Some(stripped) = part.strip_prefix("shadowrun") {
        let dice_count = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Shadowrun dice count in '{}'", part))?;
        if dice_count == 0 {
            return Err(anyhow!("Shadowrun dice count must be greater than 0"));
        }
        return Ok(Modifier::Shadowrun(dice_count));
    }

    // Wild Words RPG cutting mechanics
    if let Some(stripped) = part.strip_prefix("wwc") {
        let cut_count: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid cut count in Wild Words modifier '{}'", part))?;

        if cut_count == 0 {
            return Err(anyhow!("Cannot cut 0 dice in Wild Words"));
        }

        return Ok(Modifier::WildWords(Some(cut_count)));
    }

    if let Ok(num) = part.parse::<i32>() {
        return Ok(Modifier::Add(num));
    }

    // Marvel Multiverse handling
    if part == "mm" {
        return Ok(Modifier::MarvelMultiverse(0, 0));
    }

    if let Some(stripped) = part.strip_prefix("mme") {
        let edges = parse_marvel_count(stripped, part, "edges")?;
        return Ok(Modifier::MarvelMultiverse(edges, 0));
    }

    if let Some(stripped) = part.strip_prefix("mmt") {
        let troubles = parse_marvel_count(stripped, part, "troubles")?;
        return Ok(Modifier::MarvelMultiverse(0, troubles));
    }

    if let Some(stripped) = part.strip_prefix("sil") {
        if stripped.is_empty() {
            return Ok(Modifier::Silhouette(1));
        } else {
            let dice_count = stripped
                .parse()
                .map_err(|_| anyhow!("Invalid Silhouette dice count in '{}'", part))?;
            if dice_count == 0 || dice_count > 10 {
                return Err(anyhow!(
                    "Silhouette dice count must be 1-10, got {}",
                    dice_count
                ));
            }
            return Ok(Modifier::Silhouette(dice_count));
        }
    }

    // Mothership RPG handling
    if part == "ms" {
        return Ok(Modifier::Mothership(None, false));
    }
    // `msa`/`msd` are what `+ms`/`-ms` expand to: a letter, since a trailing
    // `+`/`-` would read as an operator
    if part == "msa" {
        return Ok(Modifier::Mothership(None, true));
    }
    if part == "msd" {
        return Ok(Modifier::Mothership(None, false));
    }

    // Mothership with stat value
    if let Some(stripped) = part.strip_prefix("ms") {
        if let Some(stat_str) = stripped.strip_suffix('a') {
            return Ok(Modifier::Mothership(
                Some(parse_mothership_stat(stat_str, part)?),
                true,
            ));
        } else if let Some(stat_str) = stripped.strip_suffix('d') {
            return Ok(Modifier::Mothership(
                Some(parse_mothership_stat(stat_str, part)?),
                false,
            ));
        } else if !stripped.is_empty() {
            // Just a stat value
            return Ok(Modifier::Mothership(
                Some(parse_mothership_stat(stripped, part)?),
                false,
            ));
        }
    }

    // VTM5 - Vampire: The Masquerade 5th Edition
    if let Some(stripped) = part.strip_prefix("vtm5p") {
        // Parse pattern like "vtm5p7h2" (pool=7, hunger=2)
        if let Some(h_pos) = stripped.find('h') {
            let pool_str = &stripped[..h_pos];
            let hunger_str = &stripped[h_pos + 1..];

            let pool_size = pool_str
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid VTM5 pool size in '{}'", part))?;
            let hunger_dice = hunger_str
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid VTM5 hunger dice count in '{}'", part))?;

            // Validate ranges
            if pool_size == 0 || pool_size > 30 {
                return Err(anyhow!("VTM5 pool size must be 1-30, got {}", pool_size));
            }
            if hunger_dice > pool_size {
                return Err(anyhow!(
                    "VTM5 hunger dice ({}) cannot exceed pool size ({})",
                    hunger_dice,
                    pool_size
                ));
            }
            if hunger_dice > 5 {
                return Err(anyhow!(
                    "VTM5 hunger dice cannot exceed 5, got {}",
                    hunger_dice
                ));
            }

            return Ok(Modifier::VampireMasquerade5(pool_size, hunger_dice));
        }
    }

    // Lasers & Feelings handling (lf2, lf2l, lf2f, etc.)
    if let Some(stripped) = part.strip_prefix("lf") {
        // Parse target and optional type (lf4l, lf4f, lf4)
        let chars: Vec<char> = stripped.chars().collect();

        if chars.is_empty() {
            return Err(anyhow!("Invalid Lasers & Feelings format: '{}'", part));
        }

        // Find where the number ends and type begins
        let mut number_end = 0;
        for (i, ch) in chars.iter().enumerate() {
            if ch.is_ascii_digit() {
                number_end = i + 1;
            } else {
                break;
            }
        }

        if number_end == 0 {
            return Err(anyhow!("Invalid Lasers & Feelings target in '{}'", part));
        }

        let target_str: String = chars[..number_end].iter().collect();
        let type_str: String = chars[number_end..].iter().collect();

        let target = target_str
            .parse()
            .map_err(|_| anyhow!("Invalid Lasers & Feelings target in '{}'", part))?;

        // Validate target number (2-5 as per L&F rules)
        if !(2..=5).contains(&target) {
            return Err(anyhow!(
                "Lasers & Feelings target must be between 2 and 5, got {} in '{}'",
                target,
                part
            ));
        }

        // Determine roll type
        let roll_type = match type_str.as_str() {
            "l" => LaserFeelingsType::Lasers,
            "f" => LaserFeelingsType::Feelings,
            // A bare `2lf4` rolls as Lasers, as roll_syntax.md documents
            "" => LaserFeelingsType::Lasers,
            _ => {
                return Err(anyhow!(
                    "Invalid Lasers & Feelings type '{}' in '{}'",
                    type_str,
                    part
                ));
            }
        };

        // The roller takes the dice count from the roll itself; the first field is unused
        return Ok(Modifier::LaserFeelings(0, target, roll_type));
    }

    // Wild Words RPG parsing
    if part == "ww" {
        return Ok(Modifier::WildWords(None));
    }

    if let Some(cut_match) = WWC_REGEX.captures(part) {
        let cut_count: u32 = cut_match[1]
            .parse()
            .map_err(|_| anyhow!("Invalid cut count in Wild Words modifier"))?;

        if cut_count == 0 {
            return Err(anyhow!("Cannot cut 0 dice in Wild Words"));
        }

        return Ok(Modifier::WildWords(Some(cut_count)));
    }

    Err(anyhow!("Unknown modifier: {}", part))
}

fn is_dice_expression(input: &str) -> bool {
    // Basic dice patterns, `d10` and `d%` included (the count is optional)
    if DICE_ONLY_REGEX.is_match(input) {
        return true;
    }

    // Check for advantage/disadvantage patterns
    if input == "+d20" || input == "-d20" || input == "+d%" || input == "-d%" {
        return true;
    }

    false
}

// `d10` needs no rewrite to `1d10`: parse_dice_term treats a missing count as 1.
fn parse_dice_expression_only(input: &str) -> Result<DiceRoll> {
    let (count, sides) = parse_dice_term(input)?;
    Ok(DiceRoll {
        count,
        sides,
        ..create_default_dice_roll()
    })
}

static MULTI_DIGIT_COUNT_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{2,})d(\d+)$").expect("Failed to compile MULTI_DIGIT_COUNT_DICE_REGEX")
});

static MULTI_DIGIT_SIDES_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d)d(\d{2,})$").expect("Failed to compile MULTI_DIGIT_SIDES_DICE_REGEX")
});

static COMMON_DIE_SIZE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)d([46]|8|10|12|20|100)$").expect("Failed to compile COMMON_DIE_SIZE_REGEX")
});

fn is_multi_sided_dice_expression(part: &str) -> bool {
    // Only detect dice expressions that are clearly additional dice, not modifiers

    // Pattern 1: Multi-digit count + d + sides (like "3d6", "2d10", "10d6")
    // 2+ digit count is clearly dice, not a drop modifier
    if MULTI_DIGIT_COUNT_DICE_REGEX.is_match(part) {
        return true;
    }

    // Pattern 2: Single digit count + d + multi-digit sides (like "1d20", "3d100")
    // Multi-digit sides is clearly dice, not a drop modifier
    if MULTI_DIGIT_SIDES_DICE_REGEX.is_match(part) {
        return true;
    }

    // Pattern 3: Common multi-die patterns that aren't drop modifiers
    if let Some(caps) = COMMON_DIE_SIZE_REGEX.captures(part)
        && let (Ok(count), Ok(sides)) = (caps[1].parse::<u32>(), caps[2].parse::<u32>())
    {
        // If count > 1, it's clearly additional dice, not a drop modifier
        if count > 1 {
            return true;
        }
        // For count=1, check sides - d4, d6, d8, d10, d12, d20, d100 are likely dice
        // Exclude d1, d2, d3 which are likely drop modifiers
        if sides >= 4 {
            return true;
        }
    }

    // Pattern 4: Percentile dice (d% patterns)
    if part == "d%" || part.ends_with("d%") {
        return true;
    }

    false // Default to not a dice expression - let normal modifier parsing handle it
}

/// Patterns for dice expressions that stand alone rather than being combined
/// modifiers. Order is not significant here — the check is a plain "any match".
static STANDALONE_DICE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"^\d*d\d+$", // Basic dice like "3d6", "d20", "1d4"
        r"^\d*d%$",   // Percentile dice like "d%", "2d%"
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("Failed to compile STANDALONE_DICE_PATTERNS entry"))
    .collect()
});

fn is_standalone_dice_expression(input: &str) -> bool {
    // Check for patterns that represent standalone dice expressions
    // These should NOT be treated as combined modifiers
    STANDALONE_DICE_PATTERNS
        .iter()
        .any(|regex| regex.is_match(input))
}

fn create_roll_set_with_metadata(
    captures: &regex::Captures,
    metadata: Option<&DiceRoll>,
) -> Result<Vec<DiceRoll>> {
    let count: u32 = captures[1]
        .parse()
        .map_err(|_| anyhow!("Invalid set count"))?;

    // VALIDATION: Centralized in one place
    if !(2..=20).contains(&count) {
        return Err(anyhow!("Set count must be between 2 and 20"));
    }

    // Parse the dice expression
    let mut dice = parse_single_dice_expression(&expand_roll_set_expression(&captures[2]))?;

    // Transfer metadata if provided (for flag support)
    if let Some(meta) = metadata {
        transfer_dice_metadata(meta, &mut dice);
    }

    Ok(build_roll_set(&dice, count))
}
