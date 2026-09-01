# Changelog

## [1.6.3] - 2026-08-31

## Changed

- **No change to dice syntax, roll results, or output.** Every expression rolls exactly as
  it did in 1.6.2. This release is internal work, versioned on its own because of how much
  of it there is.
- Consolidated 60 blocks of duplicated roll-handling code that had built up as game systems
  were added one at a time, for a net reduction of about 640 lines. Each behavior now lives
  in one place, so a fix reaches every game system at once instead of only the copies
  someone remembers to update.
- Adding a new game system now takes noticeably less repetitive setup, which should shorten
  the turnaround on requests for new systems.
- An expression that adds dice together, such as `1d20 + 2d6 + 1d8`, now uses a single
  random number generator instead of building a separate one for each part. Dice behave
  identically — this was checked by rolling 100,000 of each and comparing the results — and
  each roll costs the bot slightly less work.

## Added

- A regression suite covering 2,223 dice expressions, which pins everything you can see:
  how each expression is interpreted, and for every one that rolls, its exact result and
  exact Discord output. The 358 that are invalid have their exact error message pinned
  instead. A change that would alter any of it now fails the build rather than reaching
  your server.
- Every game system is included. Systems whose dice are fixed by the rules rather than by
  what you type — Savage Worlds' trait die, Wrath & Glory's pool — are now pinned exactly
  too, which previously was not possible.
- Before release, the whole suite was run against both 1.6.2 and this version and the two
  compared: identical interpretation and error messages for all 2,223 expressions, and
  identical results for the 1,088 whose dice cannot vary. The rest were checked by rolling
  each one thousands of times on both versions and confirming the outcomes matched.

## [1.6.2] - 2026-08-28

## Fixed

- Replaced the unmaintained `dotenv` crate with `dotenvy` (RUSTSEC-2021-0141); no change to `.env` handling
- Upgraded `sqlx` to 0.9 and `sysinfo` to 0.39; minimum supported Rust version is now 1.95

## [1.6.1] - 2026-08-25

## Fixed

- Substantially improved dice parsing performance across the board. Every roll is resolved
  faster, and long or modifier-heavy expressions that previously took a noticeable amount of
  time now complete almost instantly.
- Fixed an issue where sustained roll traffic could make the bot stop responding on a group
  of servers until it was restarted manually. Rolls no longer consume enough processing time
  to interfere with the bot's connection to Discord.

## Changed

- `/roll bot-info` now requires the Administrator permission. It reports process and server
  statistics intended for bot operators, and gathering them is expensive enough that it is no
  longer available on the public roll path.

## [1.6.0] - 2026-08-20

## Added

Finally caught up on over 2+ years of feature requests!

- Added Warhammer Fantasy RPG system
- Added syntax update to cyberpunk red

## [1.5.6] - 2026-8-19

## Added

- Added imploding dice (`i` / `i#`): the mirror of exploding dice, subtracting an extra die from the total (#145)

- Fixed Cyberpunk Red and Witcher critical failures displaying the subtracted die as a negative (e.g. `[1] - [-5]`), or as the Marvel logo when a 1 was subtracted

## [1.5.5] - 2026-08-16

## Added

- Added support for The Darkest House

## [1.5.4] - 2026-08-12

## Added

- Added support for Essence20 System

## [1.5.3] - 2026-08-09

## Added

- Added support for Open Legend System

## [1.5.2] - 2026-03-21

## Added

- Plotweaver: added Cosmere RPG plot die support (based on PR #180)

## [1.5.1] - 2026-02-24

## Added

- Resolved an issue related to fractional dice rolls with the Hero System
- Resolved an issue with manual startup of single shard deployments
- Added support for auto creation of databases
- Hero System normal damage (`hsn`) now displays calculated BODY and STUN values
- Fixed advantage/disadvantage rolls with modifiers failing when a comment was included (e.g. `+d20+5 ! testing`) (#175)

## [1.5.0] - 2025-08-24

## Added

- Enhanced dice randomness with cryptographically secure RNG using multiple entropy sources (OS, time, thread, process, memory)
- Support for Daggerheart
- Support for Wild Worlds
- Support for Mutants and Masterminds
- Support for Mothership
- Support for additional Exalted dice rolls

## [1.4.0] - 2025-07-05

## Added

- Made help commands respond in private message to reduce chat spam
- target lower modifier (tl) 
- keep middle modifier (km)
- double success modifier (t{num}ds{num})
- reroll greater (rg) and reroll greater indefinite (irg) modifiers
- DoS validation checks
- Guarantee of Discord 2000 character message limit compliance 
- Support for Cyberpunk RED
- Support for Witcher d10
- Support for additional wrath dice for wrath and glory
- Support for Cypher System
- Support for Brave New World
- Support for Conan
- Support for Silhouette
- Support for D6 Legends
- Support for World of Darkness Homebrew 10s cancel 1s
- Support for Vampire 5e
- Support for Laser and Feelings
- Support Level up D&D 5th Edition
- Support for Forged in the Dark
- "stability and performance improvements"

## [1.3.0] - 2025-07-03

## Added

- Added support for Marvel Multiverse game system

## [1.2.2] - 2025-07-02

## Added

- Resolved an issue related to WoD/CoD roll syntax

## [1.2.1] - 2025-06-29

## Added

- Resolved an issue with the d6 system alias

## [1.2.0] - 2025-06-29

### Added

- Start of new feature development!
- New alias added : Savage worlds /roll sw8
- Added critical glitch reporting to shadowrun

## [1.1.1] - 2025-06-28

### Added

- Updated to rust 2024

## [1.1.0] - 2025-06-23

### Added

- Bot is now at feature parity with the previous ruby version

## [1.0.0] - 2025-06-21

### Added

- Initial Commit :)
