use regex::Regex;
use std::collections::HashMap;

pub fn expand_alias(input: &str) -> Option<String> {
    let input = input.trim().to_lowercase();

    // Handle parameterized aliases first
    if let Some(result) = expand_parameterized_alias(&input) {
        return Some(result);
    }

    // Static aliases
    let aliases = get_static_aliases();
    aliases.get(&input).map(|s| s.to_string())
}

fn expand_parameterized_alias(input: &str) -> Option<String> {
    // Godbound system - full dice expressions (gb 3d8, gbs 2d10, etc.)
    let gb_dice_regex = Regex::new(r"^(gbs?)\s+(\d+)d(\d+)(?:\s*([+-]\s*\d+))?$").unwrap();
    if let Some(captures) = gb_dice_regex.captures(input) {
        let gb_type = &captures[1];
        let count = &captures[2];
        let sides = &captures[3];
        let modifier = captures.get(4).map(|m| m.as_str().trim()).unwrap_or("");

        return Some(format!("{}d{} {}{}", count, sides, gb_type, modifier));
    }

    // Godbound system - simple modifiers (gb+5, gbs-2, etc.)
    let gb_simple_regex = Regex::new(r"^(gbs?)(?:\s*([+-]\s*\d+))?$").unwrap();
    if let Some(captures) = gb_simple_regex.captures(input) {
        let gb_type = &captures[1];
        let modifier = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        return Some(format!("1d20 {}{}", gb_type, modifier));
    }

    // Wrath & Glory (wng 4d6, wng dn2 4d6, wng 4d6 !soak)
    let wng_regex = Regex::new(r"^wng(?:\s+dn(\d+))?\s+(\d+)d(\d+)(?:\s*!\s*(\w+))?$").unwrap();
    if let Some(captures) = wng_regex.captures(input) {
        let difficulty = captures.get(1).map(|m| m.as_str());
        let count = &captures[2];
        let sides = &captures[3];
        let special = captures.get(4).map(|m| m.as_str());

        // Use the new WrathGlory modifier for proper success counting
        return Some(match (difficulty, special) {
            (Some(dn), Some("soak") | Some("exempt") | Some("dmg")) => {
                // Use total instead of successes for soak/exempt/dmg rolls
                format!("{}d{} wng{}t", count, sides, dn)
            }
            (Some(dn), _) => {
                // Standard wrath & glory with wrath die and difficulty
                format!("{}d{} wng{}", count, sides, dn)
            }
            (None, Some("soak") | Some("exempt") | Some("dmg")) => {
                // Use total instead of successes for soak/exempt/dmg rolls
                format!("{}d{} wngt", count, sides)
            }
            (None, _) => {
                // Standard wrath & glory with wrath die
                format!("{}d{} wng", count, sides)
            }
        });
    }

    // Simple wng pattern (wng 4d6)
    let wng_simple_regex = Regex::new(r"^wng\s+(\d+)d(\d+)$").unwrap();
    if let Some(captures) = wng_simple_regex.captures(input) {
        let count = &captures[1];
        let sides = &captures[2];
        return Some(format!("{}d{} wng", count, sides));
    }

    // Chronicles of Darkness (4cod -> 4d10 t8 ie10)
    let cod_regex = Regex::new(r"^(\d+)cod([89r]?)$").unwrap();
    if let Some(captures) = cod_regex.captures(input) {
        let count = &captures[1];
        let variant = captures.get(2).map_or("", |m| m.as_str());

        return Some(match variant {
            "8" => format!("{}d10 t7 ie10", count),    // 8-again
            "9" => format!("{}d10 t6 ie10", count),    // 9-again
            "r" => format!("{}d10 t8 ie10 r1", count), // rote quality
            _ => format!("{}d10 t8 ie10", count),      // standard
        });
    }

    // World of Darkness (4wod8 -> 4d10 f1 ie10 t8)
    let wod_regex = Regex::new(r"^(\d+)wod(\d+)$").unwrap();
    if let Some(captures) = wod_regex.captures(input) {
        let count = &captures[1];
        let difficulty = &captures[2];
        return Some(format!("{}d10 f1 ie10 t{}", count, difficulty));
    }

    // Dark Heresy (dh 4d10 -> 4d10 ie10)
    let dh_regex = Regex::new(r"^dh\s+(\d+)d(\d+)$").unwrap();
    if let Some(captures) = dh_regex.captures(input) {
        let count = &captures[1];
        let sides = &captures[2];
        return Some(format!("{}d{} ie{}", count, sides, sides));
    }

    // Fudge dice (3df -> 3d3 fudge)
    let df_regex = Regex::new(r"^(\d+)df$").unwrap();
    if let Some(captures) = df_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d3 fudge", count));
    }

    // Warhammer (3wh4+ -> 3d6 t4)
    let wh_regex = Regex::new(r"^(\d+)wh(\d+)\+$").unwrap();
    if let Some(captures) = wh_regex.captures(input) {
        let count = &captures[1];
        let target = &captures[2];
        return Some(format!("{}d6 t{}", count, target));
    }

    // Double digit (dd34 -> (1d3 * 10) + 1d4)
    let dd_regex = Regex::new(r"^dd(\d)(\d)$").unwrap();
    if let Some(captures) = dd_regex.captures(input) {
        let tens = &captures[1];
        let ones = &captures[2];
        return Some(format!("1d{} * 10 + 1d{}", tens, ones));
    }

    // Advantage/Disadvantage (+d20, -d20, etc.)
    let adv_regex = Regex::new(r"^([+-])d(\d+|%)$").unwrap();
    if let Some(captures) = adv_regex.captures(input) {
        let modifier = &captures[1];
        let sides = &captures[2];

        let dice_sides = if sides == "%" {
            "100".to_string()
        } else {
            sides.to_string()
        };

        return Some(match modifier {
            "+" => format!("2d{} k1", dice_sides),  // advantage
            "-" => format!("2d{} kl1", dice_sides), // disadvantage
            _ => return None,
        });
    }

    // Percentile advantage/disadvantage
    if input == "+d%" {
        return Some("2d10 kl1 * 10 + 1d10".to_string());
    }
    if input == "-d%" {
        return Some("2d10 k1 * 10 + 1d10".to_string());
    }

    // Simple percentile (xd% -> xd100)
    let perc_regex = Regex::new(r"^(\d+)d%$").unwrap();
    if let Some(captures) = perc_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d100", count));
    }

    // Shadowrun (sr6 -> 6d6 t5)
    let sr_regex = Regex::new(r"^sr(\d+)$").unwrap();
    if let Some(captures) = sr_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 t5", count));
    }

    // Storypath (sp4 -> 4d10 t8 ie10)
    let sp_regex = Regex::new(r"^sp(\d+)$").unwrap();
    if let Some(captures) = sp_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d10 t8 ie10", count));
    }

    // Year Zero (6yz -> 6d6 t6)
    let yz_regex = Regex::new(r"^(\d+)yz$").unwrap();
    if let Some(captures) = yz_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 t6", count));
    }

    // Sunsails New Millennium (snm5 -> 5d6 ie6 t4)
    let snm_regex = Regex::new(r"^snm(\d+)$").unwrap();
    if let Some(captures) = snm_regex.captures(input) {
        let count = &captures[1];
        return Some(format!("{}d6 ie6 t4", count));
    }

    // D6 System (d6s4 -> 4d6 + 1d6 ie)
    let d6s_regex = Regex::new(r"^d6s(\d+)(\s*\+\s*\d+)?$").unwrap();
    if let Some(captures) = d6s_regex.captures(input) {
        let count = &captures[1];
        let pips = captures.get(2).map_or("", |m| m.as_str());
        return Some(format!("{}d6 + 1d6 ie{}", count, pips));
    }

    // Hero System (2hsn, 3hsk, 2.5hsk, 3hsh)
    let hs_regex = Regex::new(r"^(\d+(?:\.\d+)?)hs([nkh])$").unwrap();
    if let Some(captures) = hs_regex.captures(input) {
        let dice_count_str = &captures[1];
        let damage_type = &captures[2];

        match damage_type {
            "n" => {
                // Normal damage - XdY
                let dice_count = dice_count_str.parse::<f64>().ok()?;
                let whole_dice = dice_count.floor() as u32;
                let has_fractional = dice_count.fract() > 0.0;

                let dice_expr = if has_fractional {
                    format!("{}d6 + 1d3 hsn", whole_dice)
                } else {
                    format!("{}d6 hsn", whole_dice)
                };
                return Some(dice_expr);
            }
            "k" => {
                // Killing damage - XdY
                let dice_count = dice_count_str.parse::<f64>().ok()?;
                let whole_dice = dice_count.floor() as u32;
                let has_fractional = dice_count.fract() > 0.0;

                let dice_expr = if has_fractional {
                    format!("{}d6 + 1d3 hsk", whole_dice)
                } else {
                    format!("{}d6 hsk", whole_dice)
                };
                return Some(dice_expr);
            }
            "h" => {
                // To hit roll - always 3d6 regardless of the number (Hero System standard)
                return Some("3d6 hsh".to_string());
            }
            _ => return None,
        }
    }
    // Alternative Hero System notation with explicit fractional dice (2hsk1 = 2.5d6 killing)
    let hs_frac_regex = Regex::new(r"^(\d+)hs([nkh])(\d+)$").unwrap();
    if let Some(captures) = hs_frac_regex.captures(input) {
        let dice_count = &captures[1];
        let damage_type = &captures[2];
        let fraction = &captures[3];

        return Some(match (damage_type, fraction) {
            ("k", "1") => {
                // Killing damage with +0.5 dice: 2hsk1 = 2d6 + 1d3
                format!("{}d6 + 1d3", dice_count)
            }
            ("n", _) => {
                // Normal damage ignores fraction
                format!("{}d6", dice_count)
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
    let ex_regex = Regex::new(r"^ex(\d+)(?:t(\d+))?$").unwrap();
    if let Some(captures) = ex_regex.captures(input) {
        let count = &captures[1];
        let target = captures.get(2).map_or("7", |m| m.as_str());
        return Some(format!("{}d10 t{} t10", count, target));
    }

    // Earthdawn system (ed1 through ed50)
    let ed_regex = Regex::new(r"^ed(\d+)$").unwrap();
    if let Some(captures) = ed_regex.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=50).contains(&step) {
            return Some(get_earthdawn_step(step));
        }
    }

    // Earthdawn 4th edition (ed4e1 through ed4e50)
    let ed4e_regex = Regex::new(r"^ed4e(\d+)$").unwrap();
    if let Some(captures) = ed4e_regex.captures(input) {
        let step: u32 = captures[1].parse().ok()?;
        if (1..=50).contains(&step) {
            return Some(get_earthdawn_4e_step(step));
        }
    }

    // DnD style rolls with modifiers (attack +10, skill -4, save +2)
    let dnd_regex = Regex::new(r"^(attack|skill|save)(\s*[+-]\s*\d+)?$").unwrap();
    if let Some(captures) = dnd_regex.captures(input) {
        let _roll_type = &captures[1];
        let modifier = captures.get(2).map_or("", |m| m.as_str().trim());
        return Some(format!("1d20{}", modifier));
    }

    None
}

fn get_static_aliases() -> HashMap<String, String> {
    let mut aliases = HashMap::new();

    aliases.insert("age".to_string(), "2d6 + 1d6".to_string());
    aliases.insert("dndstats".to_string(), "6 4d6 k3".to_string());
    aliases.insert("attack".to_string(), "1d20".to_string());
    aliases.insert("skill".to_string(), "1d20".to_string());
    aliases.insert("save".to_string(), "1d20".to_string());
    aliases.insert("gb".to_string(), "1d20 gb".to_string()); // Basic Godbound roll with damage chart
    aliases.insert("gbs".to_string(), "1d20 gbs".to_string()); // Basic Godbound straight damage
    aliases.insert("hsn".to_string(), "1d6 hsn".to_string()); // 1d6 normal damage
    aliases.insert("hsk".to_string(), "1d6 hsk".to_string()); // 1d6 killing damage
    aliases.insert("hsh".to_string(), "3d6 hsh".to_string()); // Hero System to-hit roll
    aliases.insert("3df".to_string(), "3d3 fudge".to_string()); // Fixed Fudge dice

    aliases
}

fn get_earthdawn_step(step: u32) -> String {
    match step {
        1 => "1d4 ie - 2".to_string(),
        2 => "1d4 ie - 1".to_string(),
        3 => "1d4 ie".to_string(),
        4 => "1d6 ie".to_string(),
        5 => "1d8 ie".to_string(),
        6 => "1d10 ie".to_string(),
        7 => "1d12 ie".to_string(),
        8 => "2d6 ie".to_string(),
        9 => "1d8 ie + 1d6 ie".to_string(),
        10 => "2d8 ie".to_string(),
        11 => "1d10 ie + 1d8 ie".to_string(),
        12 => "2d10 ie".to_string(),
        13 => "1d12 ie + 1d10 ie".to_string(),
        14 => "2d12 ie".to_string(),
        15 => "1d12 ie + 2d6 ie".to_string(),
        16 => "1d12 ie + 1d8 ie + 1d6 ie".to_string(),
        17 => "1d12 ie + 2d8 ie".to_string(),
        18 => "1d12 ie + 1d10 ie + 1d8 ie".to_string(),
        19 => "1d20 ie + 2d6 ie".to_string(),
        20 => "1d20 ie + 1d8 ie + 1d6 ie".to_string(),
        21 => "1d20 ie + 1d10 ie + 1d6 ie".to_string(),
        22 => "1d20 ie + 1d10 ie + 1d8 ie".to_string(),
        23 => "1d20 ie + 2d10 ie".to_string(),
        24 => "1d20 ie + 1d12 ie + 1d10 ie".to_string(),
        25 => "1d20 ie + 1d12 ie + 1d8 ie + 1d4 ie".to_string(),
        26 => "1d20 ie + 1d12 ie + 1d8 ie + 1d6 ie".to_string(),
        27 => "1d20 ie + 1d12 ie + 2d8 ie".to_string(),
        28 => "1d20 ie + 2d10 ie + 1d8 ie".to_string(),
        29 => "1d20 ie + 1d12 ie + 1d10 ie + 1d8 ie".to_string(),
        30 => "1d20 ie + 1d12 ie + 1d10 ie + 1d8 ie".to_string(),
        31 => "1d20 ie + 1d10 ie + 2d8 ie + 1d6 ie".to_string(),
        32 => "1d20 ie + 2d10 ie + 1d8 ie + 1d6 ie".to_string(),
        33 => "1d20 ie + 2d10 ie + 2d8 ie".to_string(),
        34 => "1d20 ie + 3d10 ie + 1d8 ie".to_string(),
        35 => "1d20 ie + 1d12 ie + 2d10 ie + 1d8 ie".to_string(),
        36 => "2d20 ie + 1d10 ie + 1d8 ie + 1d4 ie".to_string(),
        37 => "2d20 ie + 1d10 ie + 1d8 ie + 1d6 ie".to_string(),
        38 => "2d20 ie + 1d10 ie + 2d8 ie".to_string(),
        39 => "2d20 ie + 2d10 ie + 1d8 ie".to_string(),
        40 => "2d20 ie + 1d12 ie + 1d10 ie + 1d8 ie".to_string(),
        41 => "2d20 ie + 1d10 ie + 1d8 ie + 2d6 ie".to_string(),
        42 => "2d20 ie + 1d10 ie + 2d8 ie + 1d6 ie".to_string(),
        43 => "2d20 ie + 2d10 ie + 1d8 ie + 1d6 ie".to_string(),
        44 => "2d20 ie + 3d10 ie + 1d8 ie".to_string(),
        45 => "2d20 ie + 3d10 ie + 1d8 ie".to_string(),
        46 => "2d20 ie + 1d12 ie + 2d10 ie + 1d8 ie".to_string(),
        47 => "2d20 ie + 2d10 ie + 2d8 ie + 1d4 ie".to_string(),
        48 => "2d20 ie + 2d10 ie + 2d8 ie + 1d6 ie".to_string(),
        49 => "2d20 ie + 2d10 ie + 3d8 ie".to_string(),
        50 => "2d20 ie + 3d10 ie + 2d8 ie".to_string(),
        _ => "1d6".to_string(), // fallback
    }
}

fn get_earthdawn_4e_step(step: u32) -> String {
    // Earthdawn 4th edition steps would be similar but potentially different
    // For now, using the same as standard earthdawn
    get_earthdawn_step(step)
}
