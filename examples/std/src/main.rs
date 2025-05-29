use tokio::time::{sleep, Duration};
use log::info;
use env_logger;

use iol::master_dl;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Hello!!!!");

    let (mut dl, dl_mode_handler) = master_dl::DL::new();
    tokio::spawn(run_dl(dl_mode_handler));

    sleep(Duration::from_secs(2)).await;

    println!("sending signal");
    dl.DL_SetMode(master_dl::Mode::STARTUP).await.unwrap();

    sleep(Duration::from_secs(100)).await;
}

async fn run_dl(mut dl: master_dl::StateMachine<master_dl::StateActionsImpl>) {
    println!("run dl");
    dl.run().await;
}
