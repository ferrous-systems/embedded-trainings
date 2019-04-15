# Beginner Course - Instructor's Guide

## Templates

There are three templates for this course:

* `beginner/templates/segment-1`
    * This is used for segments 1 and 2
* `beginner/templates/segment-3`
    * This is used as the basis for segment 3
* `beginner/templates/segment-4`
    * This is used as the basis for segment 4

Each template should compile as-is, and should serve as a basis for the exercise. You should encourage the participants to look at the documentation provided by `cargo doc --open` to find the tools that they will need to solve each exercise.

The templates often contain more crates than are initially used in the template. These crates could be useful for solving the challenge, and will be included in the cargo doc output.

## Utilities

There are several utilities included in the `beginner/utilities/` folder that are intended to be used by the instructor.

### `draw-modem`

This is a firmware image for the DWM1001-DEV attached to your PC. It acts as a serial modem, receiving messages from the radio, and sending them via USB-UART to the PC for further processing. It sends messages in a binary format defined in the `beginner/protocol` crate in this repo, encoded using the `postcard` crate.

### `draw-server`

This is a desktop application that receives serial messages sent by the `draw-modem`, and sends REST requests to the `Squares` server.

It is configured through the `draw.ron` configuration file included in the folder. This configuration file can be used to change the behavior of the server. The server should be re-launched after any configuration changes are made.

By default, the server will partition a 32x32 game board into 16 8x8 squares. Each square is mapped to a radio source address 1..=16, starting at the top left corner of the window. The x/y (column/row) index sent by the students will be remapped to the correct portion of the window (e.g. the students will send indexes 1..=8, no matter what position on the grid they take).

If partitions are removed from the configuration file (e.g. `partitions: None,`), then any source address can write to any pixel on the board, and no remapping will be performed.

The board will periodically wipe with a random color at the interval specified in the configuration file.

### `Squares` Server

This application does not live in this repo, but can be [found on GitHub](https://github.com/ferrous-systems/Squares). You will need to download, build, and run this server separately. The size of the grid specified to the Squares server must match the `draw.ron` configuration of the `draw-server`. 32x32 is recommended for up to 16 students.

### `draw-client-tester`

This is a firmware image for the DWM1001-DEV board, meant to simulate a client firmware written by a student. It will send random pixel messages within a given range, and with random colors. It will choose a source address randomly at boot, so pressing the reset message will (usually) cause it to begin filling a different section of the grid.

This firmware must be run on a separate device than the `draw-modem`.

This has two purposes:

1. To validate the end-to-end behavior of the workshop software
2. To serve as a working example for the instructor, in case they need a reference for the training.

