# Dice Rolling Syntax

## Basic Usage
- `/roll 2d6` - Roll two six-sided dice
- `/roll 3d6 + 5` - Roll 3d6 and add 5
- `/roll 4d6 k3` - Roll 4d6, keep highest 3
- `/roll d%` or `/roll d100` - Roll percentile dice

### Core Modifiers
- **Exploding**: `e6` (explode on 6), `e` (explode on max), `ie6` (explode indefinitely)
- **Keep/Drop**: `k3` (keep 3 highest), `kl2` (keep 2 lowest), `d1` (drop 1 lowest)
- **Rerolls**: `r2` (reroll ≤2 once), `ir2` (reroll ≤2 indefinitely)
- **Success Counting**: `t7` (count successes ≥7), `f1` (count failures ≤1)
- **Botch Counting**: `b1` (count botches ≤1), `b` (count botches ≤1)
- **Math Operations**: `+5`, `-3`, `*2`, `/2`
- **Additional Dice**: `+2d6`, `-1d4` (add/subtract dice rolls)

### Special Flags
- **`p`** - Private roll (only you see results)
- **`s`** - Simple output (no dice breakdown)
- **`nr`** - No results shown (just dice breakdown)
- **`ul`** - Unsorted dice results

### Advanced Features
- **Roll Sets**: `/roll 6 4d6` (roll 6 sets of 4d6, 2-20 sets allowed)
- **Multi-Roll**: `/roll 2d6 ; 3d8 ; 1d20; 4d10` (separate rolls, max 4)
- **Comments**: `/roll 2d6 ! Fire damage`
- **Labels**: `/roll (Attack) 1d20 + 5`

## Game System Aliases

### Savage Worlds
- `sw4` → 1d4 trait + 1d6 wild, keep highest, both explode
- `sw6` → 1d6 trait + 1d6 wild, keep highest, both explode
- `sw8` → 1d8 trait + 1d6 wild, keep highest, both explode
- `sw10` → 1d10 trait + 1d6 wild, keep highest, both explode
- `sw12` → 1d12 trait + 1d6 wild, keep highest, both explode
- Snake Eyes: Critical failure when both dice roll natural

### D&D/Pathfinder
- `dndstats` → 6 4d6 k3 (ability score generation)
- `attack +5` → 1d20 +5 (attack roll)
- `skill -2` → 1d20 -2 (skill check)
- `save +3` → 1d20 +3 (saving throw)
- `+d20` → 2d20 k1 (advantage)
- `-d20` → 2d20 kl1 (disadvantage)
- `+d%` → 2d10 kl1 * 10 + 1d10 - 10 (percentile advantage)
- `-d%` → 2d10 k1 * 10 + 1d10 - 10 (percentile disadvantage)

### World of Darkness / Chronicles of Darkness
- `4cod` → 4d10 t8 ie10 (Chronicles of Darkness)
- `4cod8` → 4d10 t7 ie10 (8-again rule)
- `4cod9` → 4d10 t6 ie10 (9-again rule)
- `4codr` → 4d10 t8 ie10 r1 (rote quality)
- `4wod8` → 4d10 f1 ie10 t8 (World of Darkness, difficulty 8)

### Hero System 5th Edition
- `2hsn` → 2d6 hsn (normal damage)
- `3hsk` → 3d6 hsk (killing damage with STUN multiplier)
- `2.5hsk` → 2d6 + 1d3 hsk (fractional killing damage)
- `2hsk1` → 2d6 + 1d3 hsk (alternative fractional notation)
- `3hsh` → 3d6 hsh (to-hit roll, roll-under)
- `hsn`, `hsk`, `hsh` → Single die versions

### Godbound
- `gb` → 1d20 gb (damage chart: 1-=0, 2-5=1, 6-9=2, 10+=4)
- `gbs` → 1d20 gbs (straight damage, no chart)
- `gb 3d8` → 3d8 gb (3d8 with damage chart)
- `gbs 2d10 +5` → 2d10 straight damage +5

### Warhammer 40k Wrath & Glory
- `wng 4d6` → 4d6 with wrath die and success counting
- `wng dn3 5d6` → 5d6 with difficulty 3 (shows PASS/FAIL)
- `wng 4d6 !soak` → 4d6 soak roll (uses total, not successes)
- `wng dn4 6d6 !exempt` → 6d6 exempt test without wrath die

### Dark Heresy 2nd Edition
- `dh 4d10` → 4d10 ie10 (righteous fury on natural 10s)

### Other Popular Systems
- **Shadowrun**: `sr6` → 6d6 t5 (6th edition)
- **Exalted**: `ex5` → 5d10 t7 t10, `ex5t8` → 5d10 t8 t10
- **Fudge/FATE**: `3df` → 3d3 fudge (shows +/blank/- symbols)
- **AGE System**: `age` → 2d6 + 1d6 (Dragon dice)
- **Year Zero**: `6yz` → 6d6 t6
- **Warhammer 40k/AoS**: `3wh4+` → 3d6 t4
- **Earthdawn**: `ed15` → Earthdawn step 15 (ed1 through ed50)
- **Earthdawn 4e**: `ed4e15` → 4th edition step 15
- **Double Digit**: `dd34` → 1d3*10 + 1d4 (d66-style)
- **Storypath**: `sp4` → 4d10 t8 ie10
- **Sunsails**: `snm5` → 5d6 ie6 t4
- **D6 System**: `d6s4` → 4d6 + 1d6 ie

## System-Specific Examples

### Percentile Systems (Call of Cthulhu, etc.)
```text
/roll +d%    # Advantage: keep lower tens digit
/roll -d%    # Disadvantage: keep higher tens digit
/roll d%     # Standard percentile roll
```

### Fudge/FATE Dice
```text
/roll 3df    # 3 Fudge dice: + (plus), (blank), - (minus)
/roll 4df    # Standard FATE roll
```

### Hero System Damage
```text
/roll 2hsn      # 2d6 normal damage
/roll 3hsk      # 3d6 killing: shows BODY and STUN (BODY × 1d3)
/roll 2.5hsk    # 2d6 + 1d3 killing damage
/roll 3hsh      # 3d6 to-hit (roll under 11 + OCV - DCV)
```

### Godbound Damage
```text
/roll gb        # d20 with damage chart conversion
/roll gbs       # d20 straight damage (no chart)
/roll 3d8 gb    # Each die converted: 1-=0, 2-5=1, 6-9=2, 10+=4
/roll 2d10 gbs  # Straight damage total
```

### Wrath & Glory Tests
```text
/roll wng 4d6              # Standard test with wrath die
/roll wng dn3 5d6          # Difficulty 3 test
/roll wng 4d6 !soak        # Soak test (total damage, not successes)
/roll wng dn2 6d6 !exempt  # Exempt test without wrath die
```

### Complex Examples
```text
/roll 10d6 e6 k8 +4                                # 10d6, explode 6s, keep 8 highest, +4
/roll p 3d6 + 2d4                                  # Private roll
/roll s 4d6 k3                                     # Simple output (total only)
/roll (Fireball) 8d6 ! AOE                         # Labeled roll with comment
/roll 6 4d6                                        # 6 sets of 4d6
/roll 4d100 ; 10d6 e6 k8 +4; 3d10 k2; ul 3d100     # Four separate rolls
/roll 4d10 t8 ie10 f1                              # Chronicles of Darkness with botches
```

## Help Commands
- `/roll help` or `/help` - Basic dice syntax help
- `/roll help alias` or `/help alias` - Game system aliases
- `/roll help system` or `/help system` - Detailed system examples
- `/roll donate` - Support information
- `/purge X` - Purge recent messages in channel
