#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use rtt_target::{rtt_init_print, rprintln};
use stm32f4xx_hal::{gpio::gpioc::PC13, gpio::{Output, PushPull}, prelude::*};

use cortex_m::peripheral::Peripherals;

#[app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI3])]
mod app {
    use super::*;

    
    #[shared]
    struct Shared {
        // Shared resources (if any)
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        let dp = ctx.device;

        // Configure PC13 (onboard LED) as output
        let mut rcc = dp.RCC.constrain();
        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        // Spawn tasks
        task_c::spawn().unwrap();   
        task_b::spawn().unwrap();   
        task_a::spawn().unwrap();

        (Shared {}, Local { led })
    }

    // High-priority task
    #[task(priority = 2 )]
    async fn task_a(_: task_a::Context) {
        high_priority_output.toggle();
        rprintln!("High Priority button(3) pressed");
        rprintln!("High Priority Task executed");

        cortex_m::asm::delay(50);


    }

    // Medium-priority task
    #[task(priority = 2, local = [led])]
    async fn task_b(cx: task_b::Context) {
        medium_priority_output.toggle();
        rprintln!("Medium Priority button(2) pressed");
        rprintln!("Medium Priority Task executed");

        cortex_m::asm::delay(100);
    }
    
    // Low-priority task
    #[task(priority = 1, local = [led])]
    async fn task_c(cx: task_b::Context) {
        low_priority_output.toggle();
        rprintln!("Low Priority button(1) pressed");
        rprintln!("Low Priority Task executed");

        cortex_m::asm::delay(100);
    }
}
