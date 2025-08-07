use std::{collections::HashSet, env, path::PathBuf, time::Duration};
use serde::{Deserialize, Deserializer};

// #[derive(Debug, Clone)]
// pub struct RunArgs {
//     pub cipher_key: Vec<u8>,
//     pub aes_xor_key: Vec<u8>,
//     pub game_path: PathBuf,
//     pub output_path: PathBuf,
//     pub exe_path: PathBuf,
//     pub exe_args: Vec<String>,
//     pub strategy: WaitStrategy
// }
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub log_level: String,

    #[serde(deserialize_with = "deserialize_str_to_vec")]
    pub cipher_key: Vec<u8>,

    #[serde(deserialize_with = "deserialize_hex")]
    pub aes_xor_key: Vec<u8>,

    #[serde(deserialize_with = "deserialize_pathbuf_with_env")]
    pub game_path: PathBuf,

    #[serde(deserialize_with = "deserialize_pathbuf_with_env")]
    pub output_path: PathBuf,
    pub exe_paths: Vec<ExeInfo>,

    pub process_dumper: ProcessDumperConfig,
    pub disassembler: DisassemblerConfig,
    pub cleanup: CleanupConfig
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessDumperConfig {
    pub is_enabled: bool,
    pub exe_paths: Vec<ExeInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisassemblerConfig {
    pub is_enabled: bool,
    pub filter: HashSet<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct CleanupConfig {
    pub is_enabled: bool,
    pub folders: HashSet<String>,
    pub files: HashSet<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExeInfo {
    pub is_enabled: bool,
    #[serde(deserialize_with = "deserialize_pathbuf_with_env")]
    pub path: PathBuf,
    pub args: Vec<String>,
    pub launch_method: LaunchMethod,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum LaunchMethod {
    Wait { #[serde(deserialize_with = "deserialize_duration")] wait: Duration },
    Monitor { #[serde(deserialize_with = "deserialize_hex_u64")] monitor: u64 },
}

impl AppConfig {
    pub fn new() -> anyhow::Result<Self> {
  
        let bytes = include_bytes!("./env.json");
        let mut config: AppConfig = serde_json::from_slice(bytes)?;

        if config.output_path.is_relative() {
            let current_exe = std::env::current_exe()?;
            let path = current_exe.parent().unwrap();
            let absolute = path.join(&config.output_path);
            std::fs::create_dir_all(&absolute)?;
            config.output_path = absolute.canonicalize()?;
        }
      
        config.exe_paths.retain_mut(|pr| {

            pr.is_enabled
        });

        Ok(config)
    }
}

fn deserialize_pathbuf_with_env<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: String = Deserialize::deserialize(deserializer)?;
    let expanded = expand_env_vars(&raw);
    Ok(PathBuf::from(expanded))
}

fn expand_env_vars(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut var = String::new();
            while let Some(&next) = chars.peek() {
                if next == '%' {
                    chars.next();
                    break;
                } else {
                    var.push(next);
                    chars.next();
                }
            }

            let expanded = env::var(&var).unwrap_or_else(|_| format!("%{}%", var));
            output.push_str(&expanded);
        } else {
            output.push(ch);
        }
    }

    output
}

fn deserialize_str_to_vec<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    

    let value: &str = Deserialize::deserialize(deserializer)?;
    Ok(value.as_bytes().to_vec())
}

fn deserialize_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: &str = Deserialize::deserialize(deserializer)?;
    hex::decode(value).map_err(D::Error::custom)
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_duration(&s).ok_or_else(|| serde::de::Error::custom("Invalid duration"))
}

fn deserialize_hex_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(|_| serde::de::Error::custom("Invalid hex string"))
}

fn parse_duration(s: &str) -> Option<Duration> {
    let parts: Vec<_> = s.split(':').collect();
    let seconds = match parts.as_slice() {
        [h, m, s] => h.parse::<u64>().ok()? * 3600 + m.parse::<u64>().ok()? * 60 + s.parse::<u64>().ok()?,
        [m, s] => m.parse::<u64>().ok()? * 60 + s.parse::<u64>().ok()?,
        [s] => s.parse::<u64>().ok()?,
        _ => return None,
    };
    Some(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config = AppConfig::new().unwrap();
        println!("{config:?}");
    }
}