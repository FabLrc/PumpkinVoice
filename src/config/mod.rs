use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;
use std::{env, fs};
use tracing::{debug, warn};

const CONFIG_ROOT_FOLDER: &str = "plugins/pumpkin_voice/";

#[derive(Serialize, Deserialize, Clone)]
pub struct VoicechatConfig {
    pub port: i32,
    pub bind_address: String,
    pub max_voice_distance: f64,
    pub whisper_distance: f64,
    pub codec: String,
    pub mtu_size: i32,
    pub keep_alive: i32,
    pub enable_groups: bool,
    pub voice_host: String,
    pub allow_recording: bool,
    pub spectator_interaction: bool,
    pub spectator_player_possession: bool,
    pub force_voice_chat: bool,
    pub login_timeout: i32,
    pub broadcast_range: f64,
    pub allow_pings: bool,
}

impl Default for VoicechatConfig {
    fn default() -> Self {
        Self {
            port: 24454,
            bind_address: String::new(),
            max_voice_distance: 48.0,
            whisper_distance: 24.0,
            codec: "VOIP".to_string(), // VOIP, AUDIO, RESTRICTED_LOWDELAY
            mtu_size: 1024,
            keep_alive: 1000,
            enable_groups: true,
            voice_host: String::new(),
            allow_recording: true,
            spectator_interaction: false,
            spectator_player_possession: false,
            force_voice_chat: false,
            login_timeout: 10000,
            broadcast_range: -1.0,
            allow_pings: true,
        }
    }
}

pub static CONFIG: LazyLock<VoicechatConfig> = LazyLock::new(|| {
    let exec_dir = env::current_dir().unwrap();
    VoicechatConfig::load(&exec_dir)
});

impl LoadConfiguration for VoicechatConfig {
    fn get_path() -> &'static Path {
        Path::new("config.toml")
    }

    fn validate(&self) {}
}

trait LoadConfiguration {
    fn load(exec_dir: &Path) -> Self
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        let config_dir = exec_dir.join(CONFIG_ROOT_FOLDER);
        if !config_dir.exists() {
            debug!("creating new config root folder");
            fs::create_dir_all(&config_dir).expect("Failed to create config root folder");
        }
        let path = config_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {:?}", &path));

            toml::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {:?}. Reason: {}. This is probably caused by a config update; just delete the old config and start Pumpkin again",
                    &path,
                    err.message()
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(&path, toml::to_string(&content).unwrap()) {
                warn!(
                    "Couldn't write default config to {:?}. Reason: {}",
                    &path, err
                );
            }

            content
        };

        config.validate();
        config
    }

    fn get_path() -> &'static Path;

    fn validate(&self);
}
