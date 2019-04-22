use std::io::Read;
use std::time::Duration;

use serialport::prelude::*;
use std::sync::mpsc::channel;
use std::thread::{spawn};

use modem_comms::modem_task;
use board_mgr::board_mgr_task;
use protocol::{CellCommand, ModemUartMessages};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::Path;
use std::fs::File;

mod modem_comms;
mod board_mgr;

#[derive(Deserialize, Debug)]
struct Config {
    serial: SerialConfig,
    squares: board_mgr::SquaresConfig,
    board: board_mgr::BoardManagerConfig,
}

#[derive(Deserialize, Debug)]
struct SerialConfig {
    timeout_ms: u64,
    baudrate: u32,
    port: String,
}

fn main() {
    let config: Config = just_load(Path::new("./draw.ron")).unwrap();
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(config.serial.timeout_ms);
    settings.baud_rate = config.serial.baudrate;

    let (prod_cmds, cons_cmds) = channel::<CellCommand>();
    let (prod_rqst, cons_rqst) = channel::<ModemUartMessages>();

    let port = match serialport::open_with_settings(&config.serial.port, &settings) {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", &config.serial.port, e);
            ::std::process::exit(1);
        }
    };

    let modem_hdl = spawn(move || modem_task(
        port,
        prod_cmds,
        cons_rqst,
        )
    );
    let board_hdl = spawn(move || board_mgr_task(
        &config.squares,
        &config.board,
        cons_cmds,
        prod_rqst,
        )
    );

    modem_hdl.join().unwrap().unwrap();
    board_hdl.join().unwrap().unwrap();
}

/// Attempt to load the contents of a serialized file to a `T`
///
/// If anything goes wrong (file not available, schema mismatch),
/// an error will be returned
pub fn just_load<T>(path: &Path) -> Result<T, ()>
where
    T: DeserializeOwned,
{
    let mut file = File::open(path).map_err(|_| ())?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    ron::de::from_str(&contents).map_err(|e| panic!("{:?}", e))
}
