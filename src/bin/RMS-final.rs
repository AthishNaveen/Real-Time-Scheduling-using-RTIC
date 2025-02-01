#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use stm32f4xx_hal::gpio::{gpioa::PA5, gpioc::PC13, PushPull, Output};
use stm32f4xx_hal::prelude::*;

use rtic_monotonic::systick_monotonic::prelude::*;

systick_monotonic!(Mono, 1000);
    


#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {

    use super::*;
    
    // Constants for task periods (in milliseconds)
    const TASK_A_PERIOD: u32 = 100;  // Highest priority (10Hz)
    const TASK_B_PERIOD: u32 = 200;  // Medium priority (5Hz)
    const TASK_C_PERIOD: u32 = 400;  // Lowest priority (2.5Hz)

    #[shared]
    struct Shared {
        data: u32,
    }

    #[local]
    struct Local {
        led: stm32f4xx_hal::gpio::gpioc::PC13<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let dp = ctx.device;
        let mut rcc = dp.RCC.constrain();
        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        Mono::start(cx.core.SYST, 36_000_000); // default STM32F303 clock-rate is 36MHz

        
        // Start all periodic tasks
        task_a::spawn().ok();
        task_b::spawn().ok();
        
        (Shared { data: 0 }, Local { led })
    }

    // Task A: Highest priority due to shortest period (100ms)
    #[task(priority = 3, shared = [data])]
    async fn task_a(ctx: task_a::Context) {
        // Simulate some work
        cortex_m::asm::delay(200);
        rprintln("Task A execution");
        // Schedule next execution
        task_a::spawn_after(TASK_A_PERIOD.millis()).ok();
    }

    // Task B: Medium priority with 200ms period
    #[task(priority = 2, shared = [data])]
    async fn task_b(ctx: task_b::Context) {
        // Simulate some work
        cortex_m::asm::delay(100);
        rprintln("Task B execution");   

        // Schedule next execution
        task_b::spawn_after(TASK_B_PERIOD.millis()).ok();
    }

    // Task C: Lowest priority with 400ms period
    #[task(priority = 1, local = [led], shared = [data])]
    async fn task_c(ctx: task_c::Context) {
        // Simulate some work
        cortex_m::asm::delay(4000);
        
        // Toggle LED to show execution
        ctx.local.led.toggle();
        
        // Schedule next execution
        task_c::spawn_after(TASK_C_PERIOD.millis()).ok();
    }
}
