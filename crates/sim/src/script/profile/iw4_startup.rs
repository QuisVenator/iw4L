use crate::script::source::SourceResolver;
use gamemode_iw4::{ModeId, ScriptHook};

/// Install runs the struct initializer itself, before the map's structs exist.
pub struct Iw4Startup {
    pub roots: Vec<String>,
    pub entries: Vec<String>,
}
impl Iw4Startup {
    pub fn new(resolver: &impl SourceResolver, gametype: &str, map: &str, mode: ModeId) -> Self {
        let gametype = format!("maps/mp/gametypes/{gametype}");
        let map = format!("maps/mp/{map}");
        let callbacks = "maps/mp/gametypes/_callbacksetup";
        let mut roots = vec![
            "codescripts/delete".to_owned(),
            "codescripts/struct".to_owned(),
            callbacks.to_owned(),
            gametype.clone(),
        ];
        let mut entries = vec![format!("{gametype}::main")];
        let extensions = mode.hooks().scripts;
        push_extensions(
            &mut roots,
            &mut entries,
            extensions,
            ScriptHook::AfterGametype,
        );
        if resolver.read_bytes(&map).is_ok() {
            entries.push(format!("{map}::main"));
            roots.push(map);
        }
        push_extensions(&mut roots, &mut entries, extensions, ScriptHook::AfterMap);
        let t6_equipment = "iw4l_t6/equipment";
        if resolver.read_bytes(t6_equipment).is_ok() {
            roots.push(t6_equipment.to_owned());
            entries.push(format!("{t6_equipment}::main"));
        }
        push_extensions(
            &mut roots,
            &mut entries,
            extensions,
            ScriptHook::BeforeGametypeCallbacks,
        );
        entries.push(format!("{callbacks}::codecallback_startgametype"));
        Self { roots, entries }
    }
}

fn push_extensions(
    roots: &mut Vec<String>,
    entries: &mut Vec<String>,
    extensions: &[gamemode_iw4::ScriptExtension],
    hook: ScriptHook,
) {
    for extension in extensions.iter().filter(|extension| extension.hook == hook) {
        if !roots.iter().any(|root| root == extension.module) {
            roots.push(extension.module.to_owned());
        }
        entries.push(format!("{}::{}", extension.module, extension.entry));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn gun_game_runs_after_the_base_gametype_and_before_the_map() {
        let sources = BTreeMap::from([
            ("maps/mp/gametypes/dm".to_owned(), String::new()),
            ("iw4l_modes/gun_game".to_owned(), String::new()),
            ("maps/mp/mp_rust".to_owned(), String::new()),
        ]);
        let startup = Iw4Startup::new(&sources, "dm", "mp_rust", ModeId::GunGame);
        assert_eq!(
            startup.entries,
            [
                "maps/mp/gametypes/dm::main",
                "iw4l_modes/gun_game::main",
                "maps/mp/mp_rust::main",
                "maps/mp/gametypes/_callbacksetup::codecallback_startgametype",
            ]
        );
    }

    #[test]
    fn a_base_mode_does_not_load_another_modes_scripts() {
        let sources = BTreeMap::from([
            ("maps/mp/gametypes/dm".to_owned(), String::new()),
            ("iw4l_modes/gun_game".to_owned(), String::new()),
        ]);
        let startup = Iw4Startup::new(&sources, "dm", "test_map", ModeId::FreeForAll);
        assert!(
            !startup
                .entries
                .iter()
                .any(|entry| entry.contains("gun_game"))
        );
    }

    #[test]
    fn kill_confirmed_runs_after_war_and_before_the_map() {
        let sources = BTreeMap::from([
            ("maps/mp/gametypes/war".to_owned(), String::new()),
            ("iw4l_modes/kill_confirmed".to_owned(), String::new()),
            ("maps/mp/mp_afghan".to_owned(), String::new()),
        ]);
        let startup = Iw4Startup::new(&sources, "war", "mp_afghan", ModeId::KillConfirmed);
        assert_eq!(
            startup.entries,
            [
                "maps/mp/gametypes/war::main",
                "iw4l_modes/kill_confirmed::main",
                "maps/mp/mp_afghan::main",
                "maps/mp/gametypes/_callbacksetup::codecallback_startgametype",
            ]
        );
    }

    #[test]
    fn team_gun_game_reuses_the_gun_game_script_over_war() {
        let sources = BTreeMap::from([
            ("maps/mp/gametypes/war".to_owned(), String::new()),
            ("iw4l_modes/gun_game".to_owned(), String::new()),
            ("maps/mp/mp_rust".to_owned(), String::new()),
        ]);
        let startup = Iw4Startup::new(&sources, "war", "mp_rust", ModeId::TeamGunGame);
        assert_eq!(
            startup.entries,
            [
                "maps/mp/gametypes/war::main",
                "iw4l_modes/gun_game::main",
                "maps/mp/mp_rust::main",
                "maps/mp/gametypes/_callbacksetup::codecallback_startgametype",
            ]
        );
    }
}
