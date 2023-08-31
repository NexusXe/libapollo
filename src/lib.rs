#![no_std]
#![allow(internal_features)]

#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]
#![feature(const_option)]
#![feature(const_mut_refs)]
#![feature(rustc_attrs)]


pub mod parameters;
pub mod tnc;
pub mod telemetry;
pub mod aprs;
pub mod easypacket;

#[cfg(test)]
mod tests;
