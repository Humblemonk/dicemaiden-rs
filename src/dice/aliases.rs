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
    Lazy::new(|| Regex::new(r"^sp(\d+)$").expect("Failed to compile SP_REGEX"));

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

static WIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^wit(?:\s*([+-]\s*\d+))?$").expect("Failed to compile WIT_REGEX"));

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
    aliases
});

pub fn expand_alias(input: &str) -> Option<String> {
    let input = input.trim().to_lowercase();

    // Handle parameterized aliases first
    if let Some(result) = expand_parameterized_alias(&input) {
        return Some(result);
    }

    // Check static aliases first (most common case) - use pre-compiled hashmap
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
            format!("{whole_dice}d6 + 1d3 {dice_type}")
        } else {
            format!("{whole_dice}d6 {dice_type}")
        };
        return Some(dice_expr);
    }
    None
}

fn expand_parameterized_alias(input: &str) -> Option<String> {
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

    // World of Darkness (4wod8 -> 4d10 f1 ie10 t8)
    if let Some(captures) = WOD_REGEX.captures(input) {
        let count = &captures[1];
        let difficulty = &captures[2];
        let modifier = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some(format!("{count}d10 f1 t{difficulty}"));
        } else {
            return Some(format!("{count}d10 f1 t{difficulty} {modifier}"));
        }
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

    // Storypath (sp4 -> 4d10 t8 ie10)
    if let Some(captures) = SP_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{count}d10 t8 ie10"));
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

    // Exalted (ex5 -> 5d10 t7 t10, ex5t8 -> 5d10 t8 t10)
    if let Some(captures) = EX_REGEX.captures(input) {
        let count = &captures[1];
        let target = captures.get(2).map_or("7", |m| m.as_str());
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
        if (4..=12).contains(&sides) && sides % 2 == 0 {
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

    if let Some(captures) = WIT_REGEX.captures(input) {
        let modifier = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some("1d10 wit".to_string());
        } else {
            return Some(format!("1d10 wit {modifier}"));
        }
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
