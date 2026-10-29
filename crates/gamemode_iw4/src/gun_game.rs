//! Host-configurable Gun Game progression shared by the UI and script runtime.

pub const PROGRESSION_DVAR: &str = "scr_gg_progression";
pub const RANDOM_SCOPE_DVAR: &str = "scr_gg_random_scope";
pub const WEAPONS_DVAR: &str = "scr_gg_weapons";
pub const POOL_DVAR: &str = "scr_gg_pool";
pub const FINAL_WEAPON_DVAR: &str = "scr_gg_final";

pub const DEFAULT_PROGRESSION: &str = "ordered";
pub const DEFAULT_RANDOM_SCOPE: &str = "shared";
pub const DEFAULT_FINAL_WEAPON: &str = "t6:weapon/hatchet_mp";

/// The curated IW4/T6 rotation used when the host has not supplied a list.
pub const DEFAULT_WEAPONS: &[&str] = &[
    "iw4:weapon/usp_mp",
    "iw4:weapon/beretta_mp",
    "iw4:weapon/deserteagle_mp",
    "iw4:weapon/coltanaconda_mp",
    "t6:weapon/beretta93r_mp",
    "t6:weapon/fiveseven_mp",
    "t6:weapon/fnp45_mp",
    "t6:weapon/kard_mp",
    "iw4:weapon/pp2000_mp",
    "iw4:weapon/tmp_mp",
    "iw4:weapon/glock_mp",
    "iw4:weapon/beretta393_mp",
    "iw4:weapon/mp5k_mp",
    "iw4:weapon/uzi_mp",
    "iw4:weapon/p90_mp",
    "iw4:weapon/kriss_mp",
    "iw4:weapon/ump45_mp",
    "t6:weapon/evoskorpion_mp",
    "t6:weapon/mp7_mp",
    "t6:weapon/pdw57_mp",
    "t6:weapon/qcw05_mp",
    "t6:weapon/vector_mp",
    "iw4:weapon/ranger_mp",
    "iw4:weapon/model1887_mp",
    "iw4:weapon/m1014_mp",
    "iw4:weapon/spas12_mp",
    "iw4:weapon/striker_mp",
    "iw4:weapon/aa12_mp",
    "t6:weapon/870mcs_mp",
    "t6:weapon/judge_mp",
    "t6:weapon/ksg_mp",
    "t6:weapon/saiga12_mp",
    "t6:weapon/srm1216_mp",
    "iw4:weapon/m4_mp",
    "iw4:weapon/famas_mp",
    "iw4:weapon/scar_mp",
    "iw4:weapon/tavor_mp",
    "iw4:weapon/fal_mp",
    "iw4:weapon/m16_mp",
    "iw4:weapon/masada_mp",
    "iw4:weapon/fn2000_mp",
    "iw4:weapon/ak47_mp",
    "t6:weapon/an94_mp",
    "t6:weapon/hk416_mp",
    "t6:weapon/insas_mp",
    "t6:weapon/sa58_mp",
    "t6:weapon/saritch_mp",
    "t6:weapon/scar_mp",
    "t6:weapon/sig556_mp",
    "t6:weapon/tar21_mp",
    "t6:weapon/type95_mp",
    "t6:weapon/xm8_mp",
    "iw4:weapon/rpd_mp",
    "iw4:weapon/sa80_mp",
    "iw4:weapon/mg4_mp",
    "iw4:weapon/m240_mp",
    "iw4:weapon/aug_mp",
    "t6:weapon/hamr_mp",
    "t6:weapon/lsat_mp",
    "t6:weapon/mk48_mp",
    "t6:weapon/qbb95_mp",
    "iw4:weapon/m21_mp",
    "t6:weapon/svu_mp",
    "iw4:weapon/wa2000_mp",
    "iw4:weapon/barrett_mp",
    "iw4:weapon/cheytac_mp",
    "t6:weapon/as50_mp",
    "t6:weapon/dsr50_mp",
    "t6:weapon/ballista_mp",
    "iw4:weapon/m79_mp",
    "iw4:weapon/at4_mp",
    "iw4:weapon/rpg_mp",
    "t6:weapon/m32_mp",
    "t6:weapon/smaw_mp",
    "t6:weapon/usrpg_mp",
    "t6:weapon/crossbow_mp",
    "iw4:weapon/throwingknife_mp",
    "t6:weapon/knife_ballistic_mp",
    DEFAULT_FINAL_WEAPON,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rotation_ends_with_the_default_random_final() {
        assert_eq!(DEFAULT_WEAPONS.len(), 79);
        assert_eq!(DEFAULT_WEAPONS.last().copied(), Some(DEFAULT_FINAL_WEAPON));
        assert!(
            !DEFAULT_WEAPONS
                .iter()
                .any(|weapon| weapon.contains("riotshield"))
        );
        assert_eq!(
            DEFAULT_WEAPONS
                .iter()
                .filter(|weapon| **weapon == DEFAULT_FINAL_WEAPON)
                .count(),
            1
        );
    }
}
