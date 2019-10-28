#![no_main]
#![no_std]

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
            TIMER1,
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
        mac::{Address, PanId, ShortAddress},
        hl::RxConfig,
        Message,
    },
};
use embedded_timeout_macros::TimeoutError;
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

const RX_PERIOD_US: u32 = 100_000;
const IDLE_WARNING_US: u32 = 1_000_000;
const IDLE_STEPDOWN: u32 = IDLE_WARNING_US / RX_PERIOD_US;
type ModemLogger = Logger<
    // Send logs + ModemUartMessages, max outgoing serialized
    // message size is 128 bytes
    RealSender<ModemUartMessages, U128>,

    // Receive ModemUartMessages, max incoming serialized message
    // size is 256 bytes, store up to 8 parsed messages
    RealReceiver<ModemUartMessages, U2048, U64>,
>;


#[app(device = dwm1001::nrf52832_hal::nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER_1:     Timer<TIMER0>             = ();
    static mut TIMER_2:     Timer<TIMER1>             = ();
    static mut LOGGER:    ModemLogger = ();
    static mut DW1000:    Option<DW<
                            Spim<SPIM2>,
                            P0_17<Output<PushPull>>,
                            dw1000::Ready,
                          >> = ();
    static mut DW_RST_PIN: DW_RST                   = ();
    static mut RANDOM:     Rng                      = ();

    #[init]
    fn init() {
        let timer_1 = device.TIMER0.constrain();
        let timer_2 = device.TIMER1.constrain();

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
            None,
        );

        let mut rst_pin = DW_RST::new(pins.p0_24.into_floating_input());

        let mut delay = Delay::new(core.SYST);

        rst_pin.reset_dw1000(&mut delay);
        let mut dw1000 = dw1000.init().unwrap();

        // Wait for the radio to become ready
        loop {
            if dw1000.set_address(MODEM_PAN, MODEM_ADDR).is_err() {
                continue;
            }

            if let Ok(raddr) = dw1000.get_address() {
                if raddr == Address::Short(MODEM_PAN, MODEM_ADDR) {
                    break;
                }
            }
        }

        RANDOM = rng;
        DW_RST_PIN = rst_pin;
        DW1000 = Some(dw1000);
        LOGGER = Logger::new(uarte0);
        TIMER_1 = timer_1;
        TIMER_2 = timer_2;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER_1, TIMER_2, LED_RED_1, LOGGER, RANDOM, DW1000])]
    fn idle() -> ! {
        let mut buffer = [0u8; 1024];
        let mut strbuf: String<U1024> = String::new();
        let mut idle_ctr = 0u32;
        let mut toggle = false;

        resources.LOGGER.start_receive().unwrap();
        resources.TIMER_2.start(250_000u32);

        loop {
            // Process incoming messages
            if resources.TIMER_2.wait().is_err() {
                resources.TIMER_2.start(250_000u32);

                if resources.LOGGER.service_receive().unwrap() > 0 {
                    while let Some(msg) = resources.LOGGER.get_msg() {
                        if toggle {
                            #[allow(deprecated)]
                            resources.LED_RED_1.set_low();
                        } else {
                            #[allow(deprecated)]
                            resources.LED_RED_1.set_high();
                        }
                        toggle = !toggle;

                        match msg {
                            x @ ModemUartMessages::Loopback(_) => {
                                resources.LOGGER.data(x).unwrap();
                            }
                            ModemUartMessages::AnnounceTurn(id) => {
                                let msg = RadioMessages::StartTurn(id);
                                let msg_buf = to_slice(&msg, &mut buffer).unwrap();

                                let mut tx = resources.DW1000
                                    .take()
                                    .expect("tx: dw1000 gone")
                                    .send(
                                        msg_buf,
                                        Address::Short(PanId(0x0386), ShortAddress::broadcast()),
                                        None,
                                    )
                                    .expect("Failed to start send");

                                block!(tx.wait())
                                    .expect("Failed to send data");

                                *resources.DW1000 = Some(tx.finish_sending().expect("failed to finish tx"));
                            }
                            _ => {
                                resources.LOGGER.error("Unexpected Cobs!").unwrap();
                            }
                        }
                    }
                }

            }

            let dw1000 = resources.DW1000.take().expect("rx: dw1000 gone");
            let mut rx = if let Ok(rx) = dw1000.receive(RxConfig::default()) {
                rx
            } else {
                resources.LOGGER.warn("Failed to start receive!").unwrap();
                resources.TIMER_1.delay(250_000);
                continue;
            };

            resources.TIMER_1.start(RX_PERIOD_US);

            let result = block_timeout!(&mut *resources.TIMER_1, rx.wait(&mut buffer));
            *resources.DW1000 = Some(rx.finish_receiving().expect("failed to finish rx"));
            match result {
                Ok(message) => {
                    // Reset idle ctr
                    idle_ctr = 0;

                    if let Ok(resp) = process_message(
                        resources.LOGGER,
                        &message
                    ) {
                        resources.LOGGER.data(resp).unwrap();
                    } else {
                        strbuf.clear();
                        write!(&mut strbuf, "^ Bad message from src {:?}", message.frame.header.source).unwrap();
                        resources.LOGGER.warn(strbuf.as_str()).unwrap();
                    }
                },
                Err(TimeoutError::Timeout) => {
                    idle_ctr += 1;

                    if idle_ctr >= IDLE_STEPDOWN {
                        strbuf.clear();
                        let (lbyt, lmsg) = resources.LOGGER.get_stats();
                        write!(&mut strbuf, "Lost: {} bytes, {} msgs", lbyt, lmsg).unwrap();
                        resources.LOGGER.warn(strbuf.as_str()).unwrap();
                        resources.LOGGER.log("RX Timeout 1s").unwrap();
                        idle_ctr = 0;
                    }

                    // TODO: Rage on the
                    continue;
                }
                Err(TimeoutError::Other(error)) => {
                    strbuf.clear();
                    write!(&mut strbuf, "RX: {:?}", error).unwrap();
                    resources.LOGGER.error(strbuf.as_str()).unwrap();
                    continue;
                }
            };
        }
    }
};

const MODEM_PAN: PanId = PanId(0x0386);
const MODEM_ADDR: ShortAddress = ShortAddress(0x0808);
const BROADCAST: PanId = PanId(0xffff);

fn process_message(logger: &mut ModemLogger, msg: &Message) -> Result<ModemUartMessages, ()> {
    let (src_pan, src_addr) = match msg.frame.header.source {
        Address::Short(pan, addr) => (pan, addr),
        _ => {
            logger.error("bad src addr!").unwrap();
            return Err(())
        }
    };

    let (dst_pan, dst_addr) = match msg.frame.header.destination {
        Address::Short(pan, addr) => (pan, addr),
        _ => {
            logger.error("bad dest addr!").unwrap();
            return Err(())
        }
    };

    if src_pan == BROADCAST {
        logger.error("bad bdcst pan!").unwrap();
        return Err(())
    }

    if src_addr == ShortAddress::broadcast() {
        logger.error("bad bdcst addr!").unwrap();
        return Err(())
    }

    if dst_pan != src_pan {
        logger.error("mismatch pan!").unwrap();
        return Err(())
    }

    if msg.frame.header.destination != Address::Short(MODEM_PAN, MODEM_ADDR) {
        logger.error("that ain't me").unwrap();
        return Err(())
    }

    if let Ok(pmsg) = from_bytes::<RadioMessages>(msg.frame.payload) {
        match pmsg {
            RadioMessages::SetCell(sc) => {
                return Ok(ModemUartMessages::SetCell(CellCommand {
                    source: src_addr.0,
                    dest: dst_addr.0,
                    cell: sc,
                }));
            }
            RadioMessages::StartTurn(_) => {
                logger.warn("ClientMSGS_PER_SEC tried to annouce turn!").unwrap();
            }
        }
    } else {
        logger.warn("Failed to decode!").unwrap();
    }

    Err(())
}

use nb::{
    block,
};


pub fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).expect("wait fail");
}
