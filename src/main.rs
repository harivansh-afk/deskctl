mod cli;
mod core;
mod daemon;

fn main() -> anyhow::Result<()> {
    if std::env::var("DESKTOP_CTL_DAEMON").is_ok() {
        return daemon::run();
    }
    cli::run()
}
