#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use stm32f4xx_hal::{pac::TIM2, prelude::*, timer::{Event, CounterUs}};
use rtt_target::{rprintln, rtt_init_print};


#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use stm32f4xx_hal::timer::CounterUs;

    use super::*;

    // Structure to represent a virtual timer
    pub struct VirtualTimer {
        deadline: u32,
        period: u32,
        active: bool,
        callback_id: u8,
    }

    #[shared]
    struct Shared {
        virtual_timers: [VirtualTimer; 4],  // Support for 4 virtual timers
        system_ticks: u32,
    }

    #[local]
    struct Local {
        hardware_timer: CounterUs<TIM2>
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        let dp = ctx.device;
        
        // Configure system clock
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

    // Configure hardware timer for 1ms ticks
        let mut timer = dp.TIM2.counter_us(&clocks);
        timer.start(1_000.micros()).unwrap(); // Start the timer with a 1ms period
        timer.listen(Event::Update);
        rprintln!("Timer Started");

        // Initialize virtual timers
        let virtual_timers = [
            VirtualTimer { deadline: 0, period: 0, active: false, callback_id: 0 },
            VirtualTimer { deadline: 0, period: 0, active: false, callback_id: 1 },
            VirtualTimer { deadline: 0, period: 0, active: false, callback_id: 2 },
            VirtualTimer { deadline: 0, period: 0, active: false, callback_id: 3 },
        ];

        // Start a demo timer
        start_virtual_timer::spawn(0, 5000).ok();  // Timer 0, period 1000ms

        (
            Shared {
                virtual_timers,
                system_ticks: 0,
            },
            Local { hardware_timer: timer },
        )
    }

    // Task to start a virtual timer
    #[task(priority = 2, shared = [virtual_timers, system_ticks])]
    async fn start_virtual_timer(mut ctx: start_virtual_timer::Context, timer_id: usize, period: u32) {
        ctx.shared.virtual_timers.lock(|timer| {
            timer[timer_id].period = period;
            timer[timer_id].deadline = ctx.shared.system_ticks.lock(|ticks| *ticks) + period;
            timer[timer_id].active = true;
        });
        rprintln!("Virtual Timer {} started with period {}", timer_id, period);
    }

    // Hardware timer interrupt handler
    #[task(binds = TIM2, priority = 3, shared = [virtual_timers, system_ticks], local = [hardware_timer])]
    fn timer_tick(mut ctx: timer_tick::Context) {
        // Clear hardware timer interrupt
        ctx.local.hardware_timer.clear_all_flags();
        // Update system tick counter
        ctx.shared.system_ticks.lock(|ticks| *ticks += 1);

        // Check virtual timers
        ctx.shared.virtual_timers.lock(|timers| {
            let current_ticks = ctx.shared.system_ticks.lock(|ticks| *ticks);
            
            for timer in timers.iter_mut() {
                if timer.active && current_ticks >= timer.deadline {
                    // Timer expired - trigger callback
                    timer_callback::spawn(timer.callback_id).ok();
                    
                    // Reset deadline for periodic timers
                    timer.deadline = current_ticks + timer.period;
                }
            }
        });
    }

    // Callback task for timer expiration
    #[task(priority = 1)]
    async fn timer_callback(ctx: timer_callback::Context, timer_id: u8) {
        // Handle timer expiration here
        // For example, toggle an LED or perform some periodic task
        rprintln!("Virtual Timer {} expired", timer_id);
    }
}