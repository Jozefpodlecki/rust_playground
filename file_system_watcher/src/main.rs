use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::{env, fs::File, io::Write, path::{Path, PathBuf}};
use anyhow::{Ok, Result};
use log::*;

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    pub version: String
}

struct Context {
    pub settings_path: PathBuf
}

fn create_settings_if_not_exists(settings_file_name: &Path) -> Result<()> {
    let settings = Settings {
        version: "0.1.0".into(),
    };

    let json_data = serde_json::to_string(&settings)?;
    let mut file = File::create(settings_file_name)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}

fn watch<P: AsRef<Path>>(path: P) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    info!("Watching: {:?}", path.as_ref());
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            std::result::Result::Ok(event) => info!("Change: {event:?}"),
            Err(error) => error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn run(context: Context) -> Result<()> {
  
    create_settings_if_not_exists(&context.settings_path)?;
    watch(context.settings_path)?;

    Ok(())
}

fn create_context() -> Result<Context> {
    let executable_path = env::current_exe()?;
    let executable_directory = executable_path.parent().unwrap();
    let settings_file_name = "settings.json";
    let settings_path = executable_directory.join(settings_file_name);

    let context = Context {
        settings_path,
    };

    Ok(context)
}

fn main() {
    SimpleLogger::new().env().init().unwrap();
    
    match create_context().and_then(run) {
        Err(error) => error!("Error: {error:?}"),
        _ => {}
    }
}
