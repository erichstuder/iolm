//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![no_main]
#![no_std]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Level, Output, Speed};
// use embassy_stm32::usart::{Config, Uart};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod state_machine;
use state_machine::{StateMachine, StateActions};

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(peripherals.PA5, Level::High, Speed::Low);

    spawner.spawn(blink(led)).unwrap();

    struct StateActionsImpl;
    impl StateActions for StateActionsImpl {
        async fn wait_ms(&self, duration: u64) {
            Timer::after_millis(duration).await;
        }
    }

    let mut state_machine = StateMachine::new(&StateActionsImpl);
    state_machine.run().await;
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
