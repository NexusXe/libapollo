#![no_std]
#![allow(internal_features)]
#![allow(incomplete_features)]

#![feature(core_intrinsics)]
#![feature(const_size_of_val)]
#![feature(const_likely)]
#![feature(const_option)]
#![feature(const_mut_refs)]
#![feature(rustc_attrs)]
#![feature(const_trait_impl)]



pub mod parameters;
pub mod telemetry;
pub mod tnc;
use parameters::*;
use telemetry::{BlockStack, BlockStackData};

pub fn generate_packet(_blockstackdata: BlockStackData) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    // _altitude = 1337.69f32.to_be_bytes();
    // _voltage = 420.69f32.to_be_bytes();
    // _temperature = 420.1337f32.to_be_bytes();
    // _latitude = 69.1337f32.to_be_bytes();
    // _longitude = 69.420f32.to_be_bytes();

    // _latitude = 41.1499498f32.to_be_bytes();
    // _longitude = (-87.2426919f32).to_be_bytes();

    //let _blocks: BlockStack = telemetry::construct_blocks(&ALTITUDE, &VOLTAGE, &TEMPERATURE, &LATITUDE, &LONGITUDE);
    let _blocks: BlockStack = telemetry::construct_blocks(&_blockstackdata);
    let _packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = telemetry::construct_packet(_blocks);

    telemetry::encode_packet(&_packet)
}

pub const fn generate_packet_no_fec(_blockstackdata: BlockStackData) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    telemetry::construct_packet(telemetry::construct_blocks(&_blockstackdata))
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn make_packet() {
        let _altitude = 1337.69f32.to_be_bytes();
        let _voltage = 420.69f32.to_be_bytes();
        let _temperature = 420.1337f32.to_be_bytes();
        let _latitude = 69.1337f32.to_be_bytes();
        let _longitude = 69.420f32.to_be_bytes();

        let _blocks = telemetry::construct_blocks(&BlockStackData { data_arr: [_altitude, _voltage, _temperature, _latitude, _longitude] } );

        let _packet = telemetry::construct_packet(_blocks);
        telemetry::encode_packet(&_packet);
    }
}