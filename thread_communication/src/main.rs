use actors::{boss::BossThread, dps::DpsThread, skills::get_slayer_skills, support::SupportThread};
use models::{Message, PlayerStats};
use simple_logger::SimpleLogger;

mod models;
mod actors;

fn main() {
    SimpleLogger::new().env().init().unwrap();
    
    let player_stats = PlayerStats {
        skills: get_slayer_skills(),
        cooldown_reduction: 0.4,
        attack_power: 1e5 as i64,
        crit_rate: 0.4
    };

    let (send, recv) = multiqueue::broadcast_queue(4);
    // let (tx, rx) =  std::sync::mpmc::sync_channel::<Message>();
    // let (tx, rx) = std::sync::mpsc::channel::<Message>();
    
    let mut sup_thread = SupportThread::new();
    let mut boss_thread = BossThread::new();
    let mut dps_thread = DpsThread::new(player_stats);

    dps_thread.start(send.clone(), recv.clone());

    dps_thread.wait();
}
