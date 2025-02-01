#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use heapless::spsc::Queue;  // Single-producer, single-consumer queue

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;

    // Message types for our communication
    #[derive(Clone, Copy)]
    enum Message {
        SensorData(u16),
        Command(CommandType),
    }

    #[derive(Clone, Copy)]
    enum CommandType {
        Start,
        Stop,
        Reset,
    }
    
    #[shared]
    struct Shared {
        // Message queue for inter-thread communication
        message_queue: Queue<Message, 16>,  // Queue capacity of 16 messages
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Create a new message queue
        let message_queue = Queue::new();

        // Start our producer and consumer tasks
        producer_task::spawn().ok();
        consumer_task::spawn().ok();

        (
            Shared { message_queue },
            Local {},
            // init::Monotonics()
        )
    }

    // Producer task - sends messages
    #[task(priority = 2, shared = [message_queue])]
    async fn producer_task(ctx: producer_task::Context) {
        // Simulate reading sensor data
        let sensor_value = 42;  // Example value

        // Send the sensor data
        ctx.shared.message_queue.lock(|queue| {
            if queue.enqueue(Message::SensorData(sensor_value)).is_ok() {
                // Message sent successfully
                rprintln("Message sent");
            }
        });

        // Send a command
        // ctx.shared.message_queue.lock(|queue| {
        //     if queue.enqueue(Message::Command(CommandType::Start)).is_ok() {
        //         // Command sent successfully
        //         rprintln("Command sent");

        //     }
        // });

        // Schedule next execution
        producer_task::spawn_after(100.millis()).ok();
    }

    // Consumer task - receives and processes messages
    #[task(priority = 1, shared = [message_queue])]
    async fn consumer_task(ctx: consumer_task::Context) {
        // Process all available messages
        ctx.shared.message_queue.lock(|queue| {
            while let Some(message) = queue.dequeue() {
                rprintln!("Message Recieved: {}", value);
                match message {
                    Message::SensorData(value) => {
                        // Handle sensor data
                        cmd = process_sensor_data(value);
                    },
                    Message::Command(cmd) => {
                        // Handle command
                        match cmd {
                            CommandType::Start => { /* Start operation */ },
                            CommandType::Stop => { /* Stop operation */ },
                            CommandType::Reset => { /* Reset system */ },
                        }
                    }
                }
            }
        });

        // Schedule next execution
        consumer_task::spawn_after(50.millis()).ok();
    }

    fn process_sensor_data(value: u16) -> CommandType{
        rprintln!{"Deciding operation : Start"};
        CommandType::Start
    }
}