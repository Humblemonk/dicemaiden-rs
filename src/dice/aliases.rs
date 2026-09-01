//! Game-system alias expansion: short user-facing syntax → canonical dice expression.
//!
//! [`expand_alias`] is called by the parser before any other processing.  If
//! the raw input matches a known alias pattern it is rewritten into a standard
//! `NdS [modifiers]` expression (or a multi-roll expression for systems that
//! need it) and parsing continues normally.
//!
//! # Supported aliases (summary)
//!
//! | Prefix / pattern | System                               |
//! |------------------|--------------------------------------|
//! | `alien`          | Alien RPG (Year Zero Engine)         |
//! | `+d` / `-d`      | Advantage / disadvantage (d20, etc.) |
//! | `Nwod` / `Ncod`  | World of Darkness / Chronicles       |
//! | `wng`            | Warhammer: Age of Sigmar – Soulbound |
//! | `sw`             | Savage Worlds trait dice             |
//! | `gb` / `gbs`     | Godbound damage                      |
//! | `df`             | Fate / Fudge dice                    |
//! | `dh`             | Dark Heresy                          |
//! | `sr`             | Shadowrun                            |
//! | `sp`             | Splittermond                         |
//! | `yz`             | Year Zero Engine (generic)           |
//! | `snm`            | Stars Without Number                 |
//! | `d6s`            | D6 System                            |
//! | `hs`             | Hero System                          |
//! | `ex`             | Exalted                              |
//! | `ms` / `ms2`     | Mothership RPG                       |
//! | `ol5` / `ol5a2`  | Open Legend RPG                      |
//! | `ola` / `old`    | Open Legend RPG                      |
//! | `ess1d8`         | Essence20 (Renegade Game Studios)    |
//! | `tdh`            | The Darkest House (Monte Cook Games) |
//! | `cpr` / `cpd`    | Cyberpunk Red skill check / damage    |
//! | `wfrp`           | Warhammer Fantasy Roleplay 4e         |
//!
//! See `roll_syntax.md` for the full syntax reference.  All regex patterns are
//! compiled once at startup via `once_cell::Lazy`.
//!
//! # Adding a new alias
//!
//! 1. Check `roll_syntax.md` — the new prefix must not be a prefix of, or
//!    prefixed by, any existing identifier (prefix-matching conflicts are silent
//!    and hard to debug).
//! 2. Add a `Lazy<Regex>` constant near the top of this file.
//! 3. Add a match arm in `expand_alias`.
//! 4. Document the syntax in `roll_syntax.md`.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

// Pre-compile all regex patterns at startup to reduce memory allocations
static GB_DICE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(gbs?)\s+(\d+)d(\d+)(?:\s*([+-]\s*\d+))?$")
        .expect("Failed to compile GB_DICE_REGEX")
});

static GB_SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(gbs?)(?:\s*([+-]\s*\d+))?$").expect("Failed to compile GB_SIMPLE_REGEX")
});

static WNG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^wng(?:\s+w(\d+))?(?:\s+dn(\d+))?\s+(\d+)d(\d+)(?:\s*!\s*(\w+))?$")
        .expect("Failed to compile WNG_REGEX")
});

static SW_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^sw(\d+)$").expect("Failed to compile SW_REGEX"));

static WNG_SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^wng(?:\s+w(\d+))?\s+(\d+)d(\d+)$").expect("Failed to compile WNG_SIMPLE_REGEX")
});

static COD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)cod([89r]?)(?:\s*([+-]\s*\d+))?$").expect("Failed to compile COD_REGEX")
});

static WOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)wod(\d+)(?:\s*([+-]\s*\d+))?$").expect("Failed to compile WOD_REGEX")
});

static WOD_CANCEL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)wod(\d+)c(?:\s*([+-]\s*\d+))?$").expect("Failed to compile WOD_CANCEL_REGEX")
});

static DH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^dh\s+(\d+)d(\d+)$").expect("Failed to compile DH_REGEX"));

static DF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)df$").expect("Failed to compile DF_REGEX"));

static WH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)wh(\d+)\+$").expect("Failed to compile WH_REGEX"));

static DD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^dd(\d)(\d)$").expect("Failed to compile DD_REGEX"));

static ADV_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+-])d(\d+)$").expect("Failed to compile ADV_REGEX"));

static PERC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)d%$").expect("Failed to compile PERC_REGEX"));

static SR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^sr(\d+)$").expect("Failed to compile SR_REGEX"));

static SP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^sp(\d+)(?:t(\d+))?$").expect("Failed to compile SP_REGEX"));

static YZ_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)yz$").expect("Failed to compile YZ_REGEX"));

static SNM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^snm(\d+)$").expect("Failed to compile SNM_REGEX"));

static D6S_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^d6s(\d+)(\s*\+\s*\d+)?$").expect("Failed to compile D6S_REGEX"));

static HS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+(?:\.\d+)?)hs([nkh])$").expect("Failed to compile HS_REGEX"));

static HS_FRAC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)hs([nkh])(\d+)$").expect("Failed to compile HS_FRAC_REGEX"));

static EX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)(?:t(\d+))?$").expect("Failed to compile EX_REGEX"));

static OL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^ol(\d{1,2})(?:\s*([ad])(\d+))?$").expect("Failed to compile OL_REGEX")
});

// Exalted with custom target and double success: ex5t8ds10 → 5d10 t8ds10
static EX_T_DS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)t(\d+)ds(\d+)$").expect("Failed to compile EX_T_DS_REGEX"));

// Custom target + double (short form): ex5t8d9 → 5d10 t8ds9
static EX_T_D_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)t(\d+)d(\d+)$").expect("Failed to compile EX_T_D_REGEX"));

// Exalted with double success short form: ex7d8 → 7d10 t7ds8
static EX_D_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)d(\d+)$").expect("Failed to compile EX_D_REGEX"));

static EX_DS_FULL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)d(\d+)t(\d+)$").expect("Failed to compile EX_DS_FULL_REGEX"));

static EX_DS_SIMPLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ex(\d+)ds(\d+)$").expect("Failed to compile EX_DS_SIMPLE_REGEX"));

static ED_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ed(\d+)$").expect("Failed to compile ED_REGEX"));

static ED4E_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ed4e(\d+)$").expect("Failed to compile ED4E_REGEX"));

static DND_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(attack|skill|save)(\s*[+-]\s*\d+)?$").expect("Failed to compile DND_REGEX")
});

static MM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^mm(?:\s+(\d*)([et])(?:\s+(\d*)([et]))?)?$").expect("Failed to compile MM_REGEX")
});

static CPR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^cpr(?:\s*([+-]\s*\d+))?$").expect("Failed to compile CPR_REGEX"));

// Cyberpunk Red damage: cpd3 -> 3d6 cpd. Distinct from `cpr` (the 1d10 skill
// check) at the third character, so neither is a prefix of the other.
static CPD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^cpd(\d+)(?:\s*([+-]\s*\d+))?$").expect("Failed to compile CPD_REGEX")
});

static WIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^wit(?:\s*([+-]\s*\d+))?$").expect("Failed to compile WIT_REGEX"));

// Warhammer Fantasy Roleplay 4e: wfrp67 -> 1d100 wfrp67. Difficulty modifiers
// adjust the target rather than the roll (WFRP is roll-under), so `wfrp67 + 20`
// folds into a target of 87 at expansion time rather than reaching the total.
static WFRP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^wfrp(\d+)(?:\s*([+-])\s*(\d+))?$").expect("Failed to compile WFRP_REGEX")
});

static CS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^cs\s+(\d+)(?:\s*([+-]\s*\d+))?$").expect("Failed to compile CS_REGEX")
});

static BNW_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^bnw(\d+)$").expect("Failed to compile BNW_REGEX"));

static CONAN_SKILL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^conan([1-9])$").expect("Failed to compile CONAN_SKILL_REGEX"));

// Packed skill test: conan tn12f3 written as one token (conan2tn12f3 = 2d20 tn12f3)
static CONAN_TN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^conan([1-9])?(tn\d+(?:f\d+)?(?:c\d+)?)$")
        .expect("Failed to compile CONAN_TN_REGEX")
});

static CONAN_COMBAT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^cd(\d+)$").expect("Failed to compile CONAN_COMBAT_REGEX"));

// Combined attack patterns (conan3cd4 = 3d20 skill + 4d6 combat)
static CONAN_COMBINED_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^conan([1-9])cd(\d+)$").expect("Failed to compile CONAN_COMBINED_REGEX")
});

static SIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^sil(\d+)$").expect("Failed to compile SIL_REGEX"));

static D6L_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)d6l$").expect("Failed to compile D6L_REGEX"));

static VTM5_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^vtm(\d+)h(\d+)$").expect("Failed to compile VTM5_REGEX"));

static LF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)lf(\d+)([lf]?)$").expect("Failed to compile LF_REGEX"));

// Daggerheart RPG system
static DAGGERHEART_PLAYER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^dheart$").expect("Failed to compile DAGGERHEART_PLAYER_REGEX"));

static DAGGERHEART_GM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^dheartgm$").expect("Failed to compile DAGGERHEART_GM_REGEX"));

static A5E_BASIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)a5e\s*([+-]?\d+)?\s*ex([1-3]|4|6|8|10|12|20|100)$")
        .expect("Failed to compile A5E_BASIC_REGEX")
});

static A5E_ADV_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)\+a5e\s*([+-]?\d+)?\s*ex([1-3]|4|6|8|10|12|20|100)$")
        .expect("Failed to compile A5E_ADV_REGEX")
});

static A5E_DIS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)-a5e\s*([+-]?\d+)?\s*ex([1-3]|4|6|8|10|12|20|100)$")
        .expect("Failed to compile A5E_DIS_REGEX")
});

static ALIEN_BASIC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?i)alien(\d+)$").expect("Failed to compile ALIEN_BASIC_REGEX"));

static ALIEN_STRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)alien(\d+)s(\d+)$").expect("Failed to compile ALIEN_STRESS_REGEX")
});

static ALIEN_PUSH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)alien(\d+)s(\d+)p$").expect("Failed to compile ALIEN_PUSH_REGEX")
});

static FITD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?i)fitd(\d+)$").expect("Failed to compile FITD_REGEX"));

static WILD_WORLDS_BASIC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ww(\d+)$").expect("Failed to compile WILD_WORLDS_BASIC_REGEX"));

static WILD_WORLDS_CUT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^ww(\d+)c(\d+)$").expect("Failed to compile WILD_WORLDS_CUT_REGEX"));

static MNM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^mnm(?:\s*([+-]\s*\d+))?$").expect("Failed to compile MNM_REGEX"));

static MS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+-])?ms(\d+)?$").expect("Failed to compile MS_REGEX"));

static DP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)dp$").expect("Failed to compile DP_REGEX"));

// Essence20: optional Edge/Snag sign, skill die, optional specialization `s`,
// optional flat modifier — `ess1d8`, `essd8`, `ess2d8s`, `+ess3d6`, `-ess1d10 +2`
static ESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-])?ess(\d*)d(\d+)(s)?(?:\s*([+-]\s*\d+))?$")
        .expect("Failed to compile ESS_REGEX")
});

// The Darkest House: rating, optional Boon (`b`) / Bane (`n`), optional
// "call upon the house" (`c`), optional flat modifier —
// `tdh4`, `tdh4b`, `tdh4n`, `tdh4c`, `tdh4bc`, `tdh4 +1`
static TDH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^tdh(\d+)([bn])?(c)?(?:\s*([+-]\s*\d+))?$").expect("Failed to compile TDH_REGEX")
});

// Use static storage for commonly used alias mappings
static STATIC_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut aliases = HashMap::new();
    aliases.insert("age", "2d6 + 1d6");
    aliases.insert("dndstats", "6 4d6 k3");
    aliases.insert("attack", "1d20");
    aliases.insert("skill", "1d20");
    aliases.insert("save", "1d20");
    aliases.insert("gb", "1d20 gb");
    aliases.insert("gbs", "1d20 gbs");
    aliases.insert("hsn", "1d6 hsn");
    aliases.insert("hsk", "1d6 hsk");
    aliases.insert("hsh", "3d6 hsh");
    aliases.insert("3df", "3d3 fudge");
    aliases.insert("4df", "4d3 fudge");
    aliases.insert("dh", "1d10 dh");
    aliases.insert("wng", "1d6 wng");
    aliases.insert("conan", "2d20 conan");
    aliases.insert("cd", "1d6 cd");
    aliases.insert("sil", "1d6 sil1");
    aliases.insert("fitd1", "1d6 fitd");
    aliases.insert("fitd2", "2d6 fitd");
    aliases.insert("fitd3", "3d6 fitd");
    aliases.insert("fitd4", "4d6 fitd");
    aliases.insert("fitd5", "5d6 fitd");
    aliases.insert("fitd6", "6d6 fitd");
    aliases.insert("fitd0", "2d6 fitd0"); // Zero dice - special case
    aliases.insert("ww1", "1d6 ww");
    aliases.insert("ww2", "2d6 ww");
    aliases.insert("ww3", "3d6 ww");
    aliases.insert("ww4", "4d6 ww");
    aliases.insert("ww5", "5d6 ww");
    aliases.insert("ww6", "6d6 ww");
    aliases.insert("ww7", "7d6 ww");
    aliases.insert("ww8", "8d6 ww");
    aliases.insert("ww9", "9d6 ww");
    aliases.insert("ww10", "10d6 ww");
    aliases.insert("ww4c1", "4d6 wwc1");
    aliases.insert("ww4c2", "4d6 wwc2");
    aliases.insert("ww5c1", "5d6 wwc1");
    aliases.insert("ww5c2", "5d6 wwc2");
    aliases.insert("ww6c1", "6d6 wwc1");
    aliases.insert("ww6c2", "6d6 wwc2");
    aliases.insert("ww6c3", "6d6 wwc3");
    aliases.insert("dp", "1d6 plot");
    aliases
});

pub fn expand_alias(input: &str) -> Option<String> {
    let input = input.trim().to_lowercase();

    // Handle Mothership advantage/disadvantage patterns FIRST (before parameterized)
    // This matches +ms, -ms, +ms45, -ms45, etc.
    if let Some(result) = expand_mothership_alias(&input) {
        return Some(result);
    }

    // Handle parameterized aliases
    if let Some(result) = expand_parameterized_alias(&input) {
        return Some(result);
    }

    // Check static aliases
    if let Some(&static_result) = STATIC_ALIASES.get(input.as_str()) {
        return Some(static_result.to_string());
    }

    None
}

// Helper function to reduce duplication in Hero System dice processing
fn process_hero_system_dice(
    dice_count_str: &str,
    _damage_type: &str,
    dice_type: &str,
) -> Option<String> {
    if let Ok(dice_count) = dice_count_str.parse::<f64>() {
        let whole_dice = dice_count.floor() as u32;
        let has_fractional = dice_count.fract() > 0.0;

        if whole_dice == 0 && has_fractional {
            return Some(format!("1d3 {dice_type}"));
        }

        let dice_expr = if has_fractional {
            format!("{whole_dice}d6 {dice_type} + 1d3")
        } else {
            format!("{whole_dice}d6 {dice_type}")
        };
        return Some(dice_expr);
    }
    None
}

// Helper function to reduce duplication in Exalted double-success alias processing.
// `target` is None for the short forms that use the default success target of 7.
// Output format is intentionally unchanged: {count}d10 t{target}ds{double}
fn process_exalted_pool(count: &str, target: Option<&str>, double: &str) -> Option<String> {
    // Validate dice count
    match count.parse::<u32>() {
        Ok(0) | Err(_) => return None,
        Ok(_) => {}
    }

    // Validate target
    let target_num = match target {
        Some(target) => match target.parse::<u32>() {
            Ok(num) if num > 0 && num <= 10 => num,
            _ => return None,
        },
        None => 7,
    };

    // Validate double (must be >= target)
    let double_num = match double.parse::<u32>() {
        Ok(num) if num > 0 && num <= 10 && num >= target_num => num,
        _ => return None,
    };

    Some(format!("{count}d10 t{target_num}ds{double_num}"))
}

fn expand_parameterized_alias(input: &str) -> Option<String> {
    // Handle Alien RPG aliases first
    if let Some(alien_result) = expand_alien_alias(input) {
        return Some(alien_result);
    }

    // Handle Daggerheart RPG aliases
    if DAGGERHEART_PLAYER_REGEX.is_match(input) {
        return Some("2d12 dheart".to_string());
    }

    if DAGGERHEART_GM_REGEX.is_match(input) {
        return Some("1d20".to_string());
    }

    // Handle percentile advantage/disadvantage FIRST before general advantage/disadvantage
    if input == "+d%" {
        return Some("2d10 kl1 * 10 + 1d10 - 10".to_string());
    }
    if input == "-d%" {
        return Some("2d10 k1 * 10 + 1d10 - 10".to_string());
    }

    // Handle standalone wng and dh first
    if input == "wng" {
        return Some("1d6 wng".to_string());
    }

    if input == "dh" {
        return Some("1d10 dh".to_string());
    }

    // Handle Mutants & Masterminds aliases
    if let Some(captures) = MNM_REGEX.captures(input) {
        let modifier = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        if modifier.is_empty() {
            return Some("1d20 mnm".to_string());
        } else {
            // Clean up the modifier string (remove extra spaces)
            let clean_modifier = modifier.replace(' ', "");
            return Some(format!("1d20{} mnm", clean_modifier));
        }
    }

    // Check for A5E (Level Up Advanced 5th Edition) aliases
    if let Some(a5e_result) = expand_a5e_alias(input) {
        return Some(a5e_result);
    }

    // Handle Wild Words RPG aliases
    if let Some(wild_words_result) = expand_wild_words_alias(input) {
        return Some(wild_words_result);
    }

    // Handle Open Legend RPG aliases
    if let Some(open_legend_result) = expand_open_legend_alias(input) {
        return Some(open_legend_result);
    }

    // Handle Essence20 aliases (before `ex`/`ed`, which share no prefix but sit
    // in the same alphabetical neighbourhood)
    if let Some(essence20_result) = expand_essence20_alias(input) {
        return Some(essence20_result);
    }

    // Handle The Darkest House aliases
    if let Some(darkest_house_result) = expand_darkest_house_alias(input) {
        return Some(darkest_house_result);
    }

    if let Some(captures) = SR_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d6 t5 shadowrun{count}"));
    }

    // Handle Hero System fractional dice properly
    if let Some(captures) = HS_REGEX.captures(input) {
        let dice_count_str = &captures[1];
        let damage_type = &captures[2];

        match damage_type {
            "n" => {
                // Normal damage - XdY
                return process_hero_system_dice(dice_count_str, damage_type, "hsn");
            }
            "k" => {
                // Killing damage - XdY
                return process_hero_system_dice(dice_count_str, damage_type, "hsk");
            }
            "h" => {
                // To hit roll - always 3d6 regardless of the number (Hero System standard)
                return Some("3d6 hsh".to_string());
            }
            _ => return None,
        }
    }

    // Godbound system - full dice expressions (gb 3d8, gbs 2d10, etc.)
    if let Some(captures) = GB_DICE_REGEX.captures(input) {
        let gb_type = &captures[1];
        let count = &captures[2];
        let sides = &captures[3];
        let modifier = captures.get(4).map(|m| m.as_str().trim()).unwrap_or("");

        return Some(format!("{count}d{sides} {gb_type}{modifier}"));
    }

    // Godbound system - simple modifiers (gb+5, gbs-2, etc.)
    if let Some(captures) = GB_SIMPLE_REGEX.captures(input) {
        let gb_type = &captures[1];
        let modifier = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        return Some(format!("1d20 {gb_type}{modifier}"));
    }

    // Wrath & Glory (wng 4d6, wng dn2 4d6, wng 4d6 !soak)
    if let Some(captures) = WNG_REGEX.captures(input) {
        let wrath_count = captures.get(1).map(|m| m.as_str()).unwrap_or("1");
        let difficulty = captures.get(2).map(|m| m.as_str());
        let count = &captures[3];
        let sides = &captures[4];
        let special = captures.get(5).map(|m| m.as_str());

        return Some(match (difficulty, special) {
            (Some(dn), Some("soak") | Some("exempt") | Some("dmg")) => {
                format!("{count}d{sides} wngw{wrath_count}dn{dn}t")
            }
            (Some(dn), _) => {
                format!("{count}d{sides} wngw{wrath_count}dn{dn}")
            }
            (None, Some("soak") | Some("exempt") | Some("dmg")) => {
                format!("{count}d{sides} wngw{wrath_count}t")
            }
            (None, _) => {
                format!("{count}d{sides} wngw{wrath_count}")
            }
        });
    }

    // Simple wng pattern (wng 4d6)
    if let Some(captures) = WNG_SIMPLE_REGEX.captures(input) {
        let wrath_count = captures.get(1).map(|m| m.as_str()).unwrap_or("1");
        let count = &captures[2];
        let sides = &captures[3];
        return Some(format!("{count}d{sides} wngw{wrath_count}"));
    }

    // Chronicles of Darkness (4cod -> 4d10 t8 ie10)
    if let Some(captures) = COD_REGEX.captures(input) {
        let count = &captures[1];
        let variant = captures.get(2).map_or("", |m| m.as_str());
        let modifier = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        let modifier_part = if modifier.is_empty() {
            String::new()
        } else {
            format!(" {modifier}")
        };

        return Some(match variant {
            "8" => format!("{count}d10 t8 ie8{modifier_part}"), // 8-again
            "9" => format!("{count}d10 t8 ie9{modifier_part}"), // 9-again
            "r" => format!("{count}d10 t8 ie10 r7{modifier_part}"), // rote quality
            _ => format!("{count}d10 t8 ie10{modifier_part}"),  // standard
        });
    }

    // World of Darkness with cancel (4wod8c -> 4d10 f1 t8 c)
    if let Some(captures) = WOD_CANCEL_REGEX.captures(input) {
        return process_wod_regex_captures(
            &captures,
            "{count}d10 f1 t{difficulty} c",
            "{count}d10 f1 t{difficulty} c {modifier}",
        );
    }

    // World of Darkness (4wod8 -> 4d10 f1 ie10 t8)
    if let Some(captures) = WOD_REGEX.captures(input) {
        return process_wod_regex_captures(
            &captures,
            "{count}d10 f1 t{difficulty}",
            "{count}d10 f1 t{difficulty} {modifier}",
        );
    }

    // Dark Heresy (dh 4d10 -> 4d10 ie10)
    if let Some(captures) = DH_REGEX.captures(input) {
        let count = &captures[1];
        let sides = &captures[2];
        return Some(format!("{count}d{sides} ie{sides} dh"));
    }

    // Fudge dice (3df -> 3d3 fudge)
    if let Some(captures) = DF_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d3 fudge"));
    }

    // Warhammer (3wh4+ -> 3d6 t4)
    if let Some(captures) = WH_REGEX.captures(input) {
        let count = &captures[1];
        let target = &captures[2];
        return Some(format!("{count}d6 t{target}"));
    }

    // Double digit (dd34 -> (1d3 * 10) + 1d4)
    if let Some(captures) = DD_REGEX.captures(input) {
        let tens = &captures[1];
        let ones = &captures[2];
        return Some(format!("1d{tens} * 10 + 1d{ones}"));
    }

    // General advantage/disadvantage (+d20, -d20, etc.) but NOT +d% or -d%
    if let Some(captures) = ADV_REGEX.captures(input) {
        let modifier = &captures[1];
        let sides = &captures[2];

        return Some(match modifier {
            "+" => format!("2d{sides} k1"),  // advantage
            "-" => format!("2d{sides} kl1"), // disadvantage
            _ => return None,
        });
    }

    // Simple percentile (xd% -> xd100)
    if let Some(captures) = PERC_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d100"));
    }

    // Shadowrun (sr6 -> 6d6 t5)
    if let Some(captures) = SR_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d6 t5"));
    }

    // Storypath (sp4 -> 4d10 t8 ie10, sp4t7 -> 4d10 t7 ie10)
    if let Some(captures) = SP_REGEX.captures(input) {
        let count = &captures[1];
        let target = captures.get(2).map_or("8", |m| m.as_str());
        return Some(format!("{count}d10 t{target} ie10"));
    }

    // Year Zero (6yz -> 6d6 t6)
    if let Some(captures) = YZ_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d6 t6"));
    }

    // Sunsails New Millennium (snm5 -> 5d6 ie6 t4)
    if let Some(captures) = SNM_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d6 ie6 t4"));
    }

    // D6 System (d6s4 -> use custom modifier instead of parsing)
    if let Some(captures) = D6S_REGEX.captures(input) {
        let count = &captures[1];
        let pips = captures.get(2).map_or("", |m| m.as_str());

        // Use a dummy roll that triggers the D6System modifier
        return Some(format!("1d1 d6s{count}{pips}"));
    }

    // Alternative Hero System notation with explicit fractional dice (2hsk1 = 2.5d6 killing)
    if let Some(captures) = HS_FRAC_REGEX.captures(input) {
        let dice_count = &captures[1];
        let damage_type = &captures[2];
        let fraction = &captures[3];

        return Some(match (damage_type, fraction) {
            ("k", "1") => {
                // Killing damage with +0.5 dice: 2hsk1 = 2d6 + 1d3
                format!("{dice_count}d6 + 1d3 hsk")
            }
            ("n", _) => {
                // Normal damage ignores fraction
                format!("{dice_count}d6 hsn")
            }
            ("h", _) => {
                // Healing ignores fraction modifier
                if let Ok(count) = dice_count.parse::<u32>() {
                    format!("{count}d6 + {count}")
                } else {
                    format!("{dice_count}d6 + {dice_count}")
                }
            }
            _ => return None,
        });
    }

    // Plot dice (3dp -> 3d6 plot)
    if let Some(captures) = DP_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d6 plot"));
    }

    // 1. Handle ex5t8ds10 → 5d10 t8ds10 (custom target + double, long form)
    if let Some(captures) = EX_T_DS_REGEX.captures(input) {
        return process_exalted_pool(&captures[1], Some(&captures[2]), &captures[3]);
    }

    // 2. Handle ex5t8d9 → 5d10 t8ds9 (custom target + double, short form)
    if let Some(captures) = EX_T_D_REGEX.captures(input) {
        return process_exalted_pool(&captures[1], Some(&captures[2]), &captures[3]);
    }

    // 3. Handle ex7d8 → 7d10 t7ds8 (simple double, short form)
    if let Some(captures) = EX_D_REGEX.captures(input) {
        return process_exalted_pool(&captures[1], None, &captures[2]);
    }

    // 4. Handle ex10d8t6 → 10d10 t6ds8 (explicit double and target, reversed)
    if let Some(captures) = EX_DS_FULL_REGEX.captures(input) {
        return process_exalted_pool(&captures[1], Some(&captures[3]), &captures[2]);
    }

    // 5. Handle ex5ds9 → 5d10 t7ds9 (simple double, long form)
    if let Some(captures) = EX_DS_SIMPLE_REGEX.captures(input) {
        return process_exalted_pool(&captures[1], None, &captures[2]);
    }

    // 6. Handle ex5, ex5t8 → old format (BACKWARD COMPATIBLE)
    if let Some(captures) = EX_REGEX.captures(input) {
        let count = &captures[1];
        let target = captures.get(2).map_or("7", |m| m.as_str());

        // Validate dice count
        if let Ok(count_num) = count.parse::<u32>() {
            if count_num == 0 {
                return None;
            }
        } else {
            return None;
        }

        // Validate target
        if let Ok(target_num) = target.parse::<u32>() {
            if target_num == 0 || target_num > 10 {
                return None;
            }
        } else {
            return None;
        }

        // IMPORTANT: Keep old format for backward compatibility!
        return Some(format!("{count}d10 t{target} t10"));
    }

    // Earthdawn system (ed1 through ed50)
    if let Some(captures) = ED_REGEX.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=50).contains(&step) {
            return Some(get_earthdawn_step(step));
        }
    }

    // Earthdawn 4th edition (ed4e1 through ed4e100)
    if let Some(captures) = ED4E_REGEX.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=100).contains(&step) {
            return Some(get_earthdawn_4e_step(step));
        }
    }

    // DnD style rolls with modifiers (attack +10, skill -4, save +2)
    if let Some(captures) = DND_REGEX.captures(input) {
        let _roll_type = &captures[1];
        let modifier = captures.get(2).map_or("", |m| m.as_str().trim());
        return Some(format!("1d20{modifier}"));
    }

    // Savage Worlds (sw8 -> special handling for trait + wild dice)
    if let Some(captures) = SW_REGEX.captures(input) {
        let sides: u32 = captures[1].parse().ok()?;
        // Savage Worlds uses even-sided dice from d4 to d12
        if (4..=12).contains(&sides) && sides.is_multiple_of(2) {
            // We need to create an expression that rolls both dice and keeps the highest
            // This requires a different approach than simple addition
            return Some(format!("2d1 sw{sides}"));
        }
    }

    if let Some(result) = expand_marvel_multiverse_alias(input) {
        return Some(result);
    }

    if let Some(captures) = CPR_REGEX.captures(input) {
        let modifier = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some("1d10 cpr".to_string());
        } else {
            return Some(format!("1d10 cpr {modifier}"));
        }
    }

    if let Some(captures) = CPD_REGEX.captures(input) {
        let dice_count = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let modifier = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some(format!("{dice_count}d6 cpd"));
        } else {
            return Some(format!("{dice_count}d6 cpd {modifier}"));
        }
    }

    if let Some(captures) = WIT_REGEX.captures(input) {
        let modifier = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some("1d10 wit".to_string());
        } else {
            return Some(format!("1d10 wit {modifier}"));
        }
    }

    if let Some(captures) = WFRP_REGEX.captures(input) {
        let base_target: i64 = captures.get(1)?.as_str().parse().ok()?;

        // A Characteristic or Skill above 100 is not a legal starting target;
        // difficulty modifiers are what push the effective target past it.
        if !(1..=100).contains(&base_target) {
            return None;
        }

        let target = match (captures.get(2), captures.get(3)) {
            (Some(sign), Some(amount)) => {
                let amount: i64 = amount.as_str().parse().ok()?;
                if sign.as_str() == "+" {
                    base_target + amount
                } else {
                    base_target - amount
                }
            }
            _ => base_target,
        };

        // A modifier can drive the target to or below zero (a Very Hard test of
        // an unskilled Characteristic). Nothing then succeeds but an 01-05.
        let target = target.clamp(0, i64::from(crate::dice::WFRP_MAX_TARGET));

        return Some(format!("1d100 wfrp{target}"));
    }

    if let Some(captures) = CS_REGEX.captures(input) {
        let level = &captures[1];
        let modifier = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some(format!("1d20 cs{level}"));
        } else {
            return Some(format!("1d20 cs{level} {modifier}"));
        }
    }

    // Brave New World (bnw3 -> 3d6 bnw)
    if let Some(captures) = BNW_REGEX.captures(input) {
        let pool_size = &captures[1];
        return Some(format!("{pool_size}d6 bnw"));
    }

    // Handle the packed skill test first: conan2tn12f3 -> 2d20 conan2 tn12f3
    if let Some(captures) = CONAN_TN_REGEX.captures(input) {
        let dice_count = captures.get(1).map_or("2", |m| m.as_str());
        let test = &captures[2];
        return Some(format!("{dice_count}d20 conan{dice_count} {test}"));
    }

    // Handle Conan combined patterns first (most specific)
    if let Some(captures) = CONAN_COMBINED_REGEX.captures(input) {
        let skill_dice = &captures[1];
        let combat_dice = &captures[2];
        return Some(format!(
            "{skill_dice}d20 conan{skill_dice} {combat_dice}d6 cd{combat_dice}"
        ));
    }

    // Handle Conan skill dice patterns
    if let Some(captures) = CONAN_SKILL_REGEX.captures(input) {
        let dice_count = &captures[1];
        return Some(format!("{dice_count}d20 conan{dice_count}"));
    }

    // Handle Conan combat dice patterns
    if let Some(captures) = CONAN_COMBAT_REGEX.captures(input) {
        let dice_count = &captures[1];
        return Some(format!("{dice_count}d6 cd{dice_count}"));
    }

    if let Some(captures) = SIL_REGEX.captures(input) {
        let dice_count_str = &captures[1];
        let dice_count: u32 = dice_count_str.parse().unwrap_or(0);
        if dice_count == 0 || dice_count > 10 {
            return None; // Invalid dice count, don't expand
        }
        // Expand to dice notation that works with existing parser
        return Some(format!("1d6 sil{dice_count_str}"));
    }

    // D6 Legends (8d6l -> 7d6 t4 + 1d6 t4f1ie6)
    if let Some(captures) = D6L_REGEX.captures(input) {
        let count_str = &captures[1];

        // Validate dice count - must be positive
        if let Ok(count) = count_str.parse::<u32>() {
            if count == 0 {
                return None; // Invalid dice count
            }

            if count == 1 {
                // Special case: 1d6l = wild die only
                return Some("1d6 t4f1ie6".to_string());
            } else {
                // Standard case: count > 1 = (count-1) regular dice + 1 wild die
                return Some(format!("{}d6 t4 + 1d6 t4f1ie6", count - 1));
            }
        }
        return None; // Invalid count format
    }

    // VTM5 - Vampire: The Masquerade 5th Edition (vtm7h2 -> 7d10 vtm5p7h2)
    if let Some(captures) = VTM5_REGEX.captures(input) {
        let pool_size = &captures[1];
        let hunger_dice = &captures[2];

        // Validate ranges
        if let (Ok(pool), Ok(hunger)) = (pool_size.parse::<u32>(), hunger_dice.parse::<u32>())
            && pool > 0
            && pool <= 30
            && hunger <= pool
        {
            return Some(format!("{pool_size}d10 vtm5p{pool_size}h{hunger_dice}"));
        }
    }

    // Lasers & Feelings (2lf4, 2lf4l, 2lf4f)
    if let Some(captures) = LF_REGEX.captures(input) {
        let dice_count_str = &captures[1];
        let target_str = &captures[2];
        let roll_type = captures.get(3).map_or("", |m| m.as_str());

        if let (Ok(dice_count), Ok(target)) =
            (dice_count_str.parse::<u32>(), target_str.parse::<u32>())
        {
            // Validate dice count (reasonable range)
            if dice_count == 0 || dice_count > 20 {
                return None;
            }

            // Validate target (2-5 as per L&F rules)
            if !(2..=5).contains(&target) {
                return None;
            }

            // Determine roll type
            let lf_type = match roll_type {
                "l" => "l", // Explicit Lasers
                "f" => "f", // Explicit Feelings
                _ => "",    // Generic (let user decide)
            };

            return Some(format!("{dice_count}d6 lf{target}{lf_type}"));
        }
    }

    // Forged in the Dark (fitd7 -> 7d6 fitd, fitd0 -> 2d6 fitd0)
    if let Some(captures) = FITD_REGEX.captures(input) {
        let dice_count_str = &captures[1];

        if let Ok(dice_count) = dice_count_str.parse::<u32>() {
            if dice_count == 0 {
                return Some("2d6 fitd0".to_string());
            } else if dice_count <= 10 {
                return Some(format!("{dice_count}d6 fitd"));
            }
        }
        return None; // Invalid dice count
    }

    None
}

// Shared base steps for both Earthdawn editions (steps 1-18 are identical)
static EARTHDAWN_BASE_STEPS: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
    let mut steps = HashMap::new();
    steps.insert(1, "1d4 ie - 2");
    steps.insert(2, "1d4 ie - 1");
    steps.insert(3, "1d4 ie");
    steps.insert(4, "1d6 ie");
    steps.insert(5, "1d8 ie");
    steps.insert(6, "1d10 ie");
    steps.insert(7, "1d12 ie");
    steps.insert(8, "2d6 ie");
    steps.insert(9, "1d8 ie + 1d6 ie");
    steps.insert(10, "2d8 ie");
    steps.insert(11, "1d10 ie + 1d8 ie");
    steps.insert(12, "2d10 ie");
    steps.insert(13, "1d12 ie + 1d10 ie");
    steps.insert(14, "2d12 ie");
    steps.insert(15, "1d12 ie + 2d6 ie");
    steps.insert(16, "1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(17, "1d12 ie + 2d8 ie");
    steps.insert(18, "1d12 ie + 1d10 ie + 1d8 ie");
    steps
});

// Earthdawn 1st Edition specific steps (19-50, different from 4E)
static EARTHDAWN_1E_EXTENDED_STEPS: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
    let mut steps = HashMap::new();
    steps.insert(19, "1d20 ie + 2d6 ie");
    steps.insert(20, "1d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(21, "1d20 ie + 1d10 ie + 1d6 ie");
    steps.insert(22, "1d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(23, "1d20 ie + 2d10 ie");
    steps.insert(24, "1d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(25, "1d20 ie + 1d12 ie + 1d8 ie + 1d4 ie");
    steps.insert(26, "1d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(27, "1d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(28, "1d20 ie + 2d10 ie + 1d8 ie");
    steps.insert(29, "1d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(30, "1d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(31, "1d20 ie + 1d10 ie + 2d8 ie + 1d6 ie");
    steps.insert(32, "1d20 ie + 2d10 ie + 1d8 ie + 1d6 ie");
    steps.insert(33, "1d20 ie + 2d10 ie + 2d8 ie");
    steps.insert(34, "1d20 ie + 3d10 ie + 1d8 ie");
    steps.insert(35, "1d20 ie + 1d12 ie + 2d10 ie + 1d8 ie");
    steps.insert(36, "2d20 ie + 1d10 ie + 1d8 ie + 1d4 ie");
    steps.insert(37, "2d20 ie + 1d10 ie + 1d8 ie + 1d6 ie");
    steps.insert(38, "2d20 ie + 1d10 ie + 2d8 ie");
    steps.insert(39, "2d20 ie + 2d10 ie + 1d8 ie");
    steps.insert(40, "2d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(41, "2d20 ie + 1d10 ie + 1d8 ie + 2d6 ie");
    steps.insert(42, "2d20 ie + 1d10 ie + 2d8 ie + 1d6 ie");
    steps.insert(43, "2d20 ie + 2d10 ie + 1d8 ie + 1d6 ie");
    steps.insert(44, "2d20 ie + 3d10 ie + 1d8 ie");
    steps.insert(45, "2d20 ie + 3d10 ie + 1d8 ie");
    steps.insert(46, "2d20 ie + 1d12 ie + 2d10 ie + 1d8 ie");
    steps.insert(47, "2d20 ie + 2d10 ie + 2d8 ie + 1d4 ie");
    steps.insert(48, "2d20 ie + 2d10 ie + 2d8 ie + 1d6 ie");
    steps.insert(49, "2d20 ie + 2d10 ie + 3d8 ie");
    steps.insert(50, "2d20 ie + 3d10 ie + 2d8 ie");
    steps
});

// Earthdawn 4th Edition specific steps (19-100, different progression from 1E)
static EARTHDAWN_4E_EXTENDED_STEPS: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
    let mut steps = HashMap::new();
    steps.insert(19, "1d20 ie + 2d6 ie");
    steps.insert(20, "1d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(21, "1d20 ie + 2d8 ie");
    steps.insert(22, "1d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(23, "1d20 ie + 2d10 ie");
    steps.insert(24, "1d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(25, "1d20 ie + 2d12 ie");
    steps.insert(26, "1d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(27, "1d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(28, "1d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(29, "1d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(30, "2d20 ie + 2d6 ie");
    steps.insert(31, "2d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(32, "2d20 ie + 2d8 ie");
    steps.insert(33, "2d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(34, "2d20 ie + 2d10 ie");
    steps.insert(35, "2d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(36, "2d20 ie + 2d12 ie");
    steps.insert(37, "2d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(38, "2d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(39, "2d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(40, "2d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(41, "3d20 ie + 2d6 ie");
    steps.insert(42, "3d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(43, "3d20 ie + 2d8 ie");
    steps.insert(44, "3d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(45, "3d20 ie + 2d10 ie");
    steps.insert(46, "3d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(47, "3d20 ie + 2d12 ie");
    steps.insert(48, "3d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(49, "3d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(50, "3d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(51, "3d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(52, "4d20 ie + 2d6 ie");
    steps.insert(53, "4d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(54, "4d20 ie + 2d8 ie");
    steps.insert(55, "4d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(56, "4d20 ie + 2d10 ie");
    steps.insert(57, "4d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(58, "4d20 ie + 2d12 ie");
    steps.insert(59, "4d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(60, "4d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(61, "4d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(62, "4d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(63, "5d20 ie + 2d6 ie");
    steps.insert(64, "5d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(65, "5d20 ie + 2d8 ie");
    steps.insert(66, "5d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(67, "5d20 ie + 2d10 ie");
    steps.insert(68, "5d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(69, "5d20 ie + 2d12 ie");
    steps.insert(70, "5d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(71, "5d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(72, "5d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(73, "5d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(74, "6d20 ie + 2d6 ie");
    steps.insert(75, "6d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(76, "6d20 ie + 2d8 ie");
    steps.insert(77, "6d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(78, "6d20 ie + 2d10 ie");
    steps.insert(79, "6d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(80, "6d20 ie + 2d12 ie");
    steps.insert(81, "6d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(82, "6d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(83, "6d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(84, "6d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(85, "7d20 ie + 2d6 ie");
    steps.insert(86, "7d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(87, "7d20 ie + 2d8 ie");
    steps.insert(88, "7d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(89, "7d20 ie + 2d10 ie");
    steps.insert(90, "7d20 ie + 1d12 ie + 1d10 ie");
    steps.insert(91, "7d20 ie + 2d12 ie");
    steps.insert(92, "7d20 ie + 1d12 ie + 2d6 ie");
    steps.insert(93, "7d20 ie + 1d12 ie + 1d8 ie + 1d6 ie");
    steps.insert(94, "7d20 ie + 1d12 ie + 2d8 ie");
    steps.insert(95, "7d20 ie + 1d12 ie + 1d10 ie + 1d8 ie");
    steps.insert(96, "8d20 ie + 2d6 ie");
    steps.insert(97, "8d20 ie + 1d8 ie + 1d6 ie");
    steps.insert(98, "8d20 ie + 2d8 ie");
    steps.insert(99, "8d20 ie + 1d10 ie + 1d8 ie");
    steps.insert(100, "8d20 ie + 2d10 ie");
    steps
});

fn get_earthdawn_step(step: u32) -> String {
    // First check base steps (1-18)
    if let Some(&step_str) = EARTHDAWN_BASE_STEPS.get(&step) {
        return step_str.to_string();
    }

    // Then check 1E extended steps (19-50)
    if let Some(&step_str) = EARTHDAWN_1E_EXTENDED_STEPS.get(&step) {
        return step_str.to_string();
    }

    "1d6".to_string() // fallback
}

fn get_earthdawn_4e_step(step: u32) -> String {
    // First check base steps (1-18)
    if let Some(&step_str) = EARTHDAWN_BASE_STEPS.get(&step) {
        return step_str.to_string();
    }

    // Then check 4E extended steps (19-100)
    if let Some(&step_str) = EARTHDAWN_4E_EXTENDED_STEPS.get(&step) {
        return step_str.to_string();
    }

    "1d6".to_string() // fallback
}

fn expand_marvel_multiverse_alias(input: &str) -> Option<String> {
    if input == "mm" {
        return Some("3d6 mm".to_string());
    }

    // IMPORTANT: Use the existing MM_REGEX first - it's stricter than our custom parsing
    if let Some(captures) = MM_REGEX.captures(input) {
        let mut edges = 0i32;
        let mut troubles = 0i32;

        // Parse first modifier
        if let Some(first_num) = captures.get(1) {
            let num = if first_num.as_str().is_empty() {
                1
            } else {
                first_num.as_str().parse().unwrap_or(1)
            };
            if let Some(first_type) = captures.get(2) {
                match first_type.as_str() {
                    "e" => edges += num,
                    "t" => troubles += num,
                    _ => {}
                }
            }
        }

        // Parse second modifier
        if let Some(second_num) = captures.get(3) {
            let num = if second_num.as_str().is_empty() {
                1
            } else {
                second_num.as_str().parse().unwrap_or(1)
            };
            if let Some(second_type) = captures.get(4) {
                match second_type.as_str() {
                    "e" => edges += num,
                    "t" => troubles += num,
                    _ => {}
                }
            }
        }

        let net_edges = if edges > troubles {
            edges - troubles
        } else {
            0
        };
        let net_troubles = if troubles > edges {
            troubles - edges
        } else {
            0
        };

        // Create format that's easier for parser to handle
        if net_edges > 0 {
            return Some(format!("3d6 mme{net_edges}"));
        } else if net_troubles > 0 {
            return Some(format!("3d6 mmt{net_troubles}"));
        } else {
            return Some("3d6 mm".to_string());
        }
    }

    // ONLY handle mathematical modifiers if we have a clear pattern
    // This is more conservative - only handle obvious mathematical expressions
    if input.starts_with("mm ") {
        // Look for patterns like "mm 5e - 3" or "mm + 5"
        // Split on mathematical operators but be very conservative
        if let Some(op_pos) = input
            .find(" + ")
            .or_else(|| input.find(" - "))
            .or_else(|| input.find(" * "))
            .or_else(|| input.find(" / "))
        {
            let mm_part = &input[..op_pos];
            let math_part = &input[op_pos..];

            // Try to expand the MM part using the strict regex
            if let Some(expanded_mm) = expand_marvel_multiverse_alias(mm_part) {
                return Some(format!("{expanded_mm}{math_part}"));
            }
        }
    }

    None
}

fn process_wod_regex_captures(
    captures: &regex::Captures,
    base_format: &str,
    base_format_with_modifier: &str,
) -> Option<String> {
    let count = &captures[1];
    let difficulty = &captures[2];
    let modifier = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

    if modifier.is_empty() {
        Some(
            base_format
                .replace("{count}", count)
                .replace("{difficulty}", difficulty),
        )
    } else {
        Some(
            base_format_with_modifier
                .replace("{count}", count)
                .replace("{difficulty}", difficulty)
                .replace("{modifier}", modifier),
        )
    }
}

/// Expand A5E (Level Up Advanced 5th Edition) aliases
///
/// Concise syntax assuming d20 by default:
/// - a5e +5 ed1 → 1d20+5 + 1d4
/// - a5e ed2 → 1d20 + 1d6 (no modifier)
/// - +a5e +5 ed1 → 2d20 kh1+5 + 1d4 (advantage)
/// - -a5e +5 ed1 → 2d20 kl1+5 + 1d4 (disadvantage)
pub fn expand_a5e_alias(input: &str) -> Option<String> {
    let input = input.trim();

    // Determine expertise die size
    let get_expertise_die = |ed_value: &str| -> Option<&str> {
        match ed_value {
            "1" => Some("1d4"),     // Level 1 expertise = d4
            "2" => Some("1d6"),     // Level 2 expertise = d6
            "3" => Some("1d8"),     // Level 3 expertise = d8
            "4" => Some("1d4"),     // Explicit d4
            "6" => Some("1d6"),     // Explicit d6
            "8" => Some("1d8"),     // Explicit d8
            "10" => Some("1d10"),   // Explicit d10 (house rules)
            "12" => Some("1d12"),   // Explicit d12 (house rules)
            "20" => Some("1d20"),   // Explicit d20 (house rules)
            "100" => Some("1d100"), // Explicit d100 (house rules)
            _ => None,
        }
    };

    // Helper function to process A5E captures and eliminate duplication
    let process_a5e_captures =
        |captures: regex::Captures, roll_type: A5eRollType| -> Option<String> {
            let modifier = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let ed_value = captures.get(2).unwrap().as_str();

            if let Some(expertise_die) = get_expertise_die(ed_value) {
                let base_roll = match roll_type {
                    A5eRollType::Basic => format!("1d20{modifier}"),
                    A5eRollType::Advantage => format!("2d20 k1{modifier}"),
                    A5eRollType::Disadvantage => format!("2d20 kl1{modifier}"),
                };
                return Some(format!("{base_roll} + {expertise_die}"));
            }
            None
        };

    // Define roll type enum to avoid duplication
    enum A5eRollType {
        Basic,
        Advantage,
        Disadvantage,
    }

    // Check for advantage A5E
    if let Some(captures) = A5E_ADV_REGEX.captures(input) {
        return process_a5e_captures(captures, A5eRollType::Advantage);
    }

    // Check for disadvantage A5E
    if let Some(captures) = A5E_DIS_REGEX.captures(input) {
        return process_a5e_captures(captures, A5eRollType::Disadvantage);
    }

    // Check for basic A5E
    if let Some(captures) = A5E_BASIC_REGEX.captures(input) {
        return process_a5e_captures(captures, A5eRollType::Basic);
    }

    None
}

fn expand_alien_alias(input: &str) -> Option<String> {
    // Check for push roll first (alien4s3p -> alien4s4)
    if let Some(captures) = ALIEN_PUSH_REGEX.captures(input) {
        let base_dice = &captures[1];
        let current_stress: u32 = captures[2].parse().ok()?;
        let new_stress = current_stress + 1; // Push adds 1 stress

        // Return the new stress level alias (which will be expanded again)
        return Some(format!("alien{base_dice}s{new_stress}"));
    }

    // Check for stress dice (alien4s2 -> 4d6 alien + 2d6 aliens2)
    if let Some(captures) = ALIEN_STRESS_REGEX.captures(input) {
        let base_dice = &captures[1];
        let stress_dice = &captures[2];

        // Validate inputs
        let base_count: u32 = base_dice.parse().ok()?;
        let stress_level: u32 = stress_dice.parse().ok()?;

        // Validate ranges
        if base_count == 0 || base_count > 20 {
            return None; // Invalid base dice count
        }
        if stress_level == 0 || stress_level > 10 {
            return None; // Invalid stress level
        }

        return Some(format!(
            "{base_dice}d6 alien + {stress_dice}d6 aliens{stress_dice}"
        ));
    }

    // Check for basic alien roll (alien4 -> 4d6 alien)
    if let Some(captures) = ALIEN_BASIC_REGEX.captures(input) {
        let base_dice = &captures[1];

        // Validate input
        let base_count: u32 = base_dice.parse().ok()?;
        if base_count == 0 || base_count > 20 {
            return None; // Invalid base dice count
        }

        return Some(format!("{base_dice}d6 alien"));
    }

    None
}

fn expand_wild_words_alias(input: &str) -> Option<String> {
    // Check for cutting dice first (ww4c2 -> 4d6 wwc2)
    if let Some(captures) = WILD_WORLDS_CUT_REGEX.captures(input) {
        let dice_count = &captures[1];
        let cut_count = &captures[2];

        // Validate inputs
        let dice_num: u32 = dice_count.parse().ok()?;
        let cut_num: u32 = cut_count.parse().ok()?;

        // Validate ranges
        if dice_num == 0 || dice_num > 20 {
            return None; // Invalid dice count
        }
        if cut_num == 0 || cut_num >= dice_num {
            return None; // Can't cut 0 or more than available dice
        }

        return Some(format!("{dice_count}d6 wwc{cut_count}"));
    }

    // Check for basic wild worlds roll (ww4 -> 4d6 ww)
    if let Some(captures) = WILD_WORLDS_BASIC_REGEX.captures(input) {
        let dice_count = &captures[1];

        // Validate input
        let dice_num: u32 = dice_count.parse().ok()?;
        if dice_num == 0 || dice_num > 20 {
            return None; // Invalid dice count
        }

        return Some(format!("{dice_count}d6 ww"));
    }

    None
}

/// Open Legend attribute dice, per the SRD Attribute Overview table.
/// Score 0 grants no attribute dice — the d20 is rolled alone.
fn open_legend_attribute_dice(score: u32) -> Option<(u32, u32)> {
    match score {
        1 => Some((1, 4)),
        2 => Some((1, 6)),
        3 => Some((1, 8)),
        4 => Some((1, 10)),
        5 => Some((2, 6)),
        6 => Some((2, 8)),
        7 => Some((2, 10)),
        8 => Some((3, 8)),
        9 => Some((3, 10)),
        10 => Some((4, 8)),
        _ => None,
    }
}

/// `ol5` → `1d20 ie20 + 2d6 ie6`, `ol5a2` → advantage 2, `ol5d1` → disadvantage 1.
///
/// Advantage/disadvantage rolls extra *attribute* dice and drops the surplus
/// before explosions, which is what `adv#`/`dis#` do. With no attribute dice
/// (score 0) the SRD instead rolls 2d20 and keeps the better/worse, and caps
/// the level at 1 — `adv1`/`dis1` on the d20 expresses exactly that.
fn expand_open_legend_alias(input: &str) -> Option<String> {
    let captures = OL_REGEX.captures(input)?;

    let score: u32 = captures[1].parse().ok()?;
    if score > 10 {
        return None;
    }

    let level: u32 = match captures.get(3) {
        Some(digits) => digits.as_str().parse().ok()?,
        None => 0,
    };
    let modifier_name = match captures.get(2).map(|m| m.as_str()) {
        Some("a") => "adv",
        Some("d") => "dis",
        _ => "",
    };

    let Some((count, sides)) = open_legend_attribute_dice(score) else {
        // No attribute dice: advantage/disadvantage applies to the d20 and
        // cannot exceed 1 (SRD: "Advantage and Disadvantage Without Attribute Dice")
        return Some(match level {
            0 => "1d20 ie20".to_string(),
            _ => format!("1d20 ie20 {modifier_name}1"),
        });
    };

    let attribute_dice = match level {
        0 => format!("{count}d{sides} ie{sides}"),
        _ => format!("{count}d{sides} ie{sides} {modifier_name}{level}"),
    };

    Some(format!("1d20 ie20 + {attribute_dice}"))
}

/// Essence20 (Renegade Game Studios) skill test: `1d20` plus a skill die.
///
/// `ess1d8` → `1d20 ess4`, `ess1d8s` (specialization) → `1d20 esss4`,
/// `+ess1d8` (Edge) → `2d20 k1 ess4`, `-ess1d8` (Snag) → `2d20 kl1 ess4`.
///
/// The skill die is carried as its rank number rather than as dice notation so
/// the modifier token stays free of the `d` that the parser uses to recognise
/// dice expressions.  `1d2` is a legal rank, so the count may be omitted
/// (`essd8`) but only ranks on the ladder expand.
fn expand_essence20_alias(input: &str) -> Option<String> {
    let captures = ESS_REGEX.captures(input)?;

    let count: u32 = match captures.get(2).map(|m| m.as_str()).unwrap_or("") {
        "" => 1,
        digits => digits.parse().ok()?,
    };
    let sides: u32 = captures[3].parse().ok()?;
    let rank = super::essence20_rank_number(count, sides)?;

    let skill_die = match captures.get(4) {
        Some(_) => format!("esss{rank}"),
        None => format!("ess{rank}"),
    };

    // Edge rolls 2d20 and keeps the higher, Snag keeps the lower
    let d20 = match captures.get(1).map(|m| m.as_str()) {
        Some("+") => "2d20 k1",
        Some("-") => "2d20 kl1",
        _ => "1d20",
    };

    let modifier = match captures.get(5) {
        Some(m) => format!(" {}", m.as_str().replace(' ', "")),
        None => String::new(),
    };

    Some(format!("{d20} {skill_die}{modifier}"))
}

/// The Darkest House (Monte Cook Games) — `2d6 + Rating` plus the House Die.
///
/// A Boon rolls a third die and discards the lowest; a Bane discards the
/// highest, which is exactly what `adv1` / `dis1` already do.  `c` marks
/// "calling upon the house", where the House Die is added to the result.
/// The rating and any flat modifier (proficiency, armour, …) are folded into a
/// single addition.
fn expand_darkest_house_alias(input: &str) -> Option<String> {
    let captures = TDH_REGEX.captures(input)?;

    let rating: i64 = captures[1].parse().ok()?;
    let flat_modifier: i64 = match captures.get(4) {
        Some(matched) => matched.as_str().replace(' ', "").parse().ok()?,
        None => 0,
    };
    let total = rating + flat_modifier;

    let dice = match captures.get(2).map(|m| m.as_str()) {
        Some("b") => "2d6 adv1",
        Some("n") => "2d6 dis1",
        _ => "2d6",
    };

    let house_die = match captures.get(3) {
        Some(_) => "tdhc",
        None => "tdh",
    };

    let modifier = match total {
        0 => String::new(),
        positive if positive > 0 => format!(" + {positive}"),
        negative => format!(" - {}", -negative),
    };

    Some(format!("{dice} {house_die}{modifier}"))
}

fn expand_mothership_alias(input: &str) -> Option<String> {
    if let Some(captures) = MS_REGEX.captures(input) {
        let advantage_sign = captures.get(1).map(|m| m.as_str());
        let stat_value = captures.get(2).map(|m| m.as_str());

        // We need to create modifiers that DON'T use +/- suffixes
        // Instead, use a different marker that won't be confused with operators
        match (advantage_sign, stat_value) {
            (Some("+"), Some(stat)) => {
                // Use 'a' suffix for advantage
                return Some(format!("2d100 ms{}a", stat));
            }
            (Some("-"), Some(stat)) => {
                // Use 'd' suffix for disadvantage
                return Some(format!("2d100 ms{}d", stat));
            }
            (Some("+"), None) => {
                return Some("2d100 msa".to_string());
            }
            (Some("-"), None) => {
                return Some("2d100 msd".to_string());
            }
            (None, Some(stat)) => {
                return Some(format!("1d100 ms{}", stat));
            }
            (None, None) => {
                return Some("1d100 ms".to_string());
            }
            _ => return None,
        }
    }

    None
}
