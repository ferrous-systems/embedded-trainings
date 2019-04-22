# Segment 2 detailed instructions

This segment begins the development of a driver for the UARTE peripheral of the nRF52832. Students will be asked to develop a blocking driver to send and receive bytes over the serial port. They will become familiar with the DMA, Event, and Task based operation of the Nordic peripheral system.

The primary goals of this segment are to implement the following behaviors for the UARTE peripheral:

1. Initialize and configure the UARTE peripheral
2. Send a fixed number of bytes
3. Receive a fixed number of bytes, blocking until reception is complete
4. Receive a maximim number of bytes in a given amount of time, returning when all bytes have been received, or the full time has elapsed

As a stretch goal, students may begin sending binary serialized and encoded data over the interface, and may begin controlling their radio operation via the serial interface.
