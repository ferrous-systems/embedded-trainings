#![no_std]
#![no_main]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_halt;

// String formatting
use core::fmt::Write;
use heapless::String as HString;



// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        Delay,
    },
    dw1000::{
        mac,
    },

    DWM1001,
};


#[entry]
fn main() -> ! {
    let mut board  = DWM1001::take().unwrap();
    let mut timer  = board.TIMER0.constrain();
    let mut _rng   = board.RNG.constrain();

    let mut s: HString<heapless::consts::U1024> = HString::new();

    let     clocks = board.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(board.SYST, clocks);

    board.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = board.DW1000.init()
        .expect("Failed to initialize DW1000");

    // You'll need to set an address. Ask your instructor
    // for more details
    let addr = mac::Address {
        pan_id: 0x0386,
        short_addr: 0,
    };

    // Wait for the radio to become ready
    loop {
        if dw1000.set_address(addr).is_err() {
            continue;
        }

        if let Ok(raddr) = dw1000.get_address() {
            if addr == raddr {
                break;
            }
        }
    }

    // First, you'll need to build a message to send to the
    // display. Check out the `protocol` crate for message
    // definitions.

    // Then, you'll need to serialize that message so you
    // can send it as bytes over the radio. You'll also need
    // to select the destination address you'll be sending to.

    // You'll also need to wait until the message has been sent
    // until you can send another one. If you send messages faster
    // than 64 messages/second, the display will reject your requests!

    let mut toggle = false;

    loop {
        s.clear();
        write!(&mut s, "Blink!\r\n").unwrap();
        board.uart.write(s.as_bytes()).unwrap();

        // board.leds.D9  - Top LED BLUE
        // board.leds.D12 - Top LED RED
        // board.leds.D11 - Bottom LED RED
        // board.leds.D10 - Bottom LED BLUE
        if toggle {
            board.leds.D10.enable();
        } else {
            board.leds.D10.disable();
        }

        toggle = !toggle;

        timer.delay(250_000);
    }
}
