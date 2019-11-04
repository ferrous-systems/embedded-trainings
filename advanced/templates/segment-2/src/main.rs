#![no_std]
#![no_main]
#![allow(unused_imports)]


// Used to define panic behavior
use panic_halt;

// String formatting
use core::fmt::Write;
use heapless::String as HString;

// This is the uarte that we will be creating
mod uarte;

// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        Delay,
        nrf52832_pac::{
            CorePeripherals,
            Peripherals,
        },
    },
    dw1000::{
        mac::{self, PanId, ShortAddress},
    },
    Led,
    Leds,
    new_dw1000,
    DW_RST,
};


#[entry]
fn main() -> ! {
    // Take the core cortex-m peripherals as well as the
    // nrf52832 peripherals
    let core_peripherals = CorePeripherals::take().unwrap();
    let peripherals = Peripherals::take().unwrap();

    // Allocate pins used for specific functionality
    let pins = peripherals.P0.split();

    let mut leds = Leds {
        D9 : Led::new(pins.p0_30.degrade()),
        D10: Led::new(pins.p0_31.degrade()),
        D11: Led::new(pins.p0_22.degrade()),
        D12: Led::new(pins.p0_14.degrade()),
    };

    let mut dw_rst = DW_RST::new(pins.p0_24);

    // Setup timer utilities
    let mut timer  = peripherals.TIMER0.constrain();
    let mut delay  = Delay::new(core_peripherals.SYST);

    // Create and initialize the dwm1001 radio
    dw_rst.reset_dw1000(&mut delay);
    let ununit_dw1000 = new_dw1000(
        peripherals.SPIM2,
        pins.p0_16,
        pins.p0_20,
        pins.p0_18,
        pins.p0_17,
        None,
    );

    let mut dw1000 = ununit_dw1000
        .init()
        .expect("Failed to initialize DW1000");

    // You'll need to set an address. Ask your instructor
    // for more details
    let pan_id = PanId(0x0386);
    let short_addr = ShortAddress(0);

    // This is the UARTE you will be building a driver for
    let mut _uarte = uarte::Uarte::new(peripherals.UARTE0);

    // Wait for the radio to become ready
    loop {
        if dw1000.set_address(pan_id, short_addr).is_err() {
            continue;
        }

        if let Ok(raddr) = dw1000.get_address() {
            if raddr == mac::Address::Short(pan_id, short_addr) {
                break;
            }
        }
    }

    // DW1001 radio initialization complete

    let mut toggle = false;
    loop {
        //.leds.D9  - Top LED BLUE
        //.leds.D12 - Top LED RED
        //.leds.D11 - Bottom LED RED
        //.leds.D10 - Bottom LED BLUE
        if toggle {
           leds.D10.enable();
        } else {
           leds.D10.disable();
        }

        toggle = !toggle;

        timer.delay(250_000);
    }
}
