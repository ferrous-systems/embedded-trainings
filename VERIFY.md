# Verify your environment

The following steps are aimed to ensure you have successfully installed all tools.

## Rust

Please make sure all of the following commands work in your terminal. The versions may vary, but all commands should produce output similar as below:

```shell
$ rustup --version
rustup 1.17.0 (069c88ed6 2019-03-05)
$ rustc --version
rustc 1.34.0 (91856ed52 2019-04-10)
$ cargo --version
cargo 1.34.0 (6789d8a0a 2019-04-01)
```

## Git + `thumbv7em-none-eabihf` target

First, clone this repository using Git:

```shell
$ git clone https://github.com/ferrous-systems/embedded-trainings.git
Cloning into 'embedded-trainings'...
remote: Enumerating objects: 175, done.
remote: Counting objects: 100% (175/175), done.
remote: Compressing objects: 100% (84/84), done.
remote: Total 175 (delta 69), reused 164 (delta 58), pack-reused 0
Receiving objects: 100% (175/175), 78.61 KiB | 725.00 KiB/s, done.
Resolving deltas: 100% (69/69), done.
```

Then, move to one of the beginner training templates (these can also be used for other workshops as well). From here, build the project using cargo:

```
$ cd embedded-trainings/beginner/templates/segment-1
$ cargo build
   Compiling semver-parser v0.7.0
   Compiling proc-macro2 v0.4.27
   Compiling unicode-xid v0.1.0
   ...
   Compiling dwm1001 v0.1.1
   Compiling segment-1 v0.1.0 (/tmp/embedded-trainings/beginner/templates/segment-1)
    Finished dev [unoptimized + debuginfo] target(s) in 32.11s
```

This may take some time if you have just installed Rust for the first time. If the project build successfully, your Rust environment should be ready.

## JLink Tools

If you do not have a `DWM1001-DEV` board handy, it may not be possible to fully verify everything is installed successfully. This can be completed the day of the course. Prior to the course, please make sure the tool was installed. Running the following command, you should see similar output (resulting in an error due to the missing hardware):

```shell
$ JLinkGDBServer -device NRF52832_XXAA -if SWD -speed 4000
SEGGER J-Link GDB Server V6.40 Command Line Version

JLinkARM.dll V6.40 (DLL compiled Oct 26 2018 15:08:28)

Command line: -device NRF52832_XXAA -if SWD -speed 4000
-----GDB Server start settings-----
GDBInit file:                  none
GDB Server Listening port:     2331
SWO raw output listening port: 2332
Terminal I/O port:             2333
Accept remote connection:      yes
Generate logfile:              off
Verify download:               off
Init regs on start:            off
Silent mode:                   off
Single run mode:               off
Target connection timeout:     0 ms
------J-Link related settings------
J-Link Host interface:         USB
J-Link script:                 none
J-Link settings file:          none
------Target related settings------
Target device:                 NRF52832_XXAA
Target interface:              SWD
Target interface speed:        4000kHz
Target endian:                 little

Connecting to J-Link...
Connecting to J-Link failed. Connected correctly?
GDBServer will be closed...
Shutting down...
Could not connect to J-Link.
Please check power, connection and settings.
```

## GDB

This can be verified by running the following command via the terminal:

```shell
$ arm-none-eabi-gdb -v
```

You should see output that looks like this:

```text
GNU gdb (GDB) 8.2.1
Copyright (C) 2018 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
```
