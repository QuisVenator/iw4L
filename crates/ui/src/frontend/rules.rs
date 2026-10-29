use frame::UiMenuDvars;

use crate::MenuMapList;

const MAX_GUN_GAME_WEAPONS: usize = 128;
const MAX_GUN_GAME_LIST_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunGameProgression {
    Ordered,
    Random,
}

impl GunGameProgression {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Ordered => "ordered",
            Self::Random => "random",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunGameRandomScope {
    Shared,
    PerPlayer,
}

impl GunGameRandomScope {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::PerPlayer => "perplayer",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GunGameSetup {
    pub progression: GunGameProgression,
    pub random_scope: GunGameRandomScope,
    pub ordered_weapons: Vec<String>,
    pub random_pool: Vec<String>,
    pub final_weapon: String,
}

impl GunGameSetup {
    pub fn summary(&self) -> String {
        match self.progression {
            GunGameProgression::Ordered => {
                format!(
                    "Saved {} ordered weapon levels.",
                    self.ordered_weapons.len()
                )
            }
            GunGameProgression::Random => {
                let playable = self
                    .random_pool
                    .iter()
                    .filter(|weapon| **weapon != self.final_weapon)
                    .count();
                format!(
                    "Saved {playable} random choices; final weapon is always {}.",
                    self.final_weapon
                )
            }
        }
    }
}

pub fn selected_game(
    dvars: &UiMenuDvars,
    maps: &MenuMapList,
) -> Result<(String, sim::HostGameModeSelection), String> {
    let map = dvars.get("ui_mapname").ok_or("No map selected")?;
    if !maps.contains(map) {
        return Err(format!("Map `{map}` is not installed"));
    }
    let mode = dvars
        .get("ui_gametype")
        .and_then(sim::HostGameModeSelection::from_token)
        .ok_or("Unsupported game mode")?;
    Ok((map.to_owned(), mode))
}

pub const MATCH_CONFIG: &str = "default_xboxlive.cfg";

fn is_rule(name: &str) -> bool {
    name.starts_with("scr_")
        || name.starts_with("koth_")
        || matches!(name, "g_hardcore" | "camera_thirdperson")
}

pub fn seed_rules(dvars: &mut UiMenuDvars, config: &str) {
    for line in config.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("set ") else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let (Some(name), value) = (words.next(), words.next().unwrap_or("")) else {
            continue;
        };
        let name = name.to_ascii_lowercase();
        if is_rule(&name) && dvars.get(&name).is_none() {
            dvars.set(&name, value.trim_matches('"'));
        }
    }
    for dvar in [
        sim::CONSTANT_RADAR_DVAR,
        sim::ENEMY_BOTS_DVAR,
        sim::FRIENDLY_BOTS_DVAR,
    ] {
        if dvars.get(dvar).is_none() {
            dvars.set(dvar, "0");
        }
    }
}

pub fn host_rules(dvars: &UiMenuDvars) -> frame::HostMatchRules {
    frame::HostMatchRules(
        dvars
            .iter()
            .filter(|(name, _)| is_rule(name))
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
    )
}

pub fn seed_kill_confirmed(dvars: &mut UiMenuDvars) {
    for (name, value) in default_kill_confirmed_values() {
        if dvars.get(name).is_none() {
            dvars.set(name, value);
        }
    }
    if dvars.get("ui_kc_status").is_none() {
        dvars.set("ui_kc_status", "");
    }
}

pub fn reset_kill_confirmed(dvars: &mut UiMenuDvars) {
    for (name, value) in default_kill_confirmed_values() {
        dvars.set(name, value);
    }
    dvars.set(
        "ui_kc_status",
        "Reset Kill Confirmed to 65 tags, 5 minutes, and 30 second tags.",
    );
}

pub fn normalize_kill_confirmed(dvars: &mut UiMenuDvars) -> Result<String, String> {
    let score_limit = parse_kill_confirmed_number(
        dvars,
        gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR,
        gamemode_iw4::kill_confirmed::DEFAULT_SCORE_LIMIT,
        gamemode_iw4::kill_confirmed::MIN_SCORE_LIMIT,
        gamemode_iw4::kill_confirmed::MAX_SCORE_LIMIT,
        "confirmed kill limit",
    )?;
    let time_limit = parse_kill_confirmed_number(
        dvars,
        gamemode_iw4::kill_confirmed::TIME_LIMIT_DVAR,
        gamemode_iw4::kill_confirmed::DEFAULT_TIME_LIMIT_MINUTES,
        gamemode_iw4::kill_confirmed::MIN_TIME_LIMIT_MINUTES,
        gamemode_iw4::kill_confirmed::MAX_TIME_LIMIT_MINUTES,
        "round time in minutes",
    )?;
    let tag_lifetime = parse_kill_confirmed_number(
        dvars,
        gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR,
        gamemode_iw4::kill_confirmed::DEFAULT_TAG_LIFETIME_SECONDS,
        gamemode_iw4::kill_confirmed::MIN_TAG_LIFETIME_SECONDS,
        gamemode_iw4::kill_confirmed::MAX_TAG_LIFETIME_SECONDS,
        "dog-tag lifetime in seconds",
    )?;

    dvars.set(
        gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR,
        score_limit.to_string(),
    );
    dvars.set(
        gamemode_iw4::kill_confirmed::TIME_LIMIT_DVAR,
        time_limit.to_string(),
    );
    dvars.set(
        gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR,
        tag_lifetime.to_string(),
    );
    let summary = format!(
        "Saved Kill Confirmed: {score_limit} tags to win, {time_limit} minute round, {tag_lifetime} second tags."
    );
    dvars.set("ui_kc_status", &summary);
    Ok(summary)
}

fn default_kill_confirmed_values() -> [(&'static str, String); 3] {
    [
        (
            gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR,
            gamemode_iw4::kill_confirmed::DEFAULT_SCORE_LIMIT.to_string(),
        ),
        (
            gamemode_iw4::kill_confirmed::TIME_LIMIT_DVAR,
            gamemode_iw4::kill_confirmed::DEFAULT_TIME_LIMIT_MINUTES.to_string(),
        ),
        (
            gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR,
            gamemode_iw4::kill_confirmed::DEFAULT_TAG_LIFETIME_SECONDS.to_string(),
        ),
    ]
}

fn parse_kill_confirmed_number(
    dvars: &UiMenuDvars,
    name: &str,
    default: u32,
    min: u32,
    max: u32,
    label: &str,
) -> Result<u32, String> {
    let value = dvars.get(name).unwrap_or_default().trim();
    if value.is_empty() {
        return Ok(default);
    }
    let number = value
        .parse::<u32>()
        .map_err(|_| format!("The {label} must be a whole number, not `{value}`"))?;
    if !(min..=max).contains(&number) {
        return Err(format!(
            "The {label} must be between {min} and {max}, not {number}"
        ));
    }
    Ok(number)
}

pub fn seed_gun_game(dvars: &mut UiMenuDvars) {
    for (name, value) in default_gun_game_values() {
        if dvars.get(name).is_none() {
            dvars.set(name, value);
        }
    }
    if dvars.get("ui_gun_game_status").is_none() {
        dvars.set("ui_gun_game_status", "");
    }
}

pub fn reset_gun_game(dvars: &mut UiMenuDvars) {
    for (name, value) in default_gun_game_values() {
        dvars.set(name, value);
    }
    dvars.set(
        "ui_gun_game_status",
        format!(
            "Reset to the default {}-level rotation.",
            gamemode_iw4::gun_game::DEFAULT_WEAPONS.len()
        ),
    );
}

pub fn normalize_gun_game(dvars: &mut UiMenuDvars) -> Result<String, String> {
    let progression = match dvars
        .get(gamemode_iw4::gun_game::PROGRESSION_DVAR)
        .unwrap_or(gamemode_iw4::gun_game::DEFAULT_PROGRESSION)
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "ordered" => GunGameProgression::Ordered,
        "random" => GunGameProgression::Random,
        value => {
            return Err(format!(
                "Gun Game progression must be `ordered` or `random`, not `{value}`"
            ));
        }
    };
    let random_scope = match dvars
        .get(gamemode_iw4::gun_game::RANDOM_SCOPE_DVAR)
        .unwrap_or(gamemode_iw4::gun_game::DEFAULT_RANDOM_SCOPE)
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "shared" => GunGameRandomScope::Shared,
        "perplayer" | "per_player" | "per-player" => GunGameRandomScope::PerPlayer,
        value => {
            return Err(format!(
                "Gun Game random scope must be `shared` or `perplayer`, not `{value}`"
            ));
        }
    };

    let (ordered_weapons, random_pool, final_weapon) = match progression {
        GunGameProgression::Ordered => {
            let text = dvars
                .get(gamemode_iw4::gun_game::WEAPONS_DVAR)
                .unwrap_or("");
            let ordered = parse_weapon_list(text, "ordered weapon list")?;
            if ordered.is_empty() {
                return Err("The ordered weapon list cannot be empty".into());
            }
            dvars.set(gamemode_iw4::gun_game::WEAPONS_DVAR, join_weapons(&ordered));
            (ordered, Vec::new(), String::new())
        }
        GunGameProgression::Random => {
            let pool = parse_weapon_list(
                dvars.get(gamemode_iw4::gun_game::POOL_DVAR).unwrap_or(""),
                "random weapon pool",
            )?;
            let final_weapon = parse_single_weapon(
                dvars
                    .get(gamemode_iw4::gun_game::FINAL_WEAPON_DVAR)
                    .unwrap_or(""),
                "final weapon",
            )?;
            if !pool.iter().any(|weapon| *weapon != final_weapon) {
                return Err(
                    "The random pool must contain a weapon other than the final weapon".into(),
                );
            }
            dvars.set(gamemode_iw4::gun_game::POOL_DVAR, join_weapons(&pool));
            dvars.set(
                gamemode_iw4::gun_game::FINAL_WEAPON_DVAR,
                final_weapon.clone(),
            );
            (Vec::new(), pool, final_weapon)
        }
    };
    dvars.set(
        gamemode_iw4::gun_game::PROGRESSION_DVAR,
        progression.token(),
    );
    dvars.set(
        gamemode_iw4::gun_game::RANDOM_SCOPE_DVAR,
        random_scope.token(),
    );
    let setup = GunGameSetup {
        progression,
        random_scope,
        ordered_weapons,
        random_pool,
        final_weapon,
    };
    let summary = setup.summary();
    dvars.set("ui_gun_game_status", &summary);
    Ok(summary)
}

fn default_gun_game_values() -> [(&'static str, String); 5] {
    let ordered = gamemode_iw4::gun_game::DEFAULT_WEAPONS.join(", ");
    let pool = gamemode_iw4::gun_game::DEFAULT_WEAPONS
        .iter()
        .copied()
        .filter(|weapon| *weapon != gamemode_iw4::gun_game::DEFAULT_FINAL_WEAPON)
        .collect::<Vec<_>>()
        .join(", ");
    [
        (
            gamemode_iw4::gun_game::PROGRESSION_DVAR,
            gamemode_iw4::gun_game::DEFAULT_PROGRESSION.into(),
        ),
        (
            gamemode_iw4::gun_game::RANDOM_SCOPE_DVAR,
            gamemode_iw4::gun_game::DEFAULT_RANDOM_SCOPE.into(),
        ),
        (gamemode_iw4::gun_game::WEAPONS_DVAR, ordered),
        (gamemode_iw4::gun_game::POOL_DVAR, pool),
        (
            gamemode_iw4::gun_game::FINAL_WEAPON_DVAR,
            gamemode_iw4::gun_game::DEFAULT_FINAL_WEAPON.into(),
        ),
    ]
}

fn parse_weapon_list(text: &str, field: &str) -> Result<Vec<String>, String> {
    if text.len() > MAX_GUN_GAME_LIST_BYTES {
        return Err(format!(
            "The {field} exceeds {MAX_GUN_GAME_LIST_BYTES} bytes"
        ));
    }
    let mut weapons = Vec::new();
    for token in text.split(|ch: char| ch == ',' || ch == ';' || ch.is_ascii_whitespace()) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let weapon = canonical_weapon(token)?;
        if !weapons.contains(&weapon) || field == "ordered weapon list" {
            weapons.push(weapon);
        }
        if weapons.len() > MAX_GUN_GAME_WEAPONS {
            return Err(format!(
                "The {field} may contain at most {MAX_GUN_GAME_WEAPONS} weapons"
            ));
        }
    }
    Ok(weapons)
}

fn parse_single_weapon(text: &str, field: &str) -> Result<String, String> {
    let weapons = parse_weapon_list(text, field)?;
    match weapons.as_slice() {
        [weapon] => Ok(weapon.clone()),
        [] => Err(format!("The {field} cannot be empty")),
        _ => Err(format!("The {field} must name exactly one weapon")),
    }
}

fn canonical_weapon(text: &str) -> Result<String, String> {
    let text = text.trim().to_ascii_lowercase();
    let (namespace, name) = text
        .split_once(':')
        .ok_or_else(|| format!("Weapon `{text}` must start with `iw4:` or `t6:`"))?;
    if !matches!(namespace, "iw4" | "t6") {
        return Err(format!(
            "Weapon `{text}` uses unsupported namespace `{namespace}`; use `iw4:` or `t6:`"
        ));
    }
    let name = name.strip_prefix("weapon/").unwrap_or(name);
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(format!(
            "Weapon `{text}` has an invalid script name; use `{namespace}:weapon/name_mp`"
        ));
    }
    Ok(format!("{namespace}:weapon/{name}"))
}

fn join_weapons(weapons: &[String]) -> String {
    weapons.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dvars() -> UiMenuDvars {
        let mut dvars = UiMenuDvars::default();
        seed_gun_game(&mut dvars);
        dvars
    }

    fn kill_confirmed_dvars() -> UiMenuDvars {
        let mut dvars = UiMenuDvars::default();
        seed_kill_confirmed(&mut dvars);
        dvars
    }

    #[test]
    fn kill_confirmed_defaults_and_reset_are_canonical() {
        let mut dvars = kill_confirmed_dvars();
        assert_eq!(
            normalize_kill_confirmed(&mut dvars).as_deref(),
            Ok("Saved Kill Confirmed: 65 tags to win, 5 minute round, 30 second tags.")
        );
        dvars.set(gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR, "90");
        dvars.set(gamemode_iw4::kill_confirmed::TIME_LIMIT_DVAR, "12");
        dvars.set(gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR, "45");
        reset_kill_confirmed(&mut dvars);
        assert_eq!(
            dvars.get(gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR),
            Some("65")
        );
        assert_eq!(
            dvars.get(gamemode_iw4::kill_confirmed::TIME_LIMIT_DVAR),
            Some("5")
        );
        assert_eq!(
            dvars.get(gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR),
            Some("30")
        );
    }

    #[test]
    fn kill_confirmed_values_are_range_checked() {
        let mut dvars = kill_confirmed_dvars();
        dvars.set(gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR, "0");
        assert!(
            normalize_kill_confirmed(&mut dvars)
                .unwrap_err()
                .contains("between 1 and 255")
        );
        dvars.set(gamemode_iw4::kill_confirmed::SCORE_LIMIT_DVAR, "65");
        dvars.set(gamemode_iw4::kill_confirmed::TAG_LIFETIME_DVAR, "soon");
        assert!(
            normalize_kill_confirmed(&mut dvars)
                .unwrap_err()
                .contains("whole number")
        );
    }

    #[test]
    fn seeded_order_is_the_curated_rotation() {
        let mut dvars = dvars();
        assert_eq!(
            normalize_gun_game(&mut dvars).as_deref(),
            Ok("Saved 79 ordered weapon levels.")
        );
        assert_eq!(
            dvars.get(gamemode_iw4::gun_game::WEAPONS_DVAR),
            Some(gamemode_iw4::gun_game::DEFAULT_WEAPONS.join(", ").as_str())
        );
    }

    #[test]
    fn random_configuration_is_canonical_and_keeps_a_fixed_final() {
        let mut dvars = dvars();
        dvars.set(gamemode_iw4::gun_game::PROGRESSION_DVAR, "random");
        dvars.set(gamemode_iw4::gun_game::RANDOM_SCOPE_DVAR, "per_player");
        dvars.set(
            gamemode_iw4::gun_game::POOL_DVAR,
            "IW4:usp_mp, t6:weapon/scar_mp; iw4:weapon/usp_mp",
        );
        dvars.set(gamemode_iw4::gun_game::FINAL_WEAPON_DVAR, "t6:hatchet_mp");

        let summary = normalize_gun_game(&mut dvars).unwrap();
        assert_eq!(
            dvars.get(gamemode_iw4::gun_game::POOL_DVAR),
            Some("iw4:weapon/usp_mp, t6:weapon/scar_mp")
        );
        assert_eq!(
            dvars.get(gamemode_iw4::gun_game::FINAL_WEAPON_DVAR),
            Some("t6:weapon/hatchet_mp")
        );
        assert_eq!(
            dvars.get(gamemode_iw4::gun_game::RANDOM_SCOPE_DVAR),
            Some("perplayer")
        );
        assert!(summary.contains("final weapon is always t6:weapon/hatchet_mp"));
    }

    #[test]
    fn ordered_mode_does_not_require_a_valid_random_pool() {
        let mut dvars = dvars();
        dvars.set(
            gamemode_iw4::gun_game::WEAPONS_DVAR,
            "iw4:usp_mp, t6:hatchet_mp",
        );
        dvars.set(gamemode_iw4::gun_game::POOL_DVAR, "not a weapon");
        assert_eq!(
            normalize_gun_game(&mut dvars).as_deref(),
            Ok("Saved 2 ordered weapon levels.")
        );
    }

    #[test]
    fn random_mode_requires_playable_pool_entries_besides_the_final() {
        let mut dvars = dvars();
        dvars.set(gamemode_iw4::gun_game::PROGRESSION_DVAR, "random");
        dvars.set(
            gamemode_iw4::gun_game::POOL_DVAR,
            gamemode_iw4::gun_game::DEFAULT_FINAL_WEAPON,
        );
        assert!(
            normalize_gun_game(&mut dvars)
                .unwrap_err()
                .contains("other than the final weapon")
        );
    }

    #[test]
    fn weapon_namespaces_are_validated() {
        let mut dvars = dvars();
        dvars.set(
            gamemode_iw4::gun_game::WEAPONS_DVAR,
            "iw4:usp_mp, t5:mp40_mp",
        );
        assert!(
            normalize_gun_game(&mut dvars)
                .unwrap_err()
                .contains("unsupported namespace")
        );
        assert!(canonical_weapon("usp_mp").is_err());
    }
}
