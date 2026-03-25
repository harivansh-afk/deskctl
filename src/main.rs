mod backend;
mod cli;
mod core;
mod daemon;
#[cfg(test)]
mod test_support;

fn main() -> anyhow::Result<()> {
    if std::env::var("DESKCTL_DAEMON").is_ok() {
        return daemon::run();
    }
    cli::run()
}
