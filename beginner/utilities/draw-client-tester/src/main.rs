#![no_std]
#![no_main]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_halt;

// String formatting
use nb::block;

use protocol::{RadioMessages, Cell};
use postcard::{to_slice, from_bytes};
use embedded_timeout_macros::TimeoutError;

// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        Delay,
    },
    dw1000::{
        hl::RxConfig,
        mac::{self, PanId, ShortAddress},
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

    let mut delay  = Delay::new(board.SYST);

    board.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = board.DW1000.init()
        .expect("Failed to initialize DW1000");

    let pan_id = PanId(0x0386);
    let short_addr = ShortAddress(rng.random_u16() % 16 + 1);

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

    let mut toggle = 0u32;
    let mut toggle2 = false;
    let mut tx_buf = [0u8; 64];
    let mut rx_buf = [0u8; 1024];

    let mut x = 1;
    let mut y = 1;

    let mut dw1000_opt = Some(dw1000);

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

        let addr = mac::Address::Short(
            PanId(0x0386),
            ShortAddress(0x0808),
        );
        let mut tx = dw1000_opt.take()
            .expect("dw1000 not there")
            .send(
                msg_buf,
                addr,
                None
            )
            .expect("Failed to start receiver");



        block!(tx.wait())
            .expect("Failed to send data");

        let dw1000 = tx.finish_sending().expect("failed to finish sending");

        toggle += 1;

        if toggle == TICKS_PER_S {
            board.leds.D10.enable();
        } else if toggle >= (2 * TICKS_PER_S) {
            board.leds.D10.disable();
            toggle = 0;
        }


        let mut rx = if let Ok(rx) = dw1000.receive(RxConfig::default()) {
            rx
        } else {
            while timer.wait().is_err() {}
            continue;
        };

        let result = block_timeout!(&mut timer, rx.wait(&mut rx_buf));
        let mut dw1000 = rx.finish_receiving().expect("finish_receiving failed");
        match result {
            Ok(message) => {
                if let Ok(pmsg) = from_bytes::<RadioMessages>(message.frame.payload) {
                    if let RadioMessages::StartTurn(addr) = pmsg {

                        if toggle2 {
                            board.leds.D11.enable();
                        } else {
                            board.leds.D11.disable();
                        }
                        toggle2 = !toggle2;

                        let pan_id = PanId(0x0386);
                        let short_addr = ShortAddress(addr);

                        // I am become turn
                        'addr: loop {
                            if dw1000.set_address(pan_id, short_addr).is_err() {
                                continue 'addr;
                            }

                            if let Ok(raddr) = dw1000.get_address() {
                                if raddr == mac::Address::Short(pan_id, short_addr) {
                                    break 'addr;
                                }
                            }
                        }
                    }
                }
            },
            Err(TimeoutError::Timeout) => { }
            Err(TimeoutError::Other(_)) => { }
        };

        dw1000_opt = Some(dw1000);
    }
}
