# Segment 3 detailed instructions

This segment pushes the students to modify their serial port to operate in a non-blocking manner. There are multiple ways to achieve this, and students will be free to implement one or more of these approaches. Students will also be introduced to a number of tools which can assist with this, including [`nb`], [`cortex-m-rtfm`], [`heapless`], and [`bbqueue`].

Students will be introduced to the concept of building safe abstractions around unsafe code, and the language invariants that must be upheld to ensure memory safety and prevent compiler mis-optimizations
