use std::env;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VoiceConfig {
    pub voice: String,
    pub speed: u32,
    pub pitch: u8,
    pub gain: u8,
    pub word_gap: u16,
}

impl VoiceConfig {
    pub fn from_env() -> Self {
        Self {
            voice: env::var("CONCILIUM_VOICE").unwrap_or_else(|_| "en-us".to_string()),
            speed: read_env_u32("CONCILIUM_VOICE_SPEED", 175),
            pitch: read_env_u8("CONCILIUM_VOICE_PITCH", 50),
            gain: read_env_u8("CONCILIUM_VOICE_GAIN", 100),
            word_gap: read_env_u16("CONCILIUM_VOICE_GAP", 0),
        }
    }
}

pub struct VoiceEngine;

impl VoiceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn speak(&self, ipa_string: &str) -> Result<(), String> {
        // espeak-ng -v en-us -q --ipa=([ipa_string])
        // Wait, espeak-ng usually takes IPA if wrapped in [[ ]]
        let formatted_ipa = format!("[[{}]]", ipa_string);

        let config = VoiceConfig::from_env();
        // Validate obvious bounds to avoid espeak-ng failures.
        if config.pitch > 99 {
            return Err("CONCILIUM_VOICE_PITCH must be 0-99".to_string());
        }
        if config.gain > 200 {
            return Err("CONCILIUM_VOICE_GAIN must be 0-200".to_string());
        }
        
        let status = Command::new("espeak-ng")
            .arg("-v")
            .arg(&config.voice) // We use English as a base voice for phonemes
            .arg("-s")
            .arg(config.speed.to_string())
            .arg("-p")
            .arg(config.pitch.to_string())
            .arg("-a")
            .arg(config.gain.to_string())
            .arg("-g")
            .arg(config.word_gap.to_string())
            .arg(formatted_ipa)
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "espeak-ng not found. Please install it using your package manager (e.g., `sudo apt install espeak-ng` on Debian/Ubuntu).".to_string()
                } else {
                    format!("failed to execute espeak-ng: {}", e)
                }
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("espeak-ng exited with status: {}", status))
        }
    }
}

fn read_env_u8(key: &str, default: u8) -> u8 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn read_env_u16(key: &str, default: u16) -> u16 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn read_env_u32(key: &str, default: u32) -> u32 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
