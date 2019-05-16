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

    let mut ledarray = &mut [&mut board.leds.D9, &mut board.leds.D12, &mut board.leds.D11, &mut board.leds.D10];

    let phase_0 = [0, 0, 0, 0];
    let phase_1 = [1, 0, 0, 0];
    let phase_2 = [1, 1, 0, 0];
    let phase_3 = [1, 1, 1, 0];
    let phase_4 = [1, 1, 1, 1];
    let phase_5 = [0, 1, 1, 1];
    let phase_6 = [0, 0, 1, 1];
    let phase_7 = [0, 0, 0, 1];
    let phase_8 = [0, 0, 0, 0];

    let phases = &[phase_0, phase_1, phase_2, phase_3, phase_4, phase_5, phase_6, phase_7, phase_8, phase_7, phase_6, phase_5, phase_4, phase_3, phase_2, phase_1];


    loop {
        s.clear();
        write!(&mut s, "Blink!\r\n").unwrap();
        board.uart.write(s.as_bytes()).unwrap();

        // board.leds.D9  - Top LED BLUE
        // board.leds.D12 - Top LED RED
        // board.leds.D11 - Bottom LED RED
        // board.leds.D10 - Bottom LED BLUE

        for phase in phases {

            for (i, led) in ledarray.iter_mut().enumerate() {

                if phase[i] == 1 {
                    led.enable();
                } else {
                    led.disable();
                }
            }

        timer.delay(100_000);


        }

        timer.delay(300_000);





        //
        // if toggle {
        //     board.leds.D10.enable();
        //     board.leds.D11.disable();
        //     board.leds.D12.disable();
        //     board.leds.D9.disable();
        //
        // } else {
        //     board.leds.D10.disable();
        //     board.leds.D11.enable();
        //     board.leds.D12.disable();
        //     board.leds.D9.disable();
        // }
        //
        // toggle = !toggle;

        timer.delay(250_000);
    }
}
