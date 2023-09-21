//! Current combined implementation of both libapollo and apolloTNC.
//!
//! In the future, this will be split into two (three?) parts:
//!
//! - libapollo: apollo-specific packet building, manipulation, and encoding.
//! - apolloTNC: aprs-specific packet building, manipulation, and encoding.
//! Should be a viable no_std library for anyone wishing to implement APRS.
//! Will it be? Who knows.
//!
//! - tnc-template: Akin to balloon-template (an implementation of libapollo),
//! it will be an actual implementation of apolloTNC.

#![no_std]

#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]
#![feature(const_option)]
#![feature(const_mut_refs)]
#![feature(rustc_attrs)]
#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(const_fn_floating_point_arithmetic)]
#![allow(internal_features)]



pub mod parameters;
pub mod telemetry;
pub mod figures;



use parameters::*;
use crate::telemetry::{BlockStackData, construct_blocks, construct_packet, encode_packet};

pub fn generate_packet(_blockstackdata: BlockStackData) -> TotalMessage {
    // _altitude = 1337.69f32.to_be_bytes();
    // _voltage = 420.69f32.to_be_bytes();
    // _temperature = 420.1337f32.to_be_bytes();
    // _latitude = 69.1337f32.to_be_bytes();
    // _longitude = 69.420f32.to_be_bytes();

    // _latitude = 41.1499498f32.to_be_bytes();
    // _longitude = (-87.2426919f32).to_be_bytes();

    //let _blocks: BlockStack = telemetry::construct_blocks(&ALTITUDE, &VOLTAGE, &TEMPERATURE, &LATITUDE, &LONGITUDE);
    let _blocks = construct_blocks(&_blockstackdata);
    let _packet: BareMessage = construct_packet(_blocks);

    encode_packet(&_packet)
}

pub const fn generate_packet_no_fec(_blockstackdata: BlockStackData) -> BareMessage {
    construct_packet(construct_blocks(&_blockstackdata))
}

pub mod aprs;
pub mod tnc;

#[cfg(test)]
mod tests;
