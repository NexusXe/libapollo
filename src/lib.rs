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
#![feature(const_float_classify)]
#![feature(const_float_bits_conv)]
#![feature(generic_arg_infer)]


/// Functions relating to APRS and TNC function.
pub mod figures;

/// Constants used across the project.
pub mod parameters;

/// Code for generating, encoding, decoding, and handling packets.
pub mod telemetry;

use crate::telemetry::{construct_blocks, construct_packet, encode_packet, BlockStackData};
use parameters::*;

/// Unifying parameters and telemetry. Cleaning up a *lot* of messiness in parameters.
pub mod qpacket;

pub fn generate_packet(_blockstackdata: BlockStackData) -> TotalMessage {
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
