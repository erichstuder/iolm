use tokio::time::{sleep, Duration, timeout};
use std::future::Future;
use log::info;
use env_logger;

use iol::master_dl;
use iol::master;

#[tokio::main]
async fn main() {
    setup_logger();

    use_master().await;
    //use_dl().await;

    sleep(Duration::from_secs(100)).await;
}



#[derive(Copy, Clone)]
struct MasterActionsImpl;

impl master::Actions for MasterActionsImpl {
    async fn port_power_on(&self) {
        info!("port power on");
    }

    async fn port_power_off(&self) {
        info!("port power off");
    }

    async fn await_with_timeout_ms<F, T>(&self, future: F, duration: u64) -> Option<T>
    where
        F: Future<Output = T>,
    {
        info!("await with timeout");
        let result = timeout(Duration::from_millis(duration), future).await.ok();
        info!("timeout");
        result
    }
}

async fn use_master() {
    let (master, port_power_switching) = master::Master::new(MasterActionsImpl);
    tokio::spawn(run_port_power_switching(port_power_switching));

    sleep(Duration::from_secs(2)).await;

    master.port_power_off_on(3000).await;
}

async fn run_port_power_switching(mut port_power_switching: master::PortPowerSwitchingStateMachine<MasterActionsImpl>) {
    info!("run port power switching");
    port_power_switching.run().await;
}



#[derive(Copy, Clone)]
struct MasterDlActionsImpl;

impl iol::master_dl::Actions for MasterDlActionsImpl {
    async fn wait_ms(&self, duration: u64) {
        sleep(Duration::from_millis(duration)).await;
    }
}

#[allow(unused)]
async fn use_dl() {
    let (mut dl, dl_mode_handler) = master_dl::DL::new(MasterDlActionsImpl);
    tokio::spawn(run_dl(dl_mode_handler));

    sleep(Duration::from_secs(2)).await;

    info!("sending signal");
    dl.DL_SetMode(master_dl::Mode::STARTUP).await.unwrap();
}

async fn run_dl(mut dl: master_dl::DlModeHandlerStateMachine<MasterDlActionsImpl>) {
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
