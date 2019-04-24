# Segment 1 detailed instructions

This segment aims to familiarize the students with their hardware and embedded Rust tooling. They will take an existing template project and gain familiarity with building, running, and debugging an embedded application.

## Part 0: Instructor Welcome

Your instructor will give you an introduction to the Rust programming language, and the embedded Rust ecosystem.

## Part 1: Building the Application

You will need to clone [this repository] using git, and move to the `beginner/templates/segment-1/` folder.

[this repository]: https://github.com/ferrous-systems/embedded-trainings

To build the firmware application, run the following command:

```shell
cargo build
```

This will compile a "debug build" of the application. If you would like to build a "release build" of the application, you can do that with this command:

```shell
cargo build --release
```

This command should complete successfully. If you see an error, please make sure you have completed all steps in the [Installation Instructions], or ask your instructor for assistance.

[Installation Instructions]: ../../INSTALL.md

Your instructor will explain the components of this project, including:

* `.cargo/config` - Configuration settings for Cargo, the Rust build tool
* `Cargo.toml` - the project/package configuration file
* `src/main.rs` - the code for this project

## Part 2: Getting the Documentation

Rust comes with built-in tools for documenting code and libraries, as well as displaying this documentation. In order to view this documentation, you can run the following command:

```shell
cargo doc --open
```

This will launch a browser window displaying the documentation for all of the dependencies of your application.

You can also browse documentation for released librares at https://docs.rs.

## Part 3: Running the code

Before we can run the code, we will need to launch the debugging interface for our board. Make sure your board is connected via USB, and you see a red light is active on your board. In a separate terminal window, run the following command:

```shell
JLinkGDBServer -device NRF52832_XXAA -if SWD -speed 4000
```

This will launch the debugging server for our device. This window will need to stay open. This command can be run from any location.

For Windows users: in `.cargo/config` the `-tui` flag in line 6 has to be removed.
The J-Link GDB Server has to be started with the following configuration:

Connection to J-Link: USB
Target device: NRF52832_XXAA
Target interface: SWD

Back in your main terminal window, you can now run your application. For this, we will replace the `cargo build` command with a `cargo run` command. Run this command from your terminal window:

```shell
cargo run
```


## Part 4: Debugging code

Once you have run `cargo run`, you will be placed in a debugging session of `gdb`. From here, you can run a number of commands. Some useful commands include:

* `c` or `continue`: Begin execution of your program
* `n` or `next`: Execute the next line of your program
* `s` or `step`: Execute the next line, if the next line includes a call to another function, step into that code
* `b $location` or `break $location`: Set a breakpoint at a place in your code. The value of `$location` can include:
    * `b 123` - Break on line 123 of the currently displayed file
    * `b main.rs:123` - Break on line 123 of the `main.rs`
* `p $data` or `print $data` - Print the value contained by the variable `$data`.
* `monitor reset`: Reset the CPU, starting execution over again

## Part 5: Modify Code

The example code blinks a single LED at a fixed rate. Try blinking other LEDs on the board, and changing the interval at which the LEDs blink. In the next segment, we'll work on making a more complex pattern with the LEDs.
