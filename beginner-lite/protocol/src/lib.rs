#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum RadioMessages {
    // Messages from clients to modem
    SetCell(Cell),
    SetGrid(ApiGrid),
    SetLine(Line),

    // Messages from modem to clients
    StartTurn(u16),
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ModemUartMessages {
    // Messages to the host system
    SetCell(CellCommand),
    SetGrid(GridCommand),
    SetLine(LineCommand),
    // Commands to the embedded device
    Loopback(u32),
    AnnounceTurn(u16),

    // Misc
    LoadLoopBack([u64; 16])
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct Line {
    pub row: usize,
    pub column: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub direction: i32,
    pub length: i32,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ApiGrid {
    pub zero_row: usize,
    pub zero_column: usize,
    pub grid: [[RGB; 8]; 8],
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct CellCommand {
    pub source: u16,
    pub dest: u16,
    pub cell: Cell,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct GridCommand {
    pub source: u16,
    pub dest: u16,
    pub grid: ApiGrid,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct LineCommand {
    pub source: u16,
    pub dest: u16,
    pub line: Line,
}
