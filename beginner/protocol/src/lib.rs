#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum RadioMessages {
    SetCell(Cell),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ModemUartMessages {
    RawPacket,
    LogMessage,
    SetCell(CellCommand),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Cell {
    row: i32,
    column: i32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CellCommand {
    source: u16,
    dest: u16,
    cell: Cell,
}
