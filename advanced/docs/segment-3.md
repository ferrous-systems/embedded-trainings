# Segment 3 detailed instructions

This segment pushes the students to modify their serial port to operate in a non-blocking manner. There are multiple ways to achieve this, and students will be free to implement one or more of these approaches. Students will also be introduced to a number of tools which can assist with this, including [`nb`], [`cortex-m-rtfm`], [`heapless`], and [`bbqueue`].

Students will be introduced to the concept of building safe abstractions around unsafe code, and the language invariants that must be upheld to ensure memory safety and prevent compiler mis-optimizations

[`nb`]: https://crates.io/crates/nb
[`cortex-m-rtfm`]: https://crates.io/crates/cortex-m-rtfm
[`heapless`]: https://crates.io/crates/heapless
[`bbqueue`]: https://crates.io/crates/bbqueue

## Instructor Notes

There are a number of possible ways to proceed here. In general, to perform non-blocking DMA, there are a couple important items:

1. Any memory buffers passed in to the DMA transaction must live long enough for the DMA to complete. This typically means all slices must have `'static` lifetime.
2. While the buffer is being accessed by the DMA, other code MUST NOT read or write from that buffer, or undefined behavior may occur.

The following are possible approaches to solving this problem. They are not the only possible solutions, so feel free to explore other ideas, or ask your instructor whether certain approaches could make sense.

Some of these approaches, particularly ones that allow multiple messages to be enqueued, may require either periodic polling or use of an interrupt to re-trigger follow on DMA transactions.

Additionally, timeouts may require the operation of a timer interrupt, or the use of shortcuts

## Approach 1: Statically Allocated Buffer

In this approach, the driver may define one or more static buffers that can be used for storing data during transmission or reception. These buffers can be statically defined within the driver module, or be passed in at runtime.

However, care must be taken that these buffers are not simultaneously accessed, or undefined behavior will occur!

Depending on your implementation, it may also not be possible to have multiple transactions "in flight" at once, if the buffer is already being used.

## Approach 2: `heapless::pool`

In this approach, you can use the Pool Allocator provided by the `heapless` crate to provide behavior similar to the use of `Box` in an environment with a heap. This allows allocating and freeing segments of memory dynamically at runtime.

This does trade some level of determinism for convenience, and care must be taken to ensure enough memory is available for the use case.

## Approach 3: `bbqueue`

In this approach, you can use the `bbqueue` crate to provide a ring-buffer style queue to store data to be sent, or data that has been received. Each queue may be fed or drained independently, and the queue may be "split".

## Approach 4: `cortex-m-rtfm`

RTFM is a framework for building asyncronous applications in Rust, and provides useful abstractions for sharing data and resources in a memory safe approach.

Although RTFM is not a component that can be used to develop a standalone driver, you may choose to build your driver targeting the RTFM environment, allowing users of RTFM to take advantage of the safe behaviors that it provides.

In this approach, you will need to provide methods that can be used from different "thread contexts", including interupt handling functions that can be called from RTFM tasks.
