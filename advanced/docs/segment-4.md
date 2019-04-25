# Segment 4 detailed instructions

This segment is left open ended to allow students to explore topics that interest them. Students are encouraged to discuss areas of interest with the instructors, and to find a topic to explore with assistance. Recommended topics for this segment include:

1. Developing a driver for the RTC peripheral, learning about making compile-time safety guarantees using [Type State Programming]
2. Use the DWM1001-DEV as a modem, sending commands via Serial Port to control sending and receiving radio packets
3. Implement [`embedded-hal`] traits for your serial port driver

## Instructor's Notes

This segment is aimed to either allow students to continue pursuing their efforts from the previous segment, or to approach a new problem to expand your knowledge in different areas.

If your areas of interest are not listed here, feel free to discuss with your instructor for support, and suggestions of reasonable challenges to tackle during this segment.

## Option 1: Develop a driver for the RTC peripheral

Similar to Segment 2 and 3, this option will allow the student to take the knowledge that they have learned, and apply it to writing a driver for a drastically different peripheral: the Real Time Counter.

This peripheral is typically used to build a real time clock, allowing for timekeeping during low power operation.

In particular, this peripheral is a good candidate for the use of Type State Programming, which allows the driver development to expose a driver that ensures correct operation at compile time.

As the RTC should not allow for changing of the counter prescaler while the counter is running. A generic parameter may be used to only expose certain methods in certain modes.

## Option 2: Use your board as a modem

This option allows the developer to use the serial port driver they have developed (or the canonical HAL UARTE driver) to command the DW1000 modem remotely.

Developers should expose the functionality of the radio, including configuring the modem, sending, and receiving packets, controlled by the host PC.

## Option 3: Implement `embedded-hal` traits

`embedded-hal` traits are one of the tools used to allow the development of portable drivers across different hardware platforms.

Students can implement these traits, in order to allow their driver to be used by abstract drivers.
