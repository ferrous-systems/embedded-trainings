use serialport::prelude::*;
use postcard::from_bytes;
use nrf52_bin_logger::LogOnLine;
use protocol::{ModemUartMessages, CellCommand};
use std::sync::mpsc::Sender;

struct Modem {
    port: Box<dyn SerialPort>,
    cobs_buf: Vec<u8>,
}

impl Modem {
    fn process_serial(&mut self) -> Result<Vec<CellCommand>, ()> {
        let mut buf = [0u8; 1024];
        let buf = match self.port.read(&mut buf) {
            Ok(ct) => &buf[..ct],
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => return Ok(vec![]),
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(());
            }
        };

        self.push_bytes(buf)
    }

    fn push_bytes(&mut self, mut data: &[u8]) -> Result<Vec<CellCommand>, ()> {
        let mut resps = vec![];

        while let Some(idx) = data.iter().position(|&b| b == 0) {
            let (end, rest) = data.split_at(idx+1);
            self.cobs_buf.extend_from_slice(end);

            use LogOnLine::*;
            use ModemUartMessages::*;
            if let Ok(idx) = cobs::decode_in_place(&mut self.cobs_buf) {
                match from_bytes::<LogOnLine<ModemUartMessages>>(&self.cobs_buf[..idx]) {
                    Ok(ProtocolMessage(SetCell(desmsg))) =>  {
                        resps.push(desmsg);
                    }
                    Ok(other) => display(&other),
                    Err(e) => {
                        eprintln!("bad_decode: {:?}", e);
                    }
                }
            }

            data = rest;
            self.cobs_buf.clear();
        }

        self.cobs_buf.extend_from_slice(data);

        Ok(resps)
    }
}

pub fn modem_task(port: Box<dyn SerialPort>, prod_cmds: Sender<CellCommand>) -> Result<(), ()> {
    println!("Receiving data on {} at {} baud:", "/dev/ttyACM0", "115200");

    let mut modem = Modem {
        port,
        cobs_buf: vec![],
    };

    loop {
        modem.process_serial()?
            .drain(..)
            .try_for_each(|m| {
                prod_cmds.send(m).map_err(|_| ())
            })?;
    }
}

fn display(msg: &LogOnLine<ModemUartMessages>) {
    match msg {
        LogOnLine::Log(log) => {
            eprintln!("{}", prefixed_lines(log, "LOG"));
        }
        LogOnLine::Warn(log) => {
            eprintln!("{}", prefixed_lines(log, "WRN"));
        }
        LogOnLine::Error(log) => {
            eprintln!("{}", prefixed_lines(log, "ERR"));
        }
        LogOnLine::BinaryRaw(log) => {
            eprintln!("{}", prefixed_lines(&format!("{:02X?}", log), "BIN"));
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
