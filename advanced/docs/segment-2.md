# Segment 2 detailed instructions

This segment begins the development of a driver for the UARTE peripheral of the nRF52832. Students will be asked to develop a blocking driver to send and receive bytes over the serial port. They will become familiar with the DMA, Event, and Task based operation of the Nordic peripheral system.

The primary goals of this segment are to implement the following behaviors for the UARTE peripheral:

1. Initialize and configure the UARTE peripheral
2. Send a fixed number of bytes
3. Receive a fixed number of bytes, blocking until reception is complete
4. Receive a maximum number of bytes in a given amount of time, returning when all bytes have been received, or the full time has elapsed

As a stretch goal, students may begin sending binary serialized and encoded data over the interface, and may begin controlling their radio operation via the serial interface.

## Instructors notes

Your instructor will explain how peripherals operate on the nRF52832 chip. Some important high level notes:

* Please refer to the [nRF52832 Datasheet] for detailed information
* Configuration of the peripherals is done through interfacing with memory mapped registers. These operations are performed through interaction with the `nrf52832_pac` library.
* Operations with a configured peripheral are typically initiated by starting a "task". Tasks are started by your code, and cause the hardware to execute a behavior.
* Signals from the hardware are read through "events". Events are set by the hardware, and are read and/or cleared by your code.
* Sending or receiving data means using EasyDMA. A pointer and length is given to the hardware, and a transaction is started by triggering a task. An event will be set when the transaction is complete.

[nRF52832 Datasheet]: ./nrf52832-datasheet-v1_4.pdf

## Step 1: Initializing the peripheral

You will need to configure the UARTE peripheral. This can be done through the `UARTE0` struct passed to the driver skeleton placed in `templates/segment-2/src/uarte.rs`.

You will need to configure a couple things, including:

* What pins are used for RXD, TXD, CTS, and RTS functionality?
* What is the baudrate of the interface?
* What is the parity of the interface?
* The UARTE will need to be enabled

The DWM1001-DEV board does not use Hardware Flow Control, and has no parity. It is recommended to use a baudrate of `115200`. The pins used for serial are:

* RXD: `P0.11`
* TXD: `P0.05`

## Step 2: Sending a fixed number of bytes

Sending bytes requires a couple of steps. These steps include:

1. Setting the starting pointer and number of bytes to be sent in the EasyDMA transaction
2. Starting the transmission with the `starttx` task
3. Waiting for the `endtx` event to occur
4. Clearing the `endtx` event
5. Checking if the transmission completed successfully

**IMPORTANT NOTE**: As the DMA is changing memory outside of the view of the compiler, it is important to use `compiler_fence()`s to tell the compiler not to over-optimize here. This should be done before the transaction has started, and after the transaction has completed.

## Step 3: Receive a fixed number of bytes

Receiving bytes requires a couple of steps. These steps include:

1. Setting the starting pointer and number of bytes to be received in the EasyDMA transaction
2. Starting the reception with the `startrx` task
3. Waiting for the `endrx` event to occur
4. Clearing the `endtx` event
5. Checking if the reception completed successfully

As before, be sure to include `compiler_fence()`s to make sure the compiler does not over-optimize here.

## Step 4: Receive with timeout

This step is similar to step three, however you will need to use a timer to allow for a maximum time to receive.

## Step 5: Stretch Goals

As a stretch goal, students may begin sending binary serialized and encoded data over the interface, and may begin controlling their radio operation via the serial interface.
