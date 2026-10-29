//! Kill Confirmed rules shared by the host UI, session bootstrap, and script.

pub const SCORE_LIMIT_DVAR: &str = "scr_kc_score_limit";
pub const TIME_LIMIT_DVAR: &str = "scr_kc_time_limit";
pub const TAG_LIFETIME_DVAR: &str = "scr_kc_tag_lifetime";
pub const PICKUP_SOUND_ALIAS: &str = "iw4l_kc_tag_pickup";
pub const FRIENDLY_PICKUP_SOUND_ALIAS: &str = "iw4l_kc_friendly_tag_pickup";

pub const DEFAULT_SCORE_LIMIT: u32 = 65;
pub const DEFAULT_TIME_LIMIT_MINUTES: u32 = 5;
pub const DEFAULT_TAG_LIFETIME_SECONDS: u32 = 30;

pub const MIN_SCORE_LIMIT: u32 = 1;
pub const MAX_SCORE_LIMIT: u32 = 255;
pub const MIN_TIME_LIMIT_MINUTES: u32 = 1;
pub const MAX_TIME_LIMIT_MINUTES: u32 = 1440;
pub const MIN_TAG_LIFETIME_SECONDS: u32 = 1;
pub const MAX_TAG_LIFETIME_SECONDS: u32 = 600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    pub score_limit: u32,
    pub time_limit_minutes: u32,
    pub tag_lifetime_seconds: u32,
}

impl Rules {
    pub const DEFAULT: Self = Self {
        score_limit: DEFAULT_SCORE_LIMIT,
        time_limit_minutes: DEFAULT_TIME_LIMIT_MINUTES,
        tag_lifetime_seconds: DEFAULT_TAG_LIFETIME_SECONDS,
    };

    pub const fn time_limit_ms(self) -> u32 {
        self.time_limit_minutes.saturating_mul(60_000)
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn score_limit(value: Option<&str>) -> u32 {
    parse_clamped(value, DEFAULT_SCORE_LIMIT, MIN_SCORE_LIMIT, MAX_SCORE_LIMIT)
}

pub fn time_limit_minutes(value: Option<&str>) -> u32 {
    parse_clamped(
        value,
        DEFAULT_TIME_LIMIT_MINUTES,
        MIN_TIME_LIMIT_MINUTES,
        MAX_TIME_LIMIT_MINUTES,
    )
}

pub fn tag_lifetime_seconds(value: Option<&str>) -> u32 {
    parse_clamped(
        value,
        DEFAULT_TAG_LIFETIME_SECONDS,
        MIN_TAG_LIFETIME_SECONDS,
        MAX_TAG_LIFETIME_SECONDS,
    )
}

pub fn normalize(score: Option<&str>, time: Option<&str>, lifetime: Option<&str>) -> Rules {
    Rules {
        score_limit: score_limit(score),
        time_limit_minutes: time_limit_minutes(time),
        tag_lifetime_seconds: tag_lifetime_seconds(lifetime),
    }
}

fn parse_clamped(value: Option<&str>, default: u32, min: u32, max: u32) -> u32 {
    value
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_iw4x_mode() {
        assert_eq!(
            Rules::DEFAULT,
            Rules {
                score_limit: 65,
                time_limit_minutes: 5,
                tag_lifetime_seconds: 30,
            }
        );
        assert_eq!(Rules::DEFAULT.time_limit_ms(), 300_000);
    }

    #[test]
    fn invalid_and_out_of_range_values_fall_back_or_clamp() {
        assert_eq!(normalize(None, None, None), Rules::DEFAULT);
        assert_eq!(score_limit(Some("not a number")), DEFAULT_SCORE_LIMIT);
        assert_eq!(time_limit_minutes(Some("0")), MIN_TIME_LIMIT_MINUTES);
        assert_eq!(tag_lifetime_seconds(Some("9999")), MAX_TAG_LIFETIME_SECONDS);
        assert_eq!(score_limit(Some(" 90 ")), 90);
    }
}
