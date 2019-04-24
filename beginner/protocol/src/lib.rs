#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum RadioMessages {
    // Messages from clients to modem
    SetCell(Cell),

    // Messages from modem to clients
    StartTurn(u16),
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ModemUartMessages {
    // Messages to the host system
    SetCell(CellCommand),

    // Commands to the embedded device
    Loopback(u32),
    AnnounceTurn(u16),

    // Misc
    LoadLoopBack([u64; 16])
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct CellCommand {
    pub source: u16,
    pub dest: u16,
    pub cell: Cell,
}
