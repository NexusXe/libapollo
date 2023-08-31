use crate::parameters::*;
use crate::telemetry::{BlockStack, BlockStackData, construct_blocks, construct_packet, encode_packet};

pub fn generate_packet(_blockstackdata: BlockStackData) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    // _altitude = 1337.69f32.to_be_bytes();
    // _voltage = 420.69f32.to_be_bytes();
    // _temperature = 420.1337f32.to_be_bytes();
    // _latitude = 69.1337f32.to_be_bytes();
    // _longitude = 69.420f32.to_be_bytes();

    // _latitude = 41.1499498f32.to_be_bytes();
    // _longitude = (-87.2426919f32).to_be_bytes();

    //let _blocks: BlockStack = telemetry::construct_blocks(&ALTITUDE, &VOLTAGE, &TEMPERATURE, &LATITUDE, &LONGITUDE);
    let _blocks: BlockStack = construct_blocks(&_blockstackdata);
    let _packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = construct_packet(_blocks);

    encode_packet(&_packet)
}

pub const fn generate_packet_no_fec(_blockstackdata: BlockStackData) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    construct_packet(construct_blocks(&_blockstackdata))
}
