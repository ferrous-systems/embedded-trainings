use std::time::{Duration, Instant};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use reqwest;
use std::collections::hash_map::HashMap;
use std::ops::RangeInclusive;

use protocol::{CellCommand, Cell};
use rand::Rng;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BoardManagerConfig {
    pub clear_interval: Duration,
    pub partitions: Option<HashMap<u16, Segment>>,
    pub total_board: Segment,
}

#[derive(Deserialize, Debug)]
pub struct SquaresConfig {
    host: String,
    port: u16,
}


#[derive(Deserialize, Debug)]
pub struct Segment {
    pub x: RangeInclusive<usize>,
    pub y: RangeInclusive<usize>,
}

pub fn board_mgr_task(cfg_sq: &SquaresConfig, cfg_bd: &BoardManagerConfig, cons_cmds: Receiver<CellCommand>) -> Result<(), ()> {
    let client = reqwest::Client::new();
    let cell_endpoint: &str = &format!("{}:{}/cell", cfg_sq.host, cfg_sq.port);
    let mut last_start = Instant::now();
    let mut rng = rand::thread_rng();

    loop {
        while last_start.elapsed() < cfg_bd.clear_interval {
            let msg = match cons_cmds.recv_timeout(Duration::from_millis(100)) {
                Ok(msg) => Ok(msg),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(e) => {
                    eprintln!("cons_cmds receive error! {:?}", e);
                    Err(())
                }
            }?;

            if let Ok((x, y)) = validate_and_remap(&cfg_bd, &msg) {
                let req = client
                    .post(cell_endpoint)
                    .json(&Cell {
                        column: x,
                        row: y,
                        .. msg.cell
                    })
                    .send();

                if let Err(e) = req {
                    eprintln!("post_err: {:?}", e);
                }
            } else {
                eprintln!("Out of range: {:?}", msg);
            }


        }

        // Time to clear the screen. Pick a muted color, update all pixels
        let red = rng.gen_range(0, u8::max_value() / 4);
        let grn = rng.gen_range(0, u8::max_value() / 4);
        let blu = rng.gen_range(0, u8::max_value() / 4);

        for x in cfg_bd.total_board.x.clone().into_iter() {
            for y in cfg_bd.total_board.y.clone().into_iter() {
                let req = client
                    .post(cell_endpoint)
                    .json(&Cell {
                        column: x,
                        row: y,
                        red: red,
                        green: grn,
                        blue: blu,
                    })
                    .send();

                if let Err(e) = req {
                    eprintln!("Error clearing screen! {:?}", e);
                }
            }
        }
        last_start = Instant::now();
    }
}

fn validate_and_remap(cfg: &BoardManagerConfig, msg: &CellCommand) -> Result<(usize, usize), ()> {
    if let Some(parts) = &cfg.partitions {
        if let Some(part) = parts.get(&msg.source) {
            let xrange = part.x.end() - part.x.start();
            let yrange = part.y.end() - part.y.start();

            if (msg.cell.column >= 1) &&
               (msg.cell.column <= (1 + xrange)) &&
               (msg.cell.row >= 1) &&
               (msg.cell.row <= (1 + yrange)) {
                Ok((
                    msg.cell.column - 1 + part.x.start(),
                    msg.cell.row    - 1 + part.y.start()
                ))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    } else {
        if (cfg.total_board.x.start() <= &msg.cell.column) &&
           (cfg.total_board.x.end() >= &msg.cell.column) &&
           (cfg.total_board.y.start() <= &msg.cell.row) &&
           (cfg.total_board.y.end() >= &msg.cell.row) {
            Ok((msg.cell.column, msg.cell.row))
        } else {
            Err(())
        }

    }
}
