use models::Message;
use process_watcher::ProcessWatcher;

mod aws_iprange;
mod process_watcher;
mod models;

#[tokio::main]
async fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<Message>();

    let mut process_watcher = ProcessWatcher::new();
    
    process_watcher.start(tx.clone());

    loop {
        let message = rx.recv().unwrap();
        println!("{:?}", message);
    }
}
