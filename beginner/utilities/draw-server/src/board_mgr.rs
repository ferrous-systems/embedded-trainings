use std::time::{Duration, Instant};
use std::sync::mpsc::{Sender, Receiver, RecvTimeoutError};
use reqwest;
use std::collections::hash_map::HashMap;
use std::ops::RangeInclusive;

use protocol::{CellCommand, Cell, ModemUartMessages};
use rand::Rng;

use serde::Deserialize;

type Partitions = HashMap<u16, Segment>;

#[derive(Deserialize, Debug)]
enum BoardMode {
    FreeDraw {
        clear_interval: Duration,
    },
    Partitioned {
        clear_interval: Duration,
        partitions: Partitions,
    },
    RoundRobin {
        turn_interval: Duration,
        notify_interval: Duration,
        players: Vec<u16>,
    }
}

#[derive(Deserialize, Debug)]
pub struct BoardManagerConfig {
    mode: BoardMode,
    total_board: Segment,
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

fn drawing(
    mut client: reqwest::Client,
    cell_endpoint: &str,
    cons_cmds: Receiver<CellCommand>,
    board: &Segment,
    clear_interval: Duration,
    parts: Option<&Partitions>,
) -> Result<(), ()>
{
    let mut last_start = Instant::now();

    loop {
        while last_start.elapsed() < clear_interval {
            let msg = match cons_cmds.recv_timeout(Duration::from_millis(100)) {
                Ok(msg) => Ok(msg),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(e) => {
                    eprintln!("cons_cmds receive error! {:?}", e);
                    Err(())
                }
            }?;

            if let Ok((x, y)) = validate_and_remap(board, parts, &msg) {
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

        clear_map(*board.x.end(), *board.y.end(), &mut client, cell_endpoint);
        last_start = Instant::now();
    }
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

fn turns(
    players: &Vec<u16>,
    board: &Segment,
    mut client: reqwest::Client,
    cell_endpoint: &str,
    cons_cmds: Receiver<CellCommand>,
    prod_rqst: Sender<ModemUartMessages>,
    turn_interval: Duration,
    notify_interval: Duration,
) -> Result<(), ()> {
    let mut boards: HashMap<u16, Vec<Vec<Color>>> = HashMap::new();
    let mut rng = rand::thread_rng();

    // Initialize each board with random colors
    for player in players.iter() {
        let red = rng.gen_range(0, u8::max_value() / 4);
        let grn = rng.gen_range(0, u8::max_value() / 4);
        let blu = rng.gen_range(0, u8::max_value() / 4);

        let mut boardvec = Vec::with_capacity(*board.y.end());

        for _y in 0..*board.y.end() {
            let mut row = Vec::with_capacity(*board.x.end());
            for _x in 0..*board.x.end() {
                row.push(Color { red: red, green: grn, blue: blu });
            }
            boardvec.push(row);
        }

        boards.insert(*player, boardvec);
    }

    for player in players.iter().cycle() {
        let start_turn = Instant::now();

        // Send announcement
        prod_rqst.send(ModemUartMessages::AnnounceTurn(*player)).unwrap();
        let mut last_announce = Instant::now();

        println!("");
        println!("");
        println!("******************************");
        println!("* PLAYER {}, START!", player);
        println!("******************************");
        println!("");
        println!("");

        // Restore board
        set_map(boards.get(player).unwrap(), &mut client, cell_endpoint);

        // Process messages for decided time
        while start_turn.elapsed() < turn_interval {
            if last_announce.elapsed() > notify_interval {
                prod_rqst.send(ModemUartMessages::AnnounceTurn(*player)).unwrap();
                last_announce = Instant::now();
            }

            let msg = match cons_cmds.recv_timeout(Duration::from_millis(100)) {
                Ok(msg) => Ok(msg),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(e) => {
                    eprintln!("cons_cmds receive error! {:?}", e);
                    Err(())
                }
            }.unwrap();

            if msg.source != *player {
                eprintln!("Player {} sent out of turn!", msg.source);
                continue;
            }

            if let Ok((x, y)) = validate_and_remap(board, None, &msg) {
                // We know that the range is valid for the board
                boards.get_mut(player).unwrap()[y-1][x-1] = Color { red: msg.cell.red, green: msg.cell.green, blue: msg.cell.blue };

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
    }

    Ok(())
}

fn set_map(y_x: &Vec<Vec<Color>>, client: &mut reqwest::Client, cell_endpoint: &str) {
    for (i, y) in y_x.iter().enumerate() {
        for (j, x) in y.iter().enumerate() {
            'retry: for _ in 0..3 {
                let req = client
                    .post(cell_endpoint)
                    .json(&Cell {
                        column: j + 1,
                        row: i + 1,
                        red: x.red,
                        green: x.green,
                        blue: x.blue,
                    })
                    .send();

                if req.is_ok() {
                    break 'retry;
                }
            }
        }
    }
}

fn clear_map(x_max: usize, y_max: usize, client: &mut reqwest::Client, cell_endpoint: &str) {
    let mut rng = rand::thread_rng();

    // Time to clear the screen. Pick a muted color, update all pixels
    let red = rng.gen_range(0, u8::max_value() / 4);
    let grn = rng.gen_range(0, u8::max_value() / 4);
    let blu = rng.gen_range(0, u8::max_value() / 4);

    for x in 1..=x_max {
        for y in 1..=y_max {
            'retry: for _ in 0..3 {
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

                if req.is_ok() {
                    break 'retry;
                }
            }
        }
    }
}

pub fn board_mgr_task(
    cfg_sq: &SquaresConfig,
    cfg_bd: &BoardManagerConfig,
    cons_cmds: Receiver<CellCommand>,
    prod_rqst: Sender<ModemUartMessages>,
) -> Result<(), ()>
{
    let client = reqwest::Client::new();
    let cell_endpoint: &str = &format!("{}:{}/cell", cfg_sq.host, cfg_sq.port);

    use BoardMode::*;
    match cfg_bd.mode {
        FreeDraw { clear_interval } => {
            drawing(
                client,
                cell_endpoint,
                cons_cmds,
                &cfg_bd.total_board,
                clear_interval,
                None,
            )
        }
        Partitioned { clear_interval, ref partitions } => {
            drawing(
                client,
                cell_endpoint,
                cons_cmds,
                &cfg_bd.total_board,
                clear_interval,
                Some(partitions),
            )
        }
        RoundRobin { turn_interval, notify_interval, ref players } => {
            turns(
                players,
                &cfg_bd.total_board,
                client,
                cell_endpoint,
                cons_cmds,
                prod_rqst,
                turn_interval,
                notify_interval,
            )
        },
    }
}

fn validate_and_remap(board: &Segment, partitions: Option<&Partitions>, msg: &CellCommand) -> Result<(usize, usize), ()> {
    if let Some(parts) = partitions {
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
        if (board.x.start() <= &msg.cell.column) &&
           (board.x.end() >= &msg.cell.column) &&
           (board.y.start() <= &msg.cell.row) &&
           (board.y.end() >= &msg.cell.row) {
            Ok((msg.cell.column, msg.cell.row))
        } else {
            Err(())
        }

    }
}
