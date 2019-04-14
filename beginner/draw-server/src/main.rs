use std::time::Duration;

use serialport::prelude::*;
use std::sync::mpsc::channel;
use std::thread::{spawn};
use reqwest;

use modem_comms::modem_task;
use protocol::CellCommand;

mod modem_comms;

fn main() {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = 115200;

    let (prod_cmds, cons_cmds) = channel::<CellCommand>();
    let client = reqwest::Client::new();

    spawn(move || {
        while let Ok(msg) = cons_cmds.recv() {
            let req = client
                .post("http://localhost:8000/cell")
                .json(&msg.cell)
                .send();

            if let Err(e) = req {
                eprintln!("post_err: {:?}", e);
            }

        }
    });



    match serialport::open_with_settings("/dev/ttyACM0", &settings) {
        Ok(port) => {
            modem_task(port, prod_cmds).unwrap();
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", "/dev/ttyACM0", e);
            ::std::process::exit(1);
        }
    }
}
