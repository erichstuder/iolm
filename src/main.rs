//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![no_main]
#![no_std]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(peripherals.PA5, Level::High, Speed::Low);

    spawner.spawn(blink(led)).unwrap();
}

#[task]
async fn blink(mut led: Output<'static>) -> ! {
    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(2000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(2000).await;
    }
}
