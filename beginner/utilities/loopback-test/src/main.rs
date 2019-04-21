#![no_main]
#![no_std]

#![allow(dead_code)]

// Built in dependencies
use core::fmt::Write;

// Crates.io dependencies
use dw1000::{DW1000 as DW};
use dwm1001::{
    self,
    nrf52832_hal::{
        delay::Delay,
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level, p0::P0_17},
        rng::Rng,
        spim::{Spim},
        nrf52832_pac::{
            TIMER0,
            SPIM2,
        },
        uarte::Baudrate as UartBaudrate,
    },
    new_dw1000,
    new_usb_uarte,
    UsbUarteConfig,
    DW_RST,
    block_timeout,
    dw1000::{
        macros::TimeoutError,
        mac::Address,
        Message,
    },
};
use heapless::{String, consts::*};
use rtfm::app;
use postcard::{from_bytes, to_slice};

// NOTE: Panic Provider
use panic_ramdump as _;

// Workspace dependencies
use protocol::{
    ModemUartMessages,
    CellCommand,
    RadioMessages,
};
use nrf52_bin_logger::{
    Logger,
    senders::RealSender,
    receivers::RealReceiver,
};

const RX_PERIOD_US: u32 = 9_000;
const IDLE_WARNING_US: u32 = 1_000_000;
const IDLE_STEPDOWN: u32 = IDLE_WARNING_US / RX_PERIOD_US;
type ModemLogger = Logger<
    // Send logs + ModemUartMessages, max outgoing serialized
    // message size is 128 bytes
    RealSender<ModemUartMessages, U1024>,

    // Receive ModemUartMessages, max incoming serialized message
    // size is 256 bytes, store up to 8 parsed messages
    RealReceiver<ModemUartMessages, U2048, U64>,
>;


#[app(device = dwm1001::nrf52832_hal::nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    ModemLogger = ();
    static mut DW1000:    DW<
                            Spim<SPIM2>,
                            P0_17<Output<PushPull>>,
                            dw1000::Ready,
                          > = ();
    static mut DW_RST_PIN: DW_RST                   = ();
    static mut RANDOM:     Rng                      = ();

    #[init]
    fn init() {
        let timer = device.TIMER0.constrain();
        let pins = device.P0.split();

        let mut uc = UsbUarteConfig::default();
        uc.baudrate = UartBaudrate::BAUD115200;

        let uarte0 = new_usb_uarte(
            device.UARTE0,
            pins.p0_05,
            pins.p0_11,
            uc,
        );

        let rng = device.RNG.constrain();

        let dw1000 = new_dw1000(
            device.SPIM2,
            pins.p0_16,
            pins.p0_20,
            pins.p0_18,
            pins.p0_17,
        );

        let mut rst_pin = DW_RST::new(pins.p0_24.into_floating_input());

        let clocks = device.CLOCK.constrain().freeze();

        let mut delay = Delay::new(core.SYST, clocks);

        rst_pin.reset_dw1000(&mut delay);
        let mut dw1000 = dw1000.init().unwrap();

        let addr = Address {
            pan_id:     MODEM_PAN,
            short_addr: MODEM_ADDR,
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

        RANDOM = rng;
        DW_RST_PIN = rst_pin;
        DW1000 = dw1000;
        LOGGER = Logger::new(uarte0);
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER, LED_RED_1, LOGGER, RANDOM, DW1000])]
    fn idle() -> ! {
        let mut buffer = [0u8; 1024];
        let mut strbuf: String<U1024> = String::new();
        let mut idle_ctr = 0u32;
        let mut toggle = false;

        resources.LOGGER.start_receive().unwrap();

        loop {
            // Process incoming messages
            if resources.LOGGER.service_receive().unwrap() > 0 {
                if toggle {
                    resources.LED_RED_1.set_low();
                } else {
                    resources.LED_RED_1.set_high();
                }
                toggle != toggle;
                while let Some(msg) = resources.LOGGER.get_msg() {
                    if let ModemUartMessages::LoadLoopBack(msg) = msg {
                        resources.LOGGER.data(ModemUartMessages::LoadLoopBack(msg)).unwrap();
                    } else {
                        resources.LOGGER.log("Odd Message").unwrap();
                    }
                }
            }

            idle_ctr += 1;

            if idle_ctr >= 4 {
                strbuf.clear();
                write!(
                    &mut strbuf,
                    "Stats: {} bytes lost, {} msgs lost, {} bad cobs, {} full buf, {} full msg, {} good msgs, {} good bytes, {} ttl got",
                    resources.LOGGER.dropped_bytes,
                    resources.LOGGER.dropped_msgs,
                    resources.LOGGER.bad_cobs,
                    resources.LOGGER.full_buf,
                    resources.LOGGER.full_msg,
                    resources.LOGGER.good_msgs,
                    resources.LOGGER.good_bytes,
                    resources.LOGGER.ttl_got,
                ).unwrap();
                resources.LOGGER.log(strbuf.as_str()).unwrap();
                idle_ctr = 0;
            }

            resources.TIMER.start(500_000u32);

            while resources.TIMER.wait().is_err() {}
        }
    }
};

const MODEM_PAN: u16 = 0x0386;
const MODEM_ADDR: u16 = 0x0808;
const BROADCAST: u16 = 0xFFFF;

use nb::{
    block,
};


pub fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).expect("wait fail");
}
