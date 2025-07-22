# Dice Rolling Syntax

## Basic Usage
- `/roll 2d6` - Roll two six-sided dice
- `/roll 3d6 + 5` - Roll 3d6 and add 5
- `/roll 4d6 k3` - Roll 4d6, keep highest 3
- `/roll d%` or `/roll d100` - Roll percentile dice

### Core Modifiers
- **Exploding**: `e6` (explode on 6), `e` (explode on max), `ie6` (explode indefinitely)
- **Keep/Drop**: `k3` (keep 3 highest), `kl2` (keep 2 lowest), `km2` (keep 2 middle), `d1` (drop 1 lowest)
- **Rerolls**: `r2` (reroll ≤2 once), `ir2` (reroll ≤2 indefinitely), `rg2` (reroll ≥ 2 once), `irg2` (reroll ≥ 2 indefinitely)
- **Success Counting**: `t7` (count successes ≥7), `tl6` (count successes ≤6), `f1` (count failures ≤1)
- **Double Success**: `t7ds10` (target 7+, double success on 10; defaults to target value), `tl6ds5` (target 6-, double success on 5)
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

### Alien RPG (Year Zero Engine)
- `alien4` → 4d6 alien (4 base dice, count 6s as successes)
- `alien5s2` → 5d6 alien + 2d6 aliens2 (5 base + 2 stress dice)
- `alien6s3p` → alien6s4 (push roll - increases stress from 3 to 4)
- **Base Dice**: Roll d6s equal to attribute + skill, count 6s as successes
- **Stress Dice**: Add more dice but 1s trigger panic rolls
- **Push Mechanics**: Add 'p' to stress aliases to push (increases stress by 1)
- **Panic System**: Automatic 1d6 + stress level when 1s appear on stress dice
- **Stress Levels**: 1-10 maximum, higher levels = more effective but more dangerous

**Examples:**
- `alien4` → 4 base dice for attribute + skill roll
- `alien5s1` → 5 base dice + 1 stress die
- `alien3s2p` → Push previous roll (becomes alien3s3)
- `3 alien4s2` → Roll 3 sets of 4 base + 2 stress dice

**Panic Table Results:**
- 1-6: Keeping it together
- 7: Tremble (-2 to next roll)
- 8: Drop item
- 9: Freeze (lose next turn)
- 10: Seek cover
- 11: Scream (others must panic roll)
- 12: Flee from threat
- 13: Berserk attack
- 14: Catatonic
- 15+: Heart attack (Broken)

**Mechanics:**
- Success on 6s (Year Zero Engine standard)
- Stress dice add successes but risk panic on 1s
- Cannot push if stress dice show 1s
- Automatic panic interpretation with results

### Savage Worlds
- `sw4` → 1d4 trait + 1d6 wild, keep highest, both explode
- `sw6` → 1d6 trait + 1d6 wild, keep highest, both explode
- `sw8` → 1d8 trait + 1d6 wild, keep highest, both explode
- `sw10` → 1d10 trait + 1d6 wild, keep highest, both explode
- `sw12` → 1d12 trait + 1d6 wild, keep highest, both explode
- Snake Eyes: Critical failure when both dice roll natural

### D6 Legends (Success-based with Wild Die)
- `1d6l` → 1d6 t4f1ie6 (wild die only)
- `8d6l` → 7d6 t4 + 1d6 t4f1ie6 (7 regular + 1 wild)
- Regular dice: Roll d6, count successes on 4-6
- Wild die: Roll d6, count successes on 4-6, explode on 6, subtract 1 success for each 1
- Target number is always 4 (no variable difficulty)
- Used in DC Universe RPG, Hercules & Xena, and other D6 Legend system games

### Conan
- `conan` → 2d20 skill roll (default)
- `conan4` → 4d20 skill roll (momentum spent for extra dice)
- `cd` → 1d6 combat dice (default)
- `cd4` → 4d6 combat dice (massive damage)
- `conan3cd5` → 3d20 skill + 5d6 combat (combined attack)
- Skill Rolls: Roll d20s, count successes (target number varies by difficulty)
- Combat Dice: Special interpretation - 1=1 damage, 2=2 damage, 3-4=0 damage, 5-6=1 damage + special effect

### Silhouette System (Dream Pod 9)
- `sil` → 1d6 Silhouette (default)
- `sil3` → 3d6 Silhouette (skilled level)
- `sil5` → 5d6 Silhouette (superhuman level)
- `sil10` → 10d6 Silhouette (maximum variant)
- Roll skill dice, keep highest, each extra 6 adds +1
- Used in Heavy Gear, Jovian Chronicles, Tribe 8, Gear Krieg

### D&D/Pathfinder
- `dndstats` → 6 4d6 k3 (ability score generation)
- `attack +5` → 1d20 +5 (attack roll)
- `skill -2` → 1d20 -2 (skill check)
- `save +3` → 1d20 +3 (saving throw)
- `+d20` → 2d20 k1 (advantage)
- `-d20` → 2d20 kl1 (disadvantage)
- `+d%` → 2d10 kl1 * 10 + 1d10 - 10 (percentile advantage)
- `-d%` → 2d10 k1 * 10 + 1d10 - 10 (percentile disadvantage)

### Level Up Advanced 5th Edition (A5E)
- `a5e +5 ex1` → 1d20+5 + 1d4 (skill check with expertise level 1)
- `a5e +7 ex2` → 1d20+7 + 1d6 (skill check with expertise level 2)
- `a5e +3 ex3` → 1d20+3 + 1d8 (skill check with expertise level 3)
- `+a5e +5 ex1` → 2d20 k1+5 + 1d4 (advantage with expertise)
- `-a5e +3 ex2` → 2d20 kl1+3 + 1d6 (disadvantage with expertise)
- `a5e ex1` → 1d20 + 1d4 (expertise without modifier)
- `a5e +2 ex6` → 1d20+2 + 1d6 (explicit d6 expertise die)
- `a5e +1 ex8` → 1d20+1 + 1d8 (explicit d8 expertise die)
- `a5e +3 ex10` → 1d20+3 + 1d10 (house rule: d10 expertise)

**Expertise Die Levels:**
- `ex1` → d4 (Level 1 expertise)
- `ex2` → d6 (Level 2 expertise)
- `ex3` → d8 (Level 3 expertise)
- `ex4`, `ex6`, `ex8`, `ex10`, `ex12`, `ex20`, `ex100` → Explicit die size

### World of Darkness / Chronicles of Darkness
- `4cod` → 4d10 t8 ie10 (Chronicles of Darkness)
- `4cod8` → 4d10 t8 ie8 (8-again rule)
- `4cod9` → 4d10 t8 ie9 (9-again rule)
- `4codr` → 4d10 t8 ie10 r7 (rote quality)
- `4wod8` → 4d10 f1 t8 (World of Darkness, difficulty 8)
- `4wod8c` → 4d10 f1 t8 c (10s cancel 1s)

### Vampire: The Masquerade 5th Edition
- `vtm7h2` - 7 dice pool with 2 hunger dice
- `vtm5h0` - 5 dice pool with no hunger
- Critical Success: Pairs of 10s = 4 successes each pair
- Messy Critical: Critical success with hunger dice showing 10s
- Bestial Failure: Failed roll with hunger dice showing 1s
- Regular Failure: Failed roll without hunger 1s

### Cyberpunk Red
- `cpr` → 1d10 cpr (critical success on 10, critical failure on 1)
- `cpr + 5` → 1d10 cpr + 5 (with modifier)
- Critical Success (10): Roll another d10 and add to total
- Critical Failure (1): Roll another d10 and subtract from total
- Each explosion happens only once per roll

### Cypher System
- `/roll cs 1` - Level 1 task (target 3+, routine)
- `/roll cs 3` - Level 3 task (target 9+, typical)
- `/roll cs 6` - Level 6 task (target 18+, demanding)
- Special Results: 1=GM Intrusion, 17-19=Minor Effect, 20=Major Effect

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

### Forged in the Dark
- `fitd1` → 1d6 FitD (1 die action roll)
- `fitd2` → 2d6 FitD (2 dice action roll)
- `fitd0` → 2d6 FitD zero dice (desperate position - take lowest)
- Result System: Roll pool of d6s, take highest single die
- 1-3: Failure (bad outcome)
- 4-5: Partial Success (consequences)
- 6: Success (you do it well)
- Multiple 6s: Critical Success (extra advantage)
- Zero Dice: Roll 2d6, take lowest (desperate situation)

### Brave New World Pool System
- `bnw3` → 3-die pool: roll 3d6, take highest die, 6s explode
- `bnw4 + 2` → 4-die pool with +2 modifier applied to final result

### Warhammer 40k Wrath & Glory
- `wng 4d6` → 4d6 with wrath die and success counting
- `wng w2 4d6` → 4d6 with 2 wrath dice (unbound psyker powers)
- `wng dn3 5d6` → 5d6 with difficulty 3 (shows PASS/FAIL)
- `wng 4d6 !soak` → 4d6 soak roll (uses total, not successes)
- `wng dn4 6d6 !exempt` → 6d6 exempt test without wrath die

### Marvel Multiverse RPG
- `mm` → 3d6 Marvel Multiverse basic roll
- `mm e` → 3d6 with 1 edge (reroll lowest die, keep higher)
- `mm 2e` → 3d6 with 2 edges
- `mm t` → 3d6 with 1 trouble (reroll highest die, keep lower)
- `mm 3t` → 3d6 with 3 troubles
- `mm 2e 3t` → 3d6 with 2 edges and 3 troubles (net 1 trouble)
- Fantastic: Marvel die (middle) showing 1 becomes 6
- Edges and troubles cancel each other out

### Witcher d10 System
- `wit` → 1d10 wit (basic Witcher skill check)
- `wit + 5` → 1d10 wit with +5 modifier
- Critical Success (10): Roll another d10 and add to total
- Critical Failure (1): Roll another d10 and subtract from total
- Indefinite Explosions: If the additional die also shows 1 or 10, the explosion continues

### Dark Heresy 2nd Edition
- `dh 4d10` → 4d10 ie10 (righteous fury on natural 10s)

### Lasers & Feelings
- `2lf4` → 2d6 target 4 (generic - defaults to Lasers)
- `2lf4l` → 2d6 target 4 Lasers (success on ≤4)
- `2lf4f` → 2d6 target 4 Feelings (success on ≥4)
- **Target Range**: 2-5 (character's number from the game)
- **LASER FEELINGS**: Rolling exactly the target number gives special insight
- **Lasers** (science/tech): Count successes on rolls ≤ target
- **Feelings** (intuition/social): Count successes on rolls ≥ target

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
- **Storypath**: `sp4` → 4d10 t8 ie10, `sp4t6` → 4d10 t6 ie10 (custom target)
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

### Marvel Multiverse Examples
```text
/roll mm                   # Basic 3d6 Marvel Multiverse roll
/roll mm e                 # 3d6 with 1 edge (reroll lowest)
/roll mm t                 # 3d6 with 1 trouble (reroll highest)
/roll mm 2e 3t             # 2 edges + 3 troubles = net 1 trouble
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
