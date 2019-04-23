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
use postcard::{to_slice, from_bytes};

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
        macros::TimeoutError,
    },

    DWM1001,
    block_timeout,
};

const MSGS_PER_SEC: u32 = 8;
const NOMINAL_INTERVAL_US: u32 = 1_000_000 / MSGS_PER_SEC;
const JITTER_US: u32 = NOMINAL_INTERVAL_US / 10;
const TICKS_PER_S: u32 = 1_000_000 / NOMINAL_INTERVAL_US;

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
    let mut toggle2 = false;
    let mut tx_buf = [0u8; 64];
    let mut rx_buf = [0u8; 1024];

    let mut x = 1;
    let mut y = 1;

    loop {
        let jitter = (NOMINAL_INTERVAL_US - JITTER_US) + (rng.random_u32() % (JITTER_US * 2));
        timer.start(jitter);

        let msg = RadioMessages::SetCell(Cell {
            row: y,
            column: x,
            red: rng.random_u8(),
            green: rng.random_u8(),
            blue: rng.random_u8(),
        });

        x += 1;

        if x > 8 {
            x = 1;
            y += 1;
        }

        if y > 8 {
            y = 1;
        }

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

        if toggle == TICKS_PER_S {
            board.leds.D10.enable();
        } else if toggle >= (2 * TICKS_PER_S) {
            board.leds.D10.disable();
            toggle = 0;
        }


        let mut rx = if let Ok(rx) = dw1000.receive() {
            rx
        } else {
            while timer.wait().is_err() {}
            continue;
        };

        match block_timeout!(&mut timer, rx.wait(&mut rx_buf)) {
            Ok(message) => {
                if let Ok(pmsg) = from_bytes::<RadioMessages>(message.frame.payload) {
                    if let RadioMessages::StartTurn(addr) = pmsg {

                        if toggle2 {
                            board.leds.D11.enable();
                        } else {
                            board.leds.D11.disable();
                        }
                        toggle2 = !toggle2;

                        let mac_addr = mac::Address {
                            pan_id: 0x0386,
                            short_addr: addr,
                        };

                        // I am become turn
                        'addr: loop {
                            if dw1000.set_address(mac_addr).is_err() {
                                continue 'addr;
                            }

                            if let Ok(raddr) = dw1000.get_address() {
                                if mac_addr == raddr {
                                    break 'addr;
                                }
                            }
                        }
                    }
                }
            },
            Err(TimeoutError::Timeout) => { }
            Err(TimeoutError::Other(error)) => { }
        };

    }
}
