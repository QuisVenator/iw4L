use crate::{dd, dom, ffa, kind::GameModeKind};

/// Where a mode's script extension runs relative to the retail gametype and map
/// scripts. The order is deliberate: a mode can replace gametype setup before a
/// map runs, or add callbacks just before `codecallback_startgametype` fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScriptHook {
    AfterGametype,
    AfterMap,
    BeforeGametypeCallbacks,
}

/// A GSC module owned by a mode and the function that starts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScriptExtension {
    pub module: &'static str,
    pub entry: &'static str,
    pub hook: ScriptHook,
}

/// Per-mode rule overrides. `None` keeps the base [`GameModeKind`] behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ModeRuleHooks {
    pub score_limit: Option<i32>,
    pub time_limit_ms: Option<u32>,
}

impl ModeRuleHooks {
    pub const NONE: Self = Self {
        score_limit: None,
        time_limit_ms: None,
    };
}

/// The mode-facing extension surface. Keep this small and declarative: modes
/// describe what the engine should run, while retail rules stay in the base
/// [`GameModeKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModeHooks {
    pub rules: ModeRuleHooks,
    pub scripts: &'static [ScriptExtension],
    pub killstreaks_enabled: bool,
    pub weapon_pickups_enabled: bool,
}

impl ModeHooks {
    pub const NONE: Self = Self {
        rules: ModeRuleHooks::NONE,
        scripts: &[],
        killstreaks_enabled: true,
        weapon_pickups_enabled: true,
    };
}

impl Default for ModeHooks {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModeSpec {
    pub token: &'static str,
    pub aliases: &'static [&'static str],
    pub display_name: &'static str,
    pub base: GameModeKind,
    pub host_selectable: bool,
    pub hooks: ModeHooks,
}

/// A selectable game mode. `ModeId` identifies the complete experience; `base`
/// identifies the retail rule set underneath it. Gun Game, for example, is its
/// own mode built on Free For All rather than a renamed Free For All.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ModeId {
    #[default]
    FreeForAll = 0,
    Demolition = 1,
    Domination = 2,
    TeamDeathmatch = 3,
    SearchAndDestroy = 4,
    CaptureTheFlag = 5,
    Headquarters = 6,
    Sabotage = 7,
    GunGame = 8,
    KillConfirmed = 9,
}

const DFS: &[&str] = &["dm", "ffa", "freeforall", "free_for_all", "deathmatch"];
const DEMOLITION: &[&str] = &["dd", "dem", "demolition"];
const DOMINATION: &[&str] = &["dom", "domination"];
const TEAM_DEATHMATCH: &[&str] = &["war", "tdm", "teamdeathmatch", "team_deathmatch"];
const SEARCH_AND_DESTROY: &[&str] = &["sd", "searchanddestroy", "search_and_destroy"];
const CAPTURE_THE_FLAG: &[&str] = &["ctf", "capturetheflag", "capture_the_flag"];
const HEADQUARTERS: &[&str] = &["koth", "hq", "headquarters"];
const SABOTAGE: &[&str] = &["sab", "sabotage"];
const GUN_GAME: &[&str] = &["gun", "gungame", "gun_game", "gg"];
const KILL_CONFIRMED: &[&str] = &["kc", "killconfirmed", "kill_confirmed", "confirmed"];

const GUN_GAME_SCRIPTS: &[ScriptExtension] = &[ScriptExtension {
    module: "iw4l_modes/gun_game",
    entry: "main",
    hook: ScriptHook::AfterGametype,
}];

const KILL_CONFIRMED_SCRIPTS: &[ScriptExtension] = &[ScriptExtension {
    module: "iw4l_modes/kill_confirmed",
    entry: "main",
    hook: ScriptHook::AfterGametype,
}];

const MODE_SPECS: [ModeSpec; 10] = [
    ModeSpec {
        token: ffa::GAMETYPE_TOKEN,
        aliases: DFS,
        display_name: ffa::DISPLAY_NAME,
        base: GameModeKind::FreeForAll,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: dd::GAMETYPE_TOKEN,
        aliases: DEMOLITION,
        display_name: dd::DISPLAY_NAME,
        base: GameModeKind::Demolition,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: dom::GAMETYPE_TOKEN,
        aliases: DOMINATION,
        display_name: dom::DISPLAY_NAME,
        base: GameModeKind::Domination,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "war",
        aliases: TEAM_DEATHMATCH,
        display_name: "TEAM DEATHMATCH",
        base: GameModeKind::TeamDeathmatch,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "sd",
        aliases: SEARCH_AND_DESTROY,
        display_name: "SEARCH AND DESTROY",
        base: GameModeKind::SearchAndDestroy,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "ctf",
        aliases: CAPTURE_THE_FLAG,
        display_name: "CAPTURE THE FLAG",
        base: GameModeKind::CaptureTheFlag,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "koth",
        aliases: HEADQUARTERS,
        display_name: "HEADQUARTERS",
        base: GameModeKind::Headquarters,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "sab",
        aliases: SABOTAGE,
        display_name: "SABOTAGE",
        base: GameModeKind::Sabotage,
        host_selectable: true,
        hooks: ModeHooks::NONE,
    },
    ModeSpec {
        token: "gun",
        aliases: GUN_GAME,
        display_name: "GUN GAME",
        base: GameModeKind::FreeForAll,
        host_selectable: true,
        hooks: ModeHooks {
            rules: ModeRuleHooks::NONE,
            scripts: GUN_GAME_SCRIPTS,
            killstreaks_enabled: false,
            weapon_pickups_enabled: false,
        },
    },
    ModeSpec {
        token: "conf",
        aliases: KILL_CONFIRMED,
        display_name: "KILL CONFIRMED",
        base: GameModeKind::TeamDeathmatch,
        host_selectable: true,
        hooks: ModeHooks {
            rules: ModeRuleHooks::NONE,
            scripts: KILL_CONFIRMED_SCRIPTS,
            killstreaks_enabled: true,
            weapon_pickups_enabled: true,
        },
    },
];

impl ModeId {
    pub const ALL: [Self; 10] = [
        Self::FreeForAll,
        Self::Demolition,
        Self::Domination,
        Self::TeamDeathmatch,
        Self::SearchAndDestroy,
        Self::CaptureTheFlag,
        Self::Headquarters,
        Self::Sabotage,
        Self::GunGame,
        Self::KillConfirmed,
    ];

    pub const fn spec(self) -> &'static ModeSpec {
        &MODE_SPECS[self as usize]
    }

    pub const fn base(self) -> GameModeKind {
        self.spec().base
    }

    pub const fn token(self) -> &'static str {
        self.spec().token
    }

    pub const fn display_name(self) -> &'static str {
        self.spec().display_name
    }

    pub const fn host_selectable(self) -> bool {
        self.spec().host_selectable
    }

    pub const fn hooks(self) -> ModeHooks {
        self.spec().hooks
    }

    pub fn parse(s: &str) -> Option<Self> {
        let mut buf = [0u8; 32];
        let bytes = s.trim().as_bytes();
        if bytes.len() > buf.len() {
            return None;
        }
        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b.to_ascii_lowercase();
        }
        let token = core::str::from_utf8(&buf[..bytes.len()]).ok()?;
        Self::ALL.into_iter().find(|mode| {
            mode.token().eq_ignore_ascii_case(token)
                || mode
                    .spec()
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(token))
        })
    }

    pub fn parse_ascii_ignore_case(s: &str) -> Option<Self> {
        Self::parse(s)
    }

    pub const fn wire_tag(self) -> u8 {
        self as u8
    }

    pub const fn from_wire_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::FreeForAll),
            1 => Some(Self::Demolition),
            2 => Some(Self::Domination),
            3 => Some(Self::TeamDeathmatch),
            4 => Some(Self::SearchAndDestroy),
            5 => Some(Self::CaptureTheFlag),
            6 => Some(Self::Headquarters),
            7 => Some(Self::Sabotage),
            8 => Some(Self::GunGame),
            9 => Some(Self::KillConfirmed),
            _ => None,
        }
    }

    pub const fn from_kind(kind: GameModeKind) -> Self {
        match kind {
            GameModeKind::FreeForAll => Self::FreeForAll,
            GameModeKind::Demolition => Self::Demolition,
            GameModeKind::Domination => Self::Domination,
            GameModeKind::TeamDeathmatch => Self::TeamDeathmatch,
            GameModeKind::SearchAndDestroy => Self::SearchAndDestroy,
            GameModeKind::CaptureTheFlag => Self::CaptureTheFlag,
            GameModeKind::Headquarters => Self::Headquarters,
            GameModeKind::Sabotage => Self::Sabotage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gun_game_is_ffa_with_a_script_hook() {
        let spec = ModeId::GunGame.spec();
        assert_eq!(spec.base, GameModeKind::FreeForAll);
        assert_eq!(spec.token, "gun");
        assert_eq!(spec.hooks.scripts.len(), 1);
        assert!(!spec.hooks.killstreaks_enabled);
        assert!(!spec.hooks.weapon_pickups_enabled);
        assert!(ModeHooks::default().killstreaks_enabled);
        assert!(ModeHooks::default().weapon_pickups_enabled);
    }

    #[test]
    fn kill_confirmed_is_tdm_with_its_own_script_hook() {
        let spec = ModeId::KillConfirmed.spec();
        assert_eq!(spec.base, GameModeKind::TeamDeathmatch);
        assert_eq!(spec.token, "conf");
        assert_eq!(ModeId::parse("kill_confirmed"), Some(ModeId::KillConfirmed));
        assert_eq!(ModeId::parse("kc"), Some(ModeId::KillConfirmed));
        assert_eq!(spec.hooks.scripts.len(), 1);
        assert_eq!(spec.hooks.scripts[0].hook, ScriptHook::AfterGametype);
        assert_eq!(spec.hooks.scripts[0].module, "iw4l_modes/kill_confirmed");
    }

    #[test]
    fn mode_tokens_round_trip_through_wire_tags() {
        for mode in ModeId::ALL {
            assert_eq!(ModeId::from_wire_tag(mode.wire_tag()), Some(mode));
            assert_eq!(ModeId::parse(mode.token()), Some(mode));
            assert_eq!(
                ModeId::parse(&mode.token().to_ascii_uppercase()),
                Some(mode)
            );
        }
    }

    #[test]
    fn every_host_mode_has_a_token_and_distinct_wire_tag() {
        for (index, mode) in ModeId::ALL.into_iter().enumerate() {
            assert!(mode.host_selectable());
            assert!(!mode.token().is_empty());
            assert_eq!(mode.wire_tag() as usize, index);
        }
    }
}
