use clap::{arg, command, Parser, ValueEnum};


/// Forking app with --type and --port
#[derive(Parser, Debug)]
#[command(name = "ForkApp")]
#[command(about = "App that spawns a child process with args")]
pub struct CommandArgs {
    #[arg(long, value_enum, default_value_t = ProcessType::Server)]
    pub r#type: ProcessType,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub ip_address: String,

    #[arg(long, default_value_t = 6042)]
    pub port: u16,

    #[arg(long, default_value_t = String::from("Collector"))]
    pub pipe_name: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[clap(rename_all = "lower")]
pub enum ProcessType {
    Server,
    Child,
}
