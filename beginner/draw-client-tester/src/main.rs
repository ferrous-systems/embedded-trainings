#![no_std]
#![no_main]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_halt;

// String formatting
use core::fmt::Write;
use heapless::String as HString;

use nb::block;

use protocol::{RadioMessages, Cell};
use postcard::to_slice;

// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        Delay,
        Uarte,
        nrf52832_pac::{
            UARTE0,
        },
    },
    dw1000::{
        hl::{
            Message,
        },
        mac,
    },

    DWM1001,
    block_timeout,
};


#[entry]
fn main() -> ! {
    let mut board = DWM1001::take().unwrap();
    let mut timer = board.TIMER0.constrain();
    let mut rng   = board.RNG.constrain();

    let mut s: HString<heapless::consts::U1024> = HString::new();

    let     clocks = board.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(board.SYST, clocks);

    board.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = board.DW1000.init()
        .expect("Failed to initialize DW1000");

    let addr = mac::Address {
        pan_id: 0x0386,
        short_addr: rng.random_u16() % 16 + 1,
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

    let mut toggle = 0u32;
    let mut tx_buf = [0u8; 64];

    loop {
        let msg = RadioMessages::SetCell(Cell {
            row: ((rng.random_u8() % 8) + 1) as usize,
            column: ((rng.random_u8() % 8) + 1) as usize,
            red: rng.random_u8(),
            green: rng.random_u8(),
            blue: rng.random_u8(),
        });

        let msg_buf = to_slice(&msg, &mut tx_buf).unwrap();

        let mut tx = dw1000
            .send(
                msg_buf,
                mac::Address {
                    pan_id:     0x0386, // 0x0386,
                    short_addr: 0x0808, // 0x0001,
                },
                None
            )
            .expect("Failed to start receiver");



        block!(tx.wait())
            .expect("Failed to send data");

        toggle += 1;

        if toggle == 100 {
            board.leds.D10.enable();
        } else if toggle >= 200 {
            board.leds.D10.disable();
            toggle = 0;
        }


        timer.delay(1_000);
    }
}
