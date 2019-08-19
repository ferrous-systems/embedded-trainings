#![no_std]
#![no_main]

// Panic provider crate
use panic_halt as _;

// String formatting
use core::fmt::Write;
use heapless::String as HString;

// Used to set the program entry point
use cortex_m_rt::entry;

// Provides definitions for our development board
use dwm1001::{
    nrf52832_hal::{
        prelude::*,
    },
    DWM1001,
};


#[entry]
fn main() -> ! {
    let mut board  = DWM1001::take().unwrap();
    let mut timer  = board.TIMER0.constrain();
    let mut _rng   = board.RNG.constrain();

    let mut s: HString<heapless::consts::U1024> = HString::new();

    let mut toggle = false;

    //Make an array of all the LEDs.
    //Make an array for each animation phase.
    //Make an array of all phases.


    loop {
        s.clear();
        write!(&mut s, "Blink!\r\n").unwrap();
        board.uart.write(s.as_bytes()).unwrap();


        //Iterate through the phases, and apply the phases to the LEDs.


        timer.delay(250_000);
    }
}
