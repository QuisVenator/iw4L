const PICKUP_PCM: &[u8] = include_bytes!("../assets/kill_confirmed_pickup/coin_received.pcm");
const FRIENDLY_PICKUP_PCM: &[u8] =
    include_bytes!("../assets/kill_confirmed_pickup/coin_received_reverse.pcm");
const PICKUP_RATE: u32 = 48_000;
const PICKUP_CHANNELS: u16 = 2;
const PICKUP_VOLUME: f32 = 0.5;

pub(crate) fn install(catalog: &mut asset_audio::SoundCatalog) -> Result<String, String> {
    for (alias, pcm) in [
        (gamemode_iw4::kill_confirmed::PICKUP_SOUND_ALIAS, PICKUP_PCM),
        (
            gamemode_iw4::kill_confirmed::FRIENDLY_PICKUP_SOUND_ALIAS,
            FRIENDLY_PICKUP_PCM,
        ),
    ] {
        catalog
            .install_pcm16_alias(
                asset_audio::ZoneGame::Iw4,
                alias,
                PICKUP_RATE,
                PICKUP_CHANNELS,
                pcm.to_vec(),
                PICKUP_VOLUME,
            )
            .map_err(|error| format!("Kill Confirmed pickup sound `{alias}`: {error}"))?;
    }
    Ok(format!(
        "Kill Confirmed pickup sounds: `{}` and `{}` embedded, 48 kHz stereo PCM, {:.2}s/{:.2}s",
        gamemode_iw4::kill_confirmed::PICKUP_SOUND_ALIAS,
        gamemode_iw4::kill_confirmed::FRIENDLY_PICKUP_SOUND_ALIAS,
        duration_seconds(PICKUP_PCM),
        duration_seconds(FRIENDLY_PICKUP_PCM),
    ))
}

fn duration_seconds(pcm: &[u8]) -> f32 {
    pcm.len() as f32 / (PICKUP_RATE as f32 * f32::from(PICKUP_CHANNELS) * 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_pickup_is_complete_16_bit_stereo_pcm() {
        for pcm in [PICKUP_PCM, FRIENDLY_PICKUP_PCM] {
            assert!(!pcm.is_empty());
            assert!(pcm.len().is_multiple_of(usize::from(PICKUP_CHANNELS) * 2));
        }
        let frame_bytes = usize::from(PICKUP_CHANNELS) * 2;
        let pickup_frames = PICKUP_PCM.len() / frame_bytes;
        let friendly_frames = FRIENDLY_PICKUP_PCM.len() / frame_bytes;
        assert!(friendly_frames.abs_diff(pickup_frames / 3) <= 1);
        assert_ne!(PICKUP_PCM, FRIENDLY_PICKUP_PCM);
    }
}
