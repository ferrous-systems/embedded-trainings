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
    Loopback(usize),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CellCommand {
    pub source: u16,
    pub dest: u16,
    pub cell: Cell,
}
