#![no_std]
#![no_main]

// added for interrupt support
use core::cell::RefCell;
use critical_section::Mutex;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{self, Event, Input, Io, Level, Output, Pull},
    macros::ram,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

static BUTTON: Mutex<RefCell<Option<Input<gpio::Gpio0>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    // Set GPIO2 as an output, and set its state high initially.
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);
    let mut led = Output::new(io.pins.gpio2, Level::Low);
    let mut button = Input::new(io.pins.gpio0, Pull::Up);

    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });
    led.set_high();

    esp_println::logger::init_logger_from_env();

    loop {
        log::info!("Running loop!");
        led.toggle();
        delay.delay(500.millis());
    }
}

#[handler]
#[ram]
fn handler() {
    esp_println::println!(
        "
    GPIO Interrupt with priority {}
    ",
        esp_hal::xtensa_lx::interrupt::get_level()
    );
    esp_println::println!("GPIO Interrupt");

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
