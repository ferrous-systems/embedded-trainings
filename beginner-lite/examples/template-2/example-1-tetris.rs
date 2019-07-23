#![no_std]
#![no_main]

// Used to define panic behavior
#[allow(unused_imports)]
use panic_halt;

// String formatting
use core::fmt::Write;
use heapless::String as HString;
use heapless::{Vec, consts::*};

use nb::block;

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
use postcard::{to_vec};
use protocol::{RadioMessages, RGB, ApiGrid};

fn make_grid() -> [[RGB; 8]; 8] {

    let color = RGB {
        red: 0,
        green: 0,
        blue: 0,
    };

    let grid: [[RGB; 8]; 8] = [[color; 8]; 8];
    grid
}

fn make_tetris(position: &[[i32; 2]; 4]) -> ApiGrid {

    let mut grid =  make_grid();
    for cell in position {

        let color_arr = RGB {
            red: 100,
            green: 0,
            blue: 100,
        };

        let row =  cell[0] as usize;
        let column = cell[1] as usize;
        grid[row][column] = color_arr;
    }

    let sharedgrid = ApiGrid {
        zero_row: 5,
        zero_column: 5,
        api_grid: grid,
    };

    sharedgrid
}

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
        short_addr: 3,
    };
    let recipient = mac::Address {
        pan_id: 0x0386,
        short_addr: 0x0808,
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

    let position_1 = [[3, 4], [4, 4], [5, 4], [5, 3]];
    let position_2 = [[4, 5], [4, 4], [4, 3], [3, 3]];
    let position_3 = [[5, 3], [4, 3], [3, 3], [3, 4]];
    let position_4 = [[3, 2], [3, 3], [3, 4], [4, 4]];

    let positions = &[position_1, position_2, position_3, position_4];

    loop {
        for position in positions {
            let sharedgrid = make_tetris(position);

            //This needs to change
            let message = RadioMessages::SetGrid(sharedgrid);
            let output: Vec<u8, U32> = to_vec(&message).unwrap();
            let mut future = dw1000.send(&output, recipient, None).unwrap();

            block!(future.wait()).unwrap();
            timer.delay(250_000);
        }
    }
}
