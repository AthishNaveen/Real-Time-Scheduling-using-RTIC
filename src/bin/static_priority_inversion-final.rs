#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use stm32f4xx_hal::{prelude::*, gpio::{gpioe::PE1, Output, PushPull}};

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    // Resource that could cause priority inversion
    struct SharedResource {
        data: u32,
        is_locked: bool,
        owner: Option<u8>,
    }

    #[shared]
    struct Shared {
        resource: SharedResource,
        led: PE1<Output<PushPull>>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp = ctx.device;
        
        // Configure LED
        let gpioe = dp.GPIOE.split();
        let led = gpioe.pe1.into_push_pull_output();

        // Initialize shared resource
        let resource = SharedResource {
            data: 0,
            is_locked: false,
            owner: None,
        };

        // Start our demonstration tasks
        low_priority::spawn().ok();
        high_priority::spawn().ok();

        (
            Shared { 
                resource,
                led,
            },
            Local {},
            init::Monotonics()
        )
    }

    // Low priority task that holds the shared resource
    #[task(priority = 1, shared = [resource])]
    fn low_priority(ctx: low_priority::Context) {
        // Acquire the resource without priority inheritance
        ctx.shared.resource.lock(|res| {
            res.is_locked = true;
            res.owner = Some(1);
            
            // Simulate long processing time
            for _ in 0..1000 {
                res.data += 1;
            }
            
            res.is_locked = false;
            res.owner = None;
            rprintln("(Low Priority) completed resource usage");
        });
        // Reschedule ourselves
        low_priority::spawn_after(500.millis()).ok();
    }

    // High priority task that needs the shared resource
    #[task(priority = 3, shared = [resource, led])]
    fn high_priority(ctx: high_priority::Context) {
        // Try to access the resource
        ctx.shared.resource.lock(|res| {
            if res.is_locked {
                // Resource is held by low priority task
                // We are blocked! Priority inversion occurring
                rprintln!("(High Priority) waiting for resources");
                ctx.shared.led.lock(|led| led.set_high());
            } else {
                res.data += 1;
                ctx.shared.led.lock(|led| led.set_low());
                rprintln("High Priority resume execution");
            }
        });

        rprintln!("Reschedule!");
        // Reschedule ourselves
        high_priority::spawn_after(100.millis()).ok();
    }
}