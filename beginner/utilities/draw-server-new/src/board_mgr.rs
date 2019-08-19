use std::time::{Duration, Instant};
use std::sync::mpsc::{Sender, Receiver, RecvTimeoutError};
use reqwest;
use std::collections::hash_map::HashMap;
use std::ops::RangeInclusive;

use protocol::{Line, Cell, ApiGrid, ModemUartMessages};
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
    endpoint: &str,
    cons_cmds: Receiver<ModemUartMessages>,
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


            match msg {
                ModemUartMessages::SetCell(cell_message) => {

                    if let Ok((x, y)) = validate_and_remap(board, parts, &msg) {


                        let req = client
                            .post(endpoint)
                            .json(&Cell {
                                column: x,
                                row: y,
                                .. cell_message.cell
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }

                }
                ModemUartMessages::SetLine(line_message) => {

                    if let Ok((x, y)) = validate_and_remap(board, parts, &msg) {


                        let req = client
                            .post(endpoint)
                            .json(&Line {
                                column: x,
                                row: y,
                                .. line_message.line
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }

                }
                ModemUartMessages::SetGrid(grid_message) => {

                    if let Ok((x, y)) = validate_and_remap(board, parts, &msg) {

                        let req = client
                            .post(endpoint)
                            .json(&ApiGrid {
                                zero_column: x,
                                zero_row: y,
                                .. grid_message.grid
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }
                }
                _ => continue
            }
        }

        clear_map(*board.x.end(), *board.y.end(), &mut client, endpoint);
        last_start = Instant::now();
    }
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}
//not yet adapted
fn turns(
    players: &Vec<u16>,
    board: &Segment,
    mut client: reqwest::Client,
    endpoint: &str,
    cons_cmds: Receiver<ModemUartMessages>,
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
        let cell_endpoint = "cell";
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

            match msg {
                ModemUartMessages::SetCell(cell_message) => {
                    if cell_message.source != *player {
                        eprintln!("Player {} sent out of turn!", cell_message.source);
                        continue;
                    }

                    if let Ok((x, y)) = validate_and_remap(board, None, &msg) {
                        boards.get_mut(player).unwrap()[y-1][x-1] = Color { red: cell_message.cell.red, green: cell_message.cell.green, blue: cell_message.cell.blue };

                        let req = client
                            .post(endpoint)
                            .json(&Cell {
                                column: x,
                                row: y,
                                .. cell_message.cell
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }

                }
                ModemUartMessages::SetLine(line_message) => {
                    if line_message.source != *player {
                        eprintln!("Player {} sent out of turn!", line_message.source);
                        continue;
                    }

                    if let Ok((x, y)) = validate_and_remap(board, None, &msg) {
                        //boards.get_mut(player).unwrap()[y-1][x-1] = Color { red: line_message.line.red, green: line_message.line.green, blue: line_message.line.blue };

                        let req = client
                            .post(endpoint)
                            .json(&Line {
                                column: x,
                                row: y,
                                .. line_message.line
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }

                }
                ModemUartMessages::SetGrid(grid_message) => {
                    if grid_message.source != *player {
                        eprintln!("Player {} sent out of turn!", grid_message.source);
                        continue;
                    }

                    if let Ok((x, y)) = validate_and_remap(board, None, &msg) {

                        //what does this line do?
                        //boards.get_mut(player).unwrap()[y-1][x-1] = Color { red: grid_message.grid.red, green: grid_message.grid.green, blue: grid_message.grid.blue };

                        let req = client
                            .post(endpoint)
                            .json(&ApiGrid {
                                zero_column: x,
                                zero_row: y,
                                .. grid_message.grid
                            })
                            .send();

                        if let Err(e) = req {
                            eprintln!("post_err: {:?}", e);
                        }
                    } else {
                        eprintln!("Out of range: {:?}", msg);
                    }
                }
                _ => continue
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
    cons_cmds: Receiver<ModemUartMessages>,
    prod_rqst: Sender<ModemUartMessages>,
) -> Result<(), ()>
{

    let endpoint_type = find_endpoint_type(&cons_cmds);

    let client = reqwest::Client::new();
    let endpoint: &str = &format!("{}:{}/{}", cfg_sq.host, cfg_sq.port, endpoint_type);
    println!("{}", &endpoint);


    use BoardMode::*;
    match cfg_bd.mode {
        FreeDraw { clear_interval } => {
            drawing(
                client,
                endpoint,
                cons_cmds,
                &cfg_bd.total_board,
                clear_interval,
                None,
            )
        }
        Partitioned { clear_interval, ref partitions } => {
            drawing(
                client,
                endpoint,
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
                endpoint,
                cons_cmds,
                prod_rqst,
                turn_interval,
                notify_interval,
            )
        },
    }
}

fn validate_and_remap(board: &Segment, partitions: Option<&Partitions>, msg: &ModemUartMessages) -> Result<(usize, usize), ()> {

    match msg {
        ModemUartMessages::SetCell(cell_message) => {
            if let Some(parts) = partitions {
                if let Some(part) = parts.get(&cell_message.source) {
                    let xrange = part.x.end() - part.x.start();
                    let yrange = part.y.end() - part.y.start();

                    if (cell_message.cell.column >= 1) &&
                       (cell_message.cell.column <= (1 + xrange)) &&
                       (cell_message.cell.row >= 1) &&
                       (cell_message.cell.row <= (1 + yrange)) {
                        Ok((
                            cell_message.cell.column - 1 + part.x.start(),
                            cell_message.cell.row    - 1 + part.y.start()
                        ))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            } else {
                if (board.x.start() <= &cell_message.cell.column) &&
                   (board.x.end() >= &cell_message.cell.column) &&
                   (board.y.start() <= &cell_message.cell.row) &&
                   (board.y.end() >= &cell_message.cell.row) {
                    Ok((cell_message.cell.column, cell_message.cell.row))
                } else {
                    Err(())
                }

            }


        }
        ModemUartMessages::SetLine(line_message) => {
            if let Some(parts) = partitions {
                if let Some(part) = parts.get(&line_message.source) {
                    let xrange = part.x.end() - part.x.start();
                    let yrange = part.y.end() - part.y.start();

                    if (line_message.line.column >= 1) &&
                       (line_message.line.column <= (1 + xrange)) &&
                       (line_message.line.row >= 1) &&
                       (line_message.line.row <= (1 + yrange)) {
                        Ok((
                            line_message.line.column - 1 + part.x.start(),
                            line_message.line.row   - 1 + part.y.start()
                        ))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            } else {
                if (board.x.start() <= &line_message.line.column) &&
                   (board.x.end() >= &line_message.line.column) &&
                   (board.y.start() <= &line_message.line.row) &&
                   (board.y.end() >= &line_message.line.row) {
                    Ok((line_message.line.column, line_message.line.row))
                } else {
                    Err(())
                }

            }

        }
        ModemUartMessages::SetGrid(grid_message) => {
            if let Some(parts) = partitions {
                if let Some(part) = parts.get(&grid_message.source) {
                    let xrange = part.x.end() - part.x.start();
                    let yrange = part.y.end() - part.y.start();

                    if (grid_message.grid.zero_column >= 1) &&
                       (grid_message.grid.zero_column <= (1 + xrange)) &&
                       (grid_message.grid.zero_row>= 1) &&
                       (grid_message.grid.zero_row <= (1 + yrange)) {
                        Ok((
                            grid_message.grid.zero_column - 1 + part.x.start(),
                            grid_message.grid.zero_row    - 1 + part.y.start()
                        ))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            } else {
                if (board.x.start() <= &grid_message.grid.zero_column) &&
                   (board.x.end() >= &grid_message.grid.zero_column) &&
                   (board.y.start() <= &grid_message.grid.zero_row) &&
                   (board.y.end() >= &grid_message.grid.zero_row) {
                    Ok((grid_message.grid.zero_column, grid_message.grid.zero_row))
                } else {
                    Err(())
                }

            }
        }
        _ => Ok((0,0))
    }

}

pub fn find_endpoint_type (cons_cmds: &Receiver<ModemUartMessages>) -> String {

    let msg = match cons_cmds.recv() {
        Ok(msg) => Ok(msg),
        Err(e) => {
            eprintln!("cons_cmds receive error! {:?}", e);
            Err(())
        }
    }.unwrap();
    match msg {
        ModemUartMessages::SetCell(_cc) => {
            let endpoint_type = "cell";
            endpoint_type.to_string()
        }
        ModemUartMessages::SetLine(_lc) => {
            let endpoint_type = "line";
            endpoint_type.to_string()
        }
        ModemUartMessages::SetGrid(_gc) => {
            let endpoint_type = "grid";
            endpoint_type.to_string()
        }
        _ => " ".to_string()
    }
}
