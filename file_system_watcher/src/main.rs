use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::{env, fs::File, path::{Path, PathBuf}, sync::{Arc, RwLock}, time::Duration};
use anyhow::{Ok, Result};
use log::*;

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    pub version: String
}

struct Context {
    pub settings_path: PathBuf
}

fn read_settings(settings_file_name: &Path) -> Result<Settings> {
    let file = File::open(settings_file_name)?;
    let settings = serde_json::from_reader(file)?;

    Ok(settings)
}

fn create_settings_if_not_exists(settings_file_name: &Path) -> Result<Arc<RwLock<Settings>>> {
    let settings: Settings;

    if settings_file_name.exists() {
       settings = read_settings(settings_file_name)?;
    }
    else {
        settings = Settings {
            version: "0.1.0".into(),
        };
    
        let file = File::create(settings_file_name)?;
        serde_json::to_writer(file, &settings)?;
    }

    Ok(Arc::new(RwLock::new(settings)))
}

fn watch<P: AsRef<Path>>(path: P, settings: Arc<RwLock<Settings>>) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    let path_ref = path.as_ref();
    info!("Watching: {:?}", path_ref);
    debouncer
        .watcher()
        .watch(path_ref, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            std::result::Result::Ok(events) => {
                let event = events.first().unwrap();
                info!("Event: {:?}", event);

                match event.kind {
                    DebouncedEventKind::Any => {
                        let mut settings = settings.write().unwrap();
                        *settings = read_settings(path_ref)?;
                        info!("Updated settings: {:?}", settings);
                    },
                    _ => {}
                }
            },
            Err(error) => error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn run(context: Context) -> Result<()> {
  
    let settings = create_settings_if_not_exists(&context.settings_path)?;
    watch(context.settings_path, settings)?;

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
