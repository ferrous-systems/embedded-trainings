use std::time::Duration;

use serialport::prelude::*;
use postcard::from_bytes;
use nrf52_bin_logger::LogOnLine;
use cobs::CobsDecoder;
use protocol::ModemUartMessages;

fn main() {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = 115200;

    match serialport::open_with_settings("/dev/ttyACM0", &settings) {
        Ok(mut port) => {
            let mut cobs_buf: Vec<u8> = vec![0; 2048];
            let mut cobs_dec = CobsDecoder::new(cobs_buf.as_mut_slice());
            println!("Receiving data on {} at {} baud:", "/dev/ttyACM0", "115200");

            loop {
                let mut buf = [0u8; 1024];
                match port.read(&mut buf) {
                    Ok(t) => {
                        let mut pos = 0;
                        'inner: loop {
                            match cobs_dec.push(&buf[pos..t]) {
                                Ok(Some((n, m))) => {
                                    match from_bytes::<LogOnLine<ModemUartMessages>>(&cobs_buf[..n]) {
                                        Ok(msg) => display(&msg),
                                        Err(e) => eprintln!("Message decode failed: {}", e),
                                    };

                                    pos += m;
                                    cobs_buf = vec![0; 2048];
                                    cobs_dec = CobsDecoder::new(cobs_buf.as_mut_slice());
                                }
                                Ok(None) => break 'inner,
                                Err(e) => {
                                    // TODO: log levels to see errors. These usually happen at the start,
                                    //   when we have received a bad partial message fragment
                                    eprintln!("Warning: Cobs decoding failed at byte: {}", pos + e);
                                    cobs_buf = vec![0; 2048];
                                    cobs_dec = CobsDecoder::new(cobs_buf.as_mut_slice());
                                    break 'inner;
                                }
                            }
                        }

                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        ::std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", "/dev/ttyACM0", e);
            ::std::process::exit(1);
        }
    }
}

fn display(msg: &LogOnLine<ModemUartMessages>) {
    match msg {
        LogOnLine::Log(m) => {
            println!("{}", prefixed_lines(m, "LOG"))
        }
        _ => {}
    }
}

use chrono::prelude::*;

fn prefixed_lines(st: &str, msg: &str) -> String {
    let mut out = String::new();
    out += &format!("{:?}\n", Local::now());
    st.lines().for_each(|line| {
        out += &format!(
            " => {}: {}\n",
            msg,
            line
        );
    });
    out.truncate(out.trim_end().len());
    out
}
