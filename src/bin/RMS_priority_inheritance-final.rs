#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use stm32f4xx_hal::{
    gpio::{gpioe::PE1, Output, PushPull},
    pac::TIM2,
    prelude::*,
    timer::{Event, Timer},
};

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    // Shared resources that could cause priority inversion
    #[shared]
    struct Shared {
        shared_resource: u32,
        led: PE1<Output<PushPull>>,
    }

    #[local]
    struct Local {
        timer: Timer<TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Get access to device specific peripherals
        let dp = ctx.device;
        
        // Configure system clock
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr
            .use_hse(8.MHz())
            .sysclk(180.MHz())
            .hclk(180.MHz())
            .pclk1(45.MHz())
            .pclk2(90.MHz())
            .freeze();

        // Configure LED on PE1
        let gpioe = dp.GPIOE.split();
        let led = gpioe.pe1.into_push_pull_output();

        // Configure timer for task scheduling
        let mut timer = dp.TIM2.timer_ms(&clocks);
        timer.listen(Event::TimeOut);

        // Start our periodic tasks with RMS
        high_freq_task::spawn().ok();
        med_freq_task::spawn().ok();
        low_freq_task::spawn().ok();

        (
            Shared {
                shared_resource: 0,
                led,
            },
            Local { timer },
            init::Monotonics()
        )
    }

    // High frequency task (100ms period = highest priority)
    #[task(priority = 3, shared = [shared_resource, led])]
    fn high_freq_task(ctx: high_freq_task::Context) {
        // Using RTIC's lock mechanism for priority inheritance
        (ctx.shared.shared_resource, ctx.shared.led).lock(|resource, led| {
            // Critical section that needs the shared resource
            *resource = *resource + 1;
        });
        high_priority_output.toggle();
        rprintln!("Button 3 pressed");
        // Schedule next execution
        high_freq_task::spawn_after(50.millis()).ok();
    }

    // Low frequency task (400ms period = lowest priority)
    #[task(priority = 2, shared = [shared_resource])]
    fn low_freq_task(ctx: low_freq_task::Context) {
        // This task might hold the shared resource for a long time
        ctx.shared.shared_resource.lock(|resource| {
            // Simulate some time-consuming work
            for _ in 0..1000 {
                *resource = *resource + 1;
            }
        });
        high_priority_output.toggle();
        rprintln!("Button 3 pressed");
        // Schedule next execution
        low_freq_task::spawn_after(100.millis()).ok();
    }

    // Low frequency task (400ms period = lowest priority)
    #[task(priority = 1, shared = [shared_resource])]
    fn low_freq_task(ctx: low_freq_task::Context) {
        // This task might hold the shared resource for a long time
        ctx.shared.shared_resource.lock(|resource| {
            // Simulate some time-consuming work
            for _ in 0..100 {
                *resource = *resource + 1;
            }
        });
        high_priority_output.toggle();
        rprintln!("Button 3 pressed");

        // Schedule next execution
        low_freq_task::spawn_after(200.millis()).ok();
    }

    // Hardware interrupt handler for timer
    #[task(binds = TIM2, priority = 4, local = [timer])]
    fn timer_interrupt(ctx: timer_interrupt::Context) {
        ctx.local.timer.clear_interrupt(Event::TimeOut);
    }
}