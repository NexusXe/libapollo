#![no_std]

#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]
#![feature(const_option)]
#![feature(const_mut_refs)]
#![feature(rustc_attrs)]
#![allow(internal_features)]

/**
Current combined implementation of both libapollo and apolloTNC.

In the future, this will be split into two (three?) parts:

- libapollo: apollo-specific packet building, manipulation, and encoding.
- apolloTNC: aprs-specific packet building, manipulation, and encoding.
Should be a viable no_std library for anyone wishing to implement APRS.
Will it be? Who knows.

- tnc-template: Akin to balloon-template (an implementation of libapollo),
it will be an actual implementation of apolloTNC.
*/

pub mod parameters;
pub mod telemetry;
pub mod easypacket;
pub mod aprs;
pub mod tnc;

#[cfg(test)]
mod tests;
