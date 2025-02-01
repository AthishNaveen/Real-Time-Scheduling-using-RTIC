// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa::PA5, PushPull, Output};
use stm32f4xx_hal::prelude::*;


systick_monotonic!(Mono, 1000);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        state: bool,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let _flash = cx.device.FLASH;
        let rcc = cx.device.RCC.constrain();

        // Initialize the systick interrupt & obtain the token to prove that we did
        Mono::start(cx.core.SYST, 36_000_000); // default STM32F303 clock-rate is 36MHz

        rtt_init_print!();
        rprintln!("init");

        let _clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(36.mhz())
            .pclk1(36.mhz())
            .freeze();

        // Setup LED
        let gpioa = cx.device.GPIOA.split();



        let mut led = gpioa.pa5.into_push_pull_output();

        led.set_high();


        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led, state: false })
    }

    #[task(local = [led, state])]
    async fn blink(cx: blink::Context) {
        loop {
            rprintln!("blink");
            if *cx.local.state {
                cx.local.led.set_high();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low();
                *cx.local.state = true;
            }
            Mono::delay(1000.millis()).await;
        }
    }
}
