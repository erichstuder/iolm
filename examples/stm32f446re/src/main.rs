//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![no_main]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
// use embassy_stm32::usart::{Config, Uart};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod state_machine;
use state_machine::{StateMachine, StateActions};

use l6360::{self, L6360};

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(peripherals.PA5, Level::High, Speed::Low);

    spawner.spawn(blink(led)).unwrap();
    spawner.spawn(run_statemachine()).unwrap();

    bind_interrupts!(struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    });

    let i2c = I2c::new(
        peripherals.I2C1,
        peripherals.PB8,
        peripherals.PB9,
        Irqs,
        peripherals.DMA1_CH6,
        peripherals.DMA1_CH0,
        Hertz(400_000),
        {
            let mut i2c_config = i2c::Config::default();
            i2c_config.sda_pullup = true;
            i2c_config.scl_pullup = true;
            i2c_config.timeout = embassy_time::Duration::from_millis(1000);
            i2c_config
        },
    );
    let pins = l6360::Pins {
        ENL_plus: Output::new(peripherals.PA6, Level::Low, Speed::Low),
    };
    let mut l6360 = L6360::new(i2c, 0b1100_000, pins).unwrap();
    l6360.set_control_register_1().await.unwrap();
    l6360.set_led_pattern(l6360::Led::LED1, 0xFFF0).await.unwrap();
    l6360.set_led_pattern(l6360::Led::LED2, 0x000F).await.unwrap();
    l6360.enable_ENL_plus().unwrap();
    Timer::after_millis(100_000).await;
}

#[task]
async fn run_statemachine() {
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
