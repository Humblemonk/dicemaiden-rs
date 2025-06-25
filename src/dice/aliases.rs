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
    Regex::new(r"^wng(?:\s+dn(\d+))?\s+(\d+)d(\d+)(?:\s*!\s*(\w+))?$")
        .expect("Failed to compile WNG_REGEX")
});

static WNG_SIMPLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^wng\s+(\d+)d(\d+)$").expect("Failed to compile WNG_SIMPLE_REGEX"));

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
fn process_hero_system_dice(dice_count_str: &str, _damage_type: &str, dice_type: &str) -> Option<String> {
    if let Ok(dice_count) = dice_count_str.parse::<f64>() {
        let whole_dice = dice_count.floor() as u32;
        let has_fractional = dice_count.fract() > 0.0;

        if whole_dice == 0 && has_fractional {
            return Some(format!("1d3 {}", dice_type));
        }

        let dice_expr = if has_fractional {
            format!("{}d6 + 1d3 {}", whole_dice, dice_type)
        } else {
            format!("{}d6 {}", whole_dice, dice_type)
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

        return Some(format!("{}d{} {}{}", count, sides, gb_type, modifier));
    }

    // Godbound system - simple modifiers (gb+5, gbs-2, etc.)
    if let Some(captures) = GB_SIMPLE_REGEX.captures(input) {
        let gb_type = &captures[1];
        let modifier = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        return Some(format!("1d20 {}{}", gb_type, modifier));
    }

    // Wrath & Glory (wng 4d6, wng dn2 4d6, wng 4d6 !soak)
    if let Some(captures) = WNG_REGEX.captures(input) {
        let difficulty = captures.get(1).map(|m| m.as_str());
        let count = &captures[2];
        let sides = &captures[3];
        let special = captures.get(4).map(|m| m.as_str());

        return Some(match (difficulty, special) {
            (Some(dn), Some("soak") | Some("exempt") | Some("dmg")) => {
                format!("{}d{} wngdn{}t", count, sides, dn)
            }
            (Some(dn), _) => {
                format!("{}d{} wngdn{}", count, sides, dn)
            }
            (None, Some("soak") | Some("exempt") | Some("dmg")) => {
                format!("{}d{} wngt", count, sides)
            }
            (None, _) => {
                format!("{}d{} wng", count, sides)
            }
        });
    }

    // Simple wng pattern (wng 4d6)
    if let Some(captures) = WNG_SIMPLE_REGEX.captures(input) {
        let count = &captures[1];
        let sides = &captures[2];
        return Some(format!("{}d{} wng", count, sides));
    }

    // Chronicles of Darkness (4cod -> 4d10 t8 ie10)
    if let Some(captures) = COD_REGEX.captures(input) {
        let count = &captures[1];
        let variant = captures.get(2).map_or("", |m| m.as_str());
        let modifier = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        let modifier_part = if modifier.is_empty() {
            String::new()
        } else {
            format!(" {}", modifier)
        };

        return Some(match variant {
            "8" => format!("{}d10 t7 ie10{}", count, modifier_part), // 8-again
            "9" => format!("{}d10 t6 ie10{}", count, modifier_part), // 9-again
            "r" => format!("{}d10 t8 ie10 r1{}", count, modifier_part), // rote quality
            _ => format!("{}d10 t8 ie10{}", count, modifier_part),   // standard
        });
    }

    // World of Darkness (4wod8 -> 4d10 f1 ie10 t8)
    if let Some(captures) = WOD_REGEX.captures(input) {
        let count = &captures[1];
        let difficulty = &captures[2];
        let modifier = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        if modifier.is_empty() {
            return Some(format!("{}d10 f1 ie10 t{}", count, difficulty));
        } else {
            return Some(format!("{}d10 f1 ie10 t{} {}", count, difficulty, modifier));
        }
    }

    // Dark Heresy (dh 4d10 -> 4d10 ie10)
    if let Some(captures) = DH_REGEX.captures(input) {
        let count = &captures[1];
        let sides = &captures[2];
        return Some(format!("{}d{} ie{} dh", count, sides, sides));
    }

    // Fudge dice (3df -> 3d3 fudge)
    if let Some(captures) = DF_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d3 fudge", count));
    }

    // Warhammer (3wh4+ -> 3d6 t4)
    if let Some(captures) = WH_REGEX.captures(input) {
        let count = &captures[1];
        let target = &captures[2];
        return Some(format!("{}d6 t{}", count, target));
    }

    // Double digit (dd34 -> (1d3 * 10) + 1d4)
    if let Some(captures) = DD_REGEX.captures(input) {
        let tens = &captures[1];
        let ones = &captures[2];
        return Some(format!("1d{} * 10 + 1d{}", tens, ones));
    }

    // General advantage/disadvantage (+d20, -d20, etc.) but NOT +d% or -d%
    if let Some(captures) = ADV_REGEX.captures(input) {
        let modifier = &captures[1];
        let sides = &captures[2];

        return Some(match modifier {
            "+" => format!("2d{} k1", sides),  // advantage
            "-" => format!("2d{} kl1", sides), // disadvantage
            _ => return None,
        });
    }

    // Simple percentile (xd% -> xd100)
    if let Some(captures) = PERC_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d100", count));
    }

    // Shadowrun (sr6 -> 6d6 t5)
    if let Some(captures) = SR_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 t5", count));
    }

    // Storypath (sp4 -> 4d10 t8 ie10)
    if let Some(captures) = SP_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d10 t8 ie10", count));
    }

    // Year Zero (6yz -> 6d6 t6)
    if let Some(captures) = YZ_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 t6", count));
    }

    // Sunsails New Millennium (snm5 -> 5d6 ie6 t4)
    if let Some(captures) = SNM_REGEX.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 ie6 t4", count));
    }

    // D6 System (d6s4 -> 4d6 + 1d6 ie)
    if let Some(captures) = D6S_REGEX.captures(input) {
        let count = &captures[1];
        let pips = captures.get(2).map_or("", |m| m.as_str());
        return Some(format!("{}d6 + 1d6 ie{}", count, pips));
    }

    // Alternative Hero System notation with explicit fractional dice (2hsk1 = 2.5d6 killing)
    if let Some(captures) = HS_FRAC_REGEX.captures(input) {
        let dice_count = &captures[1];
        let damage_type = &captures[2];
        let fraction = &captures[3];

        return Some(match (damage_type, fraction) {
            ("k", "1") => {
                // Killing damage with +0.5 dice: 2hsk1 = 2d6 + 1d3
                format!("{}d6 + 1d3 hsk", dice_count)
            }
            ("n", _) => {
                // Normal damage ignores fraction
                format!("{}d6 hsn", dice_count)
            }
            ("h", _) => {
                // Healing ignores fraction modifier
                if let Ok(count) = dice_count.parse::<u32>() {
                    format!("{}d6 + {}", count, count)
                } else {
                    format!("{}d6 + {}", dice_count, dice_count)
                }
            }
            _ => return None,
        });
    }

    // Exalted (ex5 -> 5d10 t7 t10, ex5t8 -> 5d10 t8 t10)
    if let Some(captures) = EX_REGEX.captures(input) {
        let count = &captures[1];
        let target = captures.get(2).map_or("7", |m| m.as_str());
        return Some(format!("{}d10 t{} t10", count, target));
    }

    // Earthdawn system (ed1 through ed50)
    if let Some(captures) = ED_REGEX.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=50).contains(&step) {
            return Some(get_earthdawn_step(step));
        }
    }

    // Earthdawn 4th edition (ed4e1 through ed4e50)
    if let Some(captures) = ED4E_REGEX.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=50).contains(&step) {
            return Some(get_earthdawn_4e_step(step));
        }
    }

    // DnD style rolls with modifiers (attack +10, skill -4, save +2)
    if let Some(captures) = DND_REGEX.captures(input) {
        let _roll_type = &captures[1];
        let modifier = captures.get(2).map_or("", |m| m.as_str().trim());
        return Some(format!("1d20{}", modifier));
    }

    None
}

// Pre-calculate and store Earthdawn step mappings for better performance
static EARTHDAWN_STEPS: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
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

fn get_earthdawn_step(step: u32) -> String {
    EARTHDAWN_STEPS
        .get(&step)
        .map(|&s| s.to_string())
        .unwrap_or_else(|| "1d6".to_string()) // fallback
}

fn get_earthdawn_4e_step(step: u32) -> String {
    // Earthdawn 4th edition steps would be similar but potentially different
    // For now, using the same as standard earthdawn
    get_earthdawn_step(step)
}
