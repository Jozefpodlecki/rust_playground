use tokio::sync::watch;

use crate::models::Settings;

pub struct SettingsManager {
    current: Settings,
    tx: watch::Sender<Settings>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let current= Settings::default();
        let (tx, _) = watch::channel(current.clone());

        Self {
            current,
            tx
        }
    }

    pub fn get(&mut self) -> &Settings {
        &self.current
    }

    pub fn save(&mut self, settings: Settings) {
        self.current = settings.clone();
        let _ = self.tx.send(settings);
    }

    pub fn subscribe(&self) -> watch::Receiver<Settings> {
        self.tx.subscribe()
    }
}