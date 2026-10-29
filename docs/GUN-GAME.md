# Gun Game progression

`make gun-game` starts the `gun` game mode, a free-for-all with an 81-level
progression drawn from the weapons available in the installed IW4 and T6
content. The same mode is selectable as `GUN GAME` in the host lobby. The host
UI and lobby protocol use the token `gun`; retail scripts still run the `dm`
base rules underneath. Levels rise from pistols
through stronger primaries, then reverse into precision weapons, launchers,
special weapons and finally throwing weapons.

| Levels | Tier | Weapons |
|---|---|---|
| 1–8 | Pistols | USP .45, M9, Desert Eagle, .44 Magnum; T6: M93R, Five-Seven, FNP45, KAP-40 |
| 9–12 | Machine pistols | PP2000, TMP, G18, M93 Raffica |
| 13–22 | SMGs | MP5K, Uzi, P90, Vector, UMP45; T6: Skorpion EVO, MP7, PDW-57, QCW-05, Vector K10 |
| 23–33 | Shotguns | Ranger, Model 1887, M1014, SPAS-12, Striker, AA-12; T6: 870 MCS, Judge, KSG, Saiga 12, SRM 1216 |
| 34–52 | Assault rifles | M4A1, FAMAS, SCAR-H, TAR-21, FAL, M16A4, Masada, F2000, AK-47; T6: AN-94, HK416, INSAS, SA58, SARITCH, SCAR-H, SIG556, TAR-21, Type 95, XM8 |
| 53–61 | LMGs | RPD, L86 LSW, MG4, M240, AUG HBAR; T6: HAMR, LSAT, MK 48, QBB LSW |
| 62–63 | DMRs | M21 EBR, SVU |
| 64–69 | Snipers | WA2000, Barrett .50cal, Intervention; T6: AS50, DSR 50, Ballista |
| 70–75 | Launchers | M79, AT4, RPG-7; T6: M32, SMAW, RPG |
| 76–78 | Special | T6 Crossbow, IW4 Riot Shield, T6 Riot Shield |
| 79–81 | Throwing weapons | Throwing Knife, Ballistic Knife, Combat Axe |

The same script names occur in both games for SCAR-H, TAR-21 and the Riot
Shield. Levels use `iw4:weapon/...` or `t6:weapon/...` so each entry resolves to
the intended game rather than the first matching script name.

Gun Game is registered in `gamemode_iw4::ModeId` as its own mode with
`GameModeKind::FreeForAll` as its base rules. Its GSC module is an
`AfterGametype` hook, so the retail `dm` gametype starts first, Gun Game layers
on top, and the map runs afterwards. Additional modes can declare the same
ordering, optional score/time rule overrides, and a display token without
adding another `IW4L_*` environment flag.

A normal kill advances the killer one level. A melee kill does not advance the
killer; instead the victim drops one level, stopping at the first weapon.
Suicides also drop the player one level, including unattributed environmental
deaths such as falling. Demoted players receive the correct weapon on their
next spawn. Killing with the final weapon starts the end-of-match sequence.

Killstreaks and dropped-weapon pickups are disabled for the whole match,
including debug-granted streaks. Players can still retrieve ammunition for the
weapon they already hold. The in-game scoreboard keeps the kills column,
replaces the score column with the player's weapon level, and sorts players by
weapon level and then kills.

Lock-on-only launchers are excluded: the IW4 Stinger and Javelin and the T6
FHJ-18 AA cannot fire until a lock exists, which makes them unsuitable for the
prototype's launcher tier. AT4, RPG-7, M32, SMAW and the T6 RPG remain.

With cheats enabled, `gungame_advance` (alias `gg_advance`) advances the local
player one level and immediately re-equips that level's weapon. Run it again to
step through the list while testing.

The T6 list covers the 40 selectable weapons found in the local installed
content. Store DLC weapons not present in that installation are not included.
