#![no_std]
#![no_main]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_halt;

// String formatting
use core::fmt::Write;
use heapless::String as HString;
use heapless::{consts::*, Vec};

use nb::block;

// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    dw1000::mac,
    embedded_hal::timer::CountDown,
    nrf52832_hal::{prelude::*, rng::Rng, Delay},
    DWM1001,
};
use postcard::to_vec;
use protocol::{Cell, RadioMessages};

// These addresses probably don't need to be changed
// for the class
const SOURCE_PAN_ID: u16 = 0x0386;
const SOURCE_ADDRESS: u16 = 0xABCD;
const DESTINATION_PAN_ID: u16 = 0x0386;

// Heads up! Your instructor will give you a new Source Address to use
// for the class. Make sure you update it!
const DESTINATION_ADDRESS: u16 = 0x0808;

#[entry]
fn main() -> ! {
    let mut board = DWM1001::take().unwrap();
    let mut timer = board.TIMER0.constrain();
    let mut rng = board.RNG.constrain();

    let clocks = board.CLOCK.constrain().freeze();
    let mut delay = Delay::new(board.SYST, clocks);

    board.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = board.DW1000.init().expect("Failed to initialize DW1000");

    let source_address = mac::Address {
        pan_id: SOURCE_PAN_ID,
        short_addr: SOURCE_ADDRESS,
    };
    let destination_address = mac::Address {
        pan_id: DESTINATION_PAN_ID,
        short_addr: DESTINATION_ADDRESS,
    };

    // Before we can send a message, we need to initialize our source
    // address. We should set our address, and confirm that the address
    // reported by the radio matches the one we set.
    loop {
        if dw1000.set_address(source_address).is_err() {
            continue;
        }

        if let Ok(radio_address) = dw1000.get_address() {
            if source_address == radio_address {
                break;
            }
        }
    }

    // First, we need to set the position and color of the pixel we would like to draw
    let red_square = Cell {
        row: 1,
        column: 1,
        red: 200_u8,
        green: 0_u8,
        blue: 0_u8,
    };

    // Then, we need to turn this into a Radio Message from our protocol definition
    let message = RadioMessages::SetCell(red_square);

    // We then need to serialize this message so it can be sent over the radio.
    // This uses serde + postcard to perform the serialization, and places the
    // serialized message into a heapless::Vec.
    let serialized: Vec<u8, U32> = to_vec(&message).unwrap();

    // Finally, we send this message over the radio, and wait for the sending to complete
    let mut future = dw1000.send(&serialized, destination_address, None).unwrap();
    block!(future.wait()).unwrap();

    // if we were in a loop, it would be good to delay for a short while to allow other
    // participants to send messages too!
    delay_with_jitter(&mut timer, &mut rng, 250_000, 50_000);

    // Now that we're all done, lets just blink a light!
    let mut toggle = false;
    let mut s: HString<heapless::consts::U1024> = HString::new();

    loop {
        s.clear();
        write!(&mut s, "Blink!\r\n").unwrap();
        board.uart.write(s.as_bytes()).unwrap();

        // board.leds.D9  - Top LED BLUE
        // board.leds.D12 - Top LED RED
        // board.leds.D11 - Bottom LED RED
        // board.leds.D10 - Bottom LED BLUE
        if toggle {
            board.leds.D11.enable();
        } else {
            board.leds.D11.disable();
        }

        toggle = !toggle;

        delay_with_jitter(&mut timer, &mut rng, 1_000_000, 500_000);
    }
}

/// Delay for `delay_us` microseconds, +/- a random amount of time <= `jitter_us`.
fn delay_with_jitter<T>(timer: &mut T, rng: &mut Rng, delay_us: u32, jitter_us: u32)
where
    T: CountDown,
    <T as CountDown>::Time: core::convert::From<u32>,
{
    assert!(
        delay_us >= jitter_us,
        "Jitter must be less than the total delay!"
    );

    let jittered_us: u32 = (delay_us - jitter_us) + (rng.random_u32() % (jitter_us * 2));
    timer.start(jittered_us);
    while timer.wait().is_err() {}
}
