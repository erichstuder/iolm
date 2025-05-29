use tokio::time::{sleep, Duration};
use log::info;
use env_logger;

use iol::master_dl;

#[tokio::main]
async fn main() {
    setup_logger();

    let (mut dl, dl_mode_handler) = master_dl::DL::new();
    tokio::spawn(run_dl(dl_mode_handler));

    sleep(Duration::from_secs(2)).await;

    info!("sending signal");
    dl.DL_SetMode(master_dl::Mode::STARTUP).await.unwrap();

    sleep(Duration::from_secs(3)).await;
}

async fn run_dl(mut dl: master_dl::StateMachine<master_dl::StateActionsImpl>) {
    info!("run dl");
    dl.run().await;
}

fn setup_logger() {
    env_logger::Builder::from_default_env()
    .format(|buf, record| {
        use std::io::Write;
        use chrono::Local;
        use anstyle;

        writeln!(
            buf,
            "{} {}{}{}\t{}\n  \x1b[2m{} @ {}:{}\x1b[0m",
            Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
            buf.default_level_style(record.level()),
            record.level(),
            anstyle::Reset,
            record.args(),
            record.module_path().unwrap_or("unknwon module"),
            record.file().unwrap_or("unknown file"),
            record.line().unwrap_or(0),
        )
    })
    .init();
}
