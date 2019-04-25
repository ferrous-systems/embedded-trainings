# Installation instructions

These instructions will help you setup the development environment necessary for participants of the course.

> If you find these instructions are incomplete or incorrect, please open an issue in this repository!

In general, development on a Linux based PC is recommended, however Windows and MacOS environments are also supported.

In short, the following utilities must be installed:

1. The Rust Compiler ([more...](#the-rust-compiler-and-tools))
2. The `thumbv7em-none-eabihf` target, via `rustup` ([more...](#the-thumbv7em-none-eabihf-target))
3. `arm-none-eabi-gdb` (or `gdb-multiarch`) - for flashing and debugging firmware ([more...](#arm-none-eabi-gdb))
4. J-Link Software and Documentation Pack ([more...](#j-link-software-and-documentation-pack))
5. Git ([more...](#git))

The following utilities are optional:

1. A serial console utility
    * Linux and MacOS users may use `screen`, or other similar included utilities
    * Windows users may use tools such as [PuTTY](https://www.chiark.greenend.org.uk/~sgtatham/putty/latest.html)
    * Users with Python installed can use [pyserial's](https://pyserial.readthedocs.io/en/latest/tools.html) `miniterm` utility.
2. A graphical frontend for GDB, particularly recommended for Windows Users, where the "TUI" mode of GDB may not be included. Examples include:
    * [gdbgui](https://gdbgui.com/) - A web browser frontend for gdb
    * The [Cortex-Debug](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug) plugin for Microsoft VSCode

## The Rust Compiler and Tools

### Linux and MacOS Users

For these platforms, it is recommended to use the [Official Rust Installation Process](https://rustup.rs/) via `rustup.rs`. When asked, the default settings (including the `stable` compiler) are suitable for this course.

This will process will install a number of tools required for this course, including `rustc`, `cargo`, and `rustup`.

### Windows Users

Windows Users may need to install additional tools. Please refer to the [following guide](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-windows) for installation instructions for Windows.


## The `thumbv7em-none-eabihf` target

Once the **Rust Compiler and Tools** have been installed, it is necessary to download support libraries for the microcontroller architecture used for this course. This will allow us to cross compile code for our microcontroller. This step is the same for all platforms (Linux/MacOS/Windows)

To install these librares, run the following command at the command line:

```shell
rustup target add thumbv7em-none-eabihf
```

## `arm-none-eabi-gdb`

To flash and debug code, it is necessary to install `arm-none-eabi-gdb`. On some platforms, this requires installing the entire `arm-none-eabi` toolchain.

Installers for all platforms may be found [via the Arm website](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads). We require **at least** version 5.4.0 or later, and the current stable version of `8-2018-q4-major` is generally recommended.

Once these tools are installed, they must be added to your path, if not done automatically by the installer.

On MacOS `arm-none-eabi-gdb` can also be installed via homebrew, for example with [homebrew-armeabi](https://github.com/eblot/homebrew-armeabi):

* Execute `brew tap eblot/armeabi`
* Install the package with `brew install arm-none-eabi-gdb`

For Linux users, your package manager may contain a package for `arm-none-eabi-gdb`, typically listed as one of the following:

* `arm-none-eabi-gdb`
    * Arch Linux - only installs GDB
    * Fedora: `dnf install arm-none-eabi-gdb` installs gdb and all required dependencies
* `arm-none-eabi-gcc`
    * Some platforms, typically installs the entire suite
* `gcc-arm-none-eabi`
    * Some platforms, typically installs the entire suite
* `gdb-multiarch`
    * Newer versions of Ubuntu ship a universal version of GDB called `gdb-multiarch` rather than an ARM-specific version. Install the `gdb-multiarch` package and then whenever these instructions specify the `arm-none-eabi-gdb` command, substitute the `gdb-multiarch` command instead.

If you install via a package manager, please verify that gdb has been installed, and is suitably new. This can be verified by running the following command via the terminal:

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
For Windows users, when finishing the setup, make sure "Add path to environment variable" is checked.

## J-Link Software and Documentation Pack

This tool is needed to communicate with the debugging device located on the development board.

The installer and installation instructions are located [on the Segger Download Page](https://www.segger.com/downloads/jlink#J-LinkSoftwareAndDocumentationPack)

## Git

The Git version control tool is used for these workshops. Please follow the [Git Installation Instructions](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) if you do not already have Git installed.

## Verifying your installation

Once you have completed the required steps above, please visit [the verification instructions](./VERIFY.md) to ensure all tools are installed successfully.
