pub mod backend;
pub mod cli;
pub mod core;
pub mod daemon;

pub fn run() -> anyhow::Result<()> {
    if std::env::var("DESKCTL_DAEMON").is_ok() {
        return daemon::run();
    }
    cli::run()
}
