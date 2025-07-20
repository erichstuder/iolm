#![deny(unsafe_code)]

use tokio::time::{sleep, Duration, timeout};
use std::future::Future;
use log::info;

use iol::master::{self, smi};

#[tokio::main]
async fn main() {
    setup_logger();

    info!("start master");
    use_master().await;

    sleep(Duration::from_secs(100)).await;
}

#[derive(Copy, Clone)]
struct MasterActions;

impl master::Actions for MasterActions {
    async fn wait(&self, duration: Duration) {
        sleep(duration).await;
    }

    async fn get_cq(&self) -> l6360::PinState {
        l6360::PinState::Low
    }

    async fn wake_up_pulse(&self, _direction: master::WakeUpPulseDirection) {
        info!("ready pulse done");
    }

    async fn port_power_on(&self) {
        info!("port power on");
    }

    async fn port_power_off(&self) {
        info!("port power off");
    }

    async fn await_event_with_timeout<F, T>(&self, duration: Duration, future: F) -> Option<T>
    where
        F: Future<Output = T>,
    {
        info!("await with timeout");
        let result = timeout(duration, future).await.ok();
        info!("timeout");
        result
    }

    async fn await_ready_pulse_with_timeout(&self, _duration: Duration) -> master::ReadyPulseResult {
        master::ReadyPulseResult::ReadyPulseOk
    }

    async  fn set_baudrate(&self, baudrate: u32) {
        info!("baudrate set to {:?}", baudrate);
    }

    async fn exchange_data(&self, _data: &[u8], _answer: &mut [u8]) {}
}

async fn use_master() {
    let mut master = master::Master::new(MasterActions);
    tokio::spawn(async move { master.run().await; });

    sleep(Duration::from_secs(2)).await;

    info!("smi port configuration");
    let client_id = 0;
    let port_number = 0;
    let _ = smi::SMI_PortConfiguration(
        client_id,
        port_number,
        smi::annex_e::PortConfigList {
            port_mode: 0,
            validation_and_backup: 0,
            iq_behavior: 0,
            port_cycle_time: 0,
            vendor_id: 0,
            device_id: 0,
        }
    ).await;

    sleep(Duration::from_secs(20)).await;
}

fn setup_logger() {
    use env_logger;

    env_logger::Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .format(|buf, record| {
            use std::io::Write;
            // use chrono::Local;
            use std::time::Instant;
            use std::sync::OnceLock;
            use anstyle;

            static START_TIME: OnceLock<Instant> = OnceLock::new();
            let start_time = START_TIME.get_or_init(|| Instant::now());

            writeln!(
                buf,
                "{:.6} {}{}{}\t{}\n  \x1b[90m{} @ {}:{}\x1b[0m",
                // Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                start_time.elapsed().as_secs_f64(),
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
