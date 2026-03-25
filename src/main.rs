mod backend;
mod cli;
mod core;
mod daemon;

fn main() -> anyhow::Result<()> {
    if std::env::var("DESKCTL_DAEMON").is_ok() {
        return daemon::run();
    }
    cli::run()
}
