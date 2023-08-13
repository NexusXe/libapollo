use crate::generate_packet;
use crate::parameters::*;

extern crate crc16;
extern crate reed_solomon;
use reed_solomon::{Encoder, Decoder};
use zerocopy::AsBytes;
use zerocopy::FromBytes;
use core::intrinsics::*;
use serde::*;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Block<const S: usize> {
    pub label: u8,
    pub data: [u8; S],
    pub do_transmit_label: bool,
}

impl<const S: usize> Block<S> {
    pub const fn len(&self) -> usize {
        // each block is its label, data, and then the delimiter
        (if likely(self.do_transmit_label) {BLOCK_LABEL_SIZE} else {0}) + self.data.len() + BLOCK_DELIMITER_SIZE
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlockStack {
    blocks: [Block<4>; BARE_MESSAGE_LENGTH_BLOCKS],
    // altitude_block: Block<ALTITUDE_SIZE>,
    // voltage_block: Block<VOLTAGE_SIZE>,
    // temperature_block: Block<TEMPERATURE_SIZE>,
    // latitude_block: Block<LATITUDE_SIZE>,
    // longitude_block: Block<LONGITUDE_SIZE>,
}

pub fn construct_blocks(altitude: &'static [u8; ALTITUDE_SIZE], voltage: &'static [u8; VOLTAGE_SIZE], temperature: &'static [u8; TEMPERATURE_SIZE], latitude: &'static [u8; LATITUDE_SIZE], longitude: &'static [u8; LONGITUDE_SIZE]) -> BlockStack {

    // let _start_header_block: Block<START_HEADER_DATA_LEN> = Block {
    //     label: 128,
    //     data: START_HEADER_DATA,
    //     do_transmit_label: false,
    // };
    let _altitude_block: Block<ALTITUDE_SIZE> = Block {
        label: 129,
        data: *altitude,
        do_transmit_label: true,
    };
    let _voltage_block: Block<VOLTAGE_SIZE> = Block {
        label:  130,
        data: *voltage,
        do_transmit_label: true,
    };
    let _temperature_block: Block<TEMPERATURE_SIZE> = Block {
        label: 131,
        data: *temperature,
        do_transmit_label: true,
    };
    let _latitude_block: Block<LATITUDE_SIZE> = Block {
        label: 132,
        data: *latitude,
        do_transmit_label: true,
    };
    let _longitude_block: Block<LONGITUDE_SIZE> = Block {
        label: 133,
        data: *longitude,
        do_transmit_label: true,
    };
    // let _end_header_block: Block<END_HEADER_DATA_LEN> = Block {
    //     label: 134,
    //     data: END_HEADER_DATA,
    //     do_transmit_label: true,
    // };
    BlockStack {
        // start_header_block: _start_header_block,
        blocks: [
            _altitude_block,
            _voltage_block,
            _temperature_block,
            _latitude_block,
            _longitude_block
        ]
        // end_header_block: _end_header_block,
    }
}

pub fn construct_packet(_blockstack: BlockStack) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    // Constructs a packet from the given blocks. Each block begins with its 1 byte label attribute (if do_transmit_label is true), followed by the data. Blocks are delimited by BLOCK_DELIMITER.
    let mut packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0; BARE_MESSAGE_LENGTH_BYTES];
    let mut packet_index: usize = 0;

    unsafe {

        // first we do the start header
        packet[packet_index..packet_index + START_HEADER_DATA_LEN].copy_from_slice(&START_HEADER_DATA);
        packet_index = unchecked_add(packet_index, START_HEADER_DATA_LEN);
        packet[packet_index] = BLOCK_DELIMITER.to_le_bytes()[0];
        packet[unchecked_add(packet_index, 1)] = BLOCK_DELIMITER.to_le_bytes()[1];
        packet_index = unchecked_add(packet_index, 2);

        for block in _blockstack.blocks.iter() {
            if likely(block.do_transmit_label) { // afaict this has genuinely no effect on AVR. too bad!
                packet[packet_index] = block.label.to_be();
                packet_index = unchecked_add(packet_index, 1);
            }
            packet[packet_index..(packet_index + block.data.len() as usize)].copy_from_slice(&block.data);
            packet_index = unchecked_add(packet_index, block.data.len() as usize);
            //packet_index += block.length as usize;
            
            packet[packet_index] = BLOCK_DELIMITER.to_le_bytes()[0];
            packet[unchecked_add(packet_index, 1)] = BLOCK_DELIMITER.to_le_bytes()[1];
            packet_index = unchecked_add(packet_index, 2);
        }

        // last we do the end header
        packet[packet_index] = 255u8.to_le_bytes()[0];
        packet_index = unchecked_add(packet_index, 1);
        packet[packet_index..packet_index + END_HEADER_DATA_LEN].copy_from_slice(&END_HEADER_DATA);
        packet_index = unchecked_add(packet_index, END_HEADER_DATA_LEN);
        packet[packet_index] = BLOCK_DELIMITER.to_le_bytes()[0];
        packet[unchecked_add(packet_index, 1)] = BLOCK_DELIMITER.to_le_bytes()[1];
    }
    
    packet
}

pub fn encode_packet(&_bare_packet: &[u8; BARE_MESSAGE_LENGTH_BYTES]) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    // Encodes the given packet using the reed_solomon crate. Returns the encoded packet.
    let enc = Encoder::new(FEC_EXTRA_BYTES);
    let _encoded_packet = enc.encode(&_bare_packet[..]);
    _encoded_packet[..].try_into().unwrap()
}

// pub fn decimal_to_dms(decimal_degrees: f32) -> (i16, u8, f32) {
//     unsafe {
//         let degrees = roundf32(decimal_degrees);
//         let minutes = roundf32(fmul_fast(fsub_fast(decimal_degrees, degrees), 60.0));
//         let seconds = fmul_fast(fmul_fast(fsub_fast(decimal_degrees, degrees), fsub_fast(60.0, minutes)), 60.0);
//         (degrees as i16, minutes as u8, seconds as f32)
//     }
// }

trait CoordinateAttributes {
    fn degrees(&self) -> f32;
    fn decimal_minutes(&self) -> f32;
    fn minutes(&self) -> f32;
    fn seconds(&self) -> f32;
}


impl CoordinateAttributes for f32 {
    fn degrees(&self) -> f32 {
        unsafe{floorf32(*self)}
    }
    fn decimal_minutes(&self) -> f32 {
        unsafe{fmul_fast(fsub_fast(*self, self.degrees()), 60.0)}
    }
    fn minutes(&self) -> f32 {
        unsafe{floorf32(self.decimal_minutes())}
    }
    fn seconds(&self) -> f32 {
        unsafe{fmul_fast(fsub_fast(self.decimal_minutes(), self.minutes()), 60.0)}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decimal_to_dms() {
        unsafe {
            static mut decimal_degrees: f32 = 123.4567;
            assert_eq!(decimal_degrees.degrees() as i16, 123i16);
            assert_eq!(decimal_degrees.minutes() as u8, 27u8);
            //assert_eq!(decimal_degrees.seconds(), 24.12f32);
            assert!(decimal_degrees.seconds() - 24.12f32 <= 0.15); // silly little floating point numbers
        }
    }
    #[test]
    fn test_decode_packet() {
        let mut _packet = generate_packet(location_filler, f32_filler, f32_filler, f32_filler);
        let mut _torture_packet = _packet.clone();
        for i in 0..14 {
            _torture_packet[i] = 0x00;
        }
        assert_eq!(decode_packet(_torture_packet), _packet[0..BARE_MESSAGE_LENGTH_BYTES], "\ndecoded packets were not the same:\nleft    : {:02x?}\nright   : {:02x?}\noriginal: {:02x?}", decode_packet(_packet), &_packet[0..BARE_MESSAGE_LENGTH_BYTES], _torture_packet);
    }
}


// http://www.aprs.org/doc/APRS101.PDF

pub const FLAG      : &'static  u8  = &APRS_FLAG      ;
pub const DST_ADDR  : &'static [u8] = &APRS_DST_ADDR  ;
pub const SRC_ADDR  : &'static [u8] = &APRS_SRC_ADDR  ;
pub const PATH      : &'static [u8] = &APRS_PATH      ;
pub const CTRL_FIELD: &'static  u8  = &APRS_CTRL_FIELD;
pub const PRTCL_ID  : &'static  u8  = &APRS_PRTCL_ID  ; 

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct AX25InformationField {
    data_type: u8,
    data: &'static [u8],
    data_extension: [u8; 7],
}

#[derive(Debug, Copy, Clone)]
struct AX25Block {
    information_field: [u8; 256],
    frame_check_sequence: [u8; 2],
}

impl AX25Block {
    pub fn to_frame(&self) -> [u8; UI_FRAME_MAX] {
        let mut _frame = [0u8; UI_FRAME_MAX];
        _frame.clone_from_slice(&[*FLAG]);
        _frame.clone_from_slice(DST_ADDR);
        _frame.clone_from_slice(SRC_ADDR);
        _frame.clone_from_slice(PATH);
        _frame.clone_from_slice(&[*CTRL_FIELD]);
        _frame.clone_from_slice(&[*PRTCL_ID]);
        _frame.clone_from_slice(&self.information_field);
        _frame.clone_from_slice(&self.frame_check_sequence);
        _frame
    }
}

const fn build_fcs(_frame: &[u8]) -> [u8; 2] {
    let _fcs: [u8; 2] = [0x69 as u8, 0x69 as u8]; // placeholder
    _fcs
}

pub unsafe fn build_aprs_data() -> [u8; UI_FRAME_MAX] {
    
    let mic_e_data: [u8; 7];
    
    let latitude: f32 = f32::from_be_bytes(crate::_latitude);
    let longitude: f32 = f32::from_be_bytes(crate::_longitude);
    
    let current_ui_frame: AX25Block = AX25Block { information_field: [0u8; 256], frame_check_sequence: [0u8; 2] };
    let fcs: [u8; 2] = build_fcs(&current_ui_frame.to_frame());
    
    
    //let (degrees, minutes, seconds) = decimal_to_dms();
    //println!("{}° {}' {}\"", degrees, minutes, seconds);
    current_ui_frame.to_frame()
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromBytes, Serialize, Deserialize)]
pub struct DecodedDataPacket {
    pub altitude: f32,
    pub voltage: f32,
    pub temperature: f32,
    pub latitude: f32,
    pub longitude: f32
}

const NO_VALUE_F16: &[u8; F16_DATA_SIZE as usize] = &[0u8; F16_DATA_SIZE as usize];
const NO_VALUE_F32: &[u8; F32_DATA_SIZE as usize] = &[0u8; F32_DATA_SIZE as usize];

const fn location_filler() -> ([u8; 4], [u8; 4]) {
    ([0xFF as u8; 4], [0xFF as u8; 4])
}

const fn f32_filler() -> [u8; 4] {
    [0xFF as u8; F32_DATA_SIZE as usize]
}

const fn f16_filler() -> [u8; 2] {
    [0xFF as u8; F16_DATA_SIZE as usize]
}



pub fn decode_packet(_packet: [u8; TOTAL_MESSAGE_LENGTH_BYTES]) -> [u8; BARE_MESSAGE_LENGTH_BYTES] { // TODO: this entire function needs to be completely refactored

    #[inline]
    fn construct_bare_refs() -> (BlockStack, [u8; BARE_MESSAGE_LENGTH_BYTES]) {
        (construct_blocks(NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32),
        construct_packet(construct_blocks(NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32, NO_VALUE_F32)))
    }
    // as a framework for decoding a packet, let's base everything off of
    // the same code that is generating the packets.
    // since the block sizes, labels, and positions are always constant, this gives us some help.


    let (_bare_blocks, _bare_packet): (BlockStack, [u8; BARE_MESSAGE_LENGTH_BYTES]) = construct_bare_refs();

    assert_eq!(_bare_blocks.blocks.len(), BARE_MESSAGE_LENGTH_BLOCKS);
    assert_eq!(_bare_packet.len(), BARE_MESSAGE_LENGTH_BYTES);


    let mut _packet_fec: [u8; FEC_EXTRA_BYTES] = [0u8; FEC_EXTRA_BYTES];
    _packet_fec.clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..]);

    assert_eq!(FEC_EXTRA_BYTES, TOTAL_MESSAGE_LENGTH_BYTES - BARE_MESSAGE_LENGTH_BYTES);

    let mut _packet_data: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    _packet_data.clone_from_slice(&_packet[..BARE_MESSAGE_LENGTH_BYTES]);

    assert_eq!(_packet_data.len() + _packet_fec.len(), _packet.len());


    // we can't rely on our delimiters or labels solely to split up the packet, as data may interfere with that
    // however, in this bare packet, this won't happen.
    // this isn't the best solution, as use-cases with troublesome callsigns or larger/different block
    // configurations may interfere.

    let mut _max_example_packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    _max_example_packet.clone_from_slice(&generate_packet(location_filler, f32_filler, f32_filler, f32_filler)[0..BARE_MESSAGE_LENGTH_BYTES]);

    // create a bitmask, showing what's different between our maxed example packet and our bare packet
    // 0 will indicate that the XOR was 0, thus meaning the values are static.

    let mut _packet_bitmask: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut mask: u8;

    let mut _reconstructed_array: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        mask = _bare_packet[i] ^ _max_example_packet[i];
        _packet_bitmask[i] = mask;

        match mask {
            0 => _reconstructed_array[i] = _packet[i],
            _ => _reconstructed_array[i] = _packet[i],
        }
    }
    
    
    let dec = Decoder::new(FEC_EXTRA_BYTES);

    let known_erasures = [0];

    let mut _packet_data_full: [u8; TOTAL_MESSAGE_LENGTH_BYTES] = [0u8; TOTAL_MESSAGE_LENGTH_BYTES];
    _packet_data_full[0..BARE_MESSAGE_LENGTH_BYTES].clone_from_slice(&_reconstructed_array);
    _packet_data_full[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES].clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES]);

    // now we theoretically have packet that we have reconstructed as well as we can.

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        _packet_data_full[i] = _reconstructed_array[i];
    }
    
    

    let recovery_buffer = dec.correct(&mut _packet_data_full, Some(&known_erasures)).unwrap();
    let recovered = recovery_buffer.data();

    let mut recovered_packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    for i in 0..recovered.len() {
      recovered_packet[i] = recovered[i];
    }

    recovered_packet
    //_packet_data_full

    // let _packet_values = DecodedDataPacket { altitude: 0.0f32, voltage: 0.0f32, temperature: 0.0f32, latitude: 0.0f32, longitude: 0.0f32 };

    // _packet_values

}

pub fn values_from_packet(_packet: [u8; BARE_MESSAGE_LENGTH_BYTES]) -> DecodedDataPacket {
    // TODO: these are evaluated at runtime, not compile time
    assert_eq!(ALTITUDE_LOCATION_END - ALTITUDE_LOCATION_START, ALTITUDE_SIZE);
    assert_eq!(VOLTAGE_LOCATION_END - VOLTAGE_LOCATION_START, VOLTAGE_SIZE);
    assert_eq!(TEMPERATURE_LOCATION_END - TEMPERATURE_LOCATION_START, TEMPERATURE_SIZE);
    assert_eq!(LATITUDE_LOCATION_END - LATITUDE_LOCATION_START, LATITUDE_SIZE);
    assert_eq!(LONGITUDE_LOCATION_END - LONGITUDE_LOCATION_START, LONGITUDE_SIZE);

    let mut _conversion_slice: [u8; 4] = [0u8; 4];
    _conversion_slice.clone_from_slice(&_packet[ALTITUDE_LOCATION_START..ALTITUDE_LOCATION_END]);
    let _altitude: f32 = f32::from_be_bytes(_conversion_slice);
    _conversion_slice.clone_from_slice(&_packet[VOLTAGE_LOCATION_START..VOLTAGE_LOCATION_END]);
    let _voltage: f32 = f32::from_be_bytes(_conversion_slice);
    _conversion_slice.clone_from_slice(&_packet[TEMPERATURE_LOCATION_START..TEMPERATURE_LOCATION_END]);
    let _temperature: f32 = f32::from_be_bytes(_conversion_slice);
    _conversion_slice.clone_from_slice(&_packet[LATITUDE_LOCATION_START..LATITUDE_LOCATION_END]);
    let _latitude: f32 = f32::from_be_bytes(_conversion_slice);
    _conversion_slice.clone_from_slice(&_packet[LONGITUDE_LOCATION_START..LONGITUDE_LOCATION_END]);
    let _longitude: f32 = f32::from_be_bytes(_conversion_slice);

    DecodedDataPacket {
        altitude: _altitude,
        voltage: _voltage,
        temperature: _temperature,
        latitude: _latitude,
        longitude: _longitude
    }
}


pub fn decode_packet_test() -> [u8; BARE_MESSAGE_LENGTH_BYTES] {

    let mut example_packet: [u8; TOTAL_MESSAGE_LENGTH_BYTES] = generate_packet(location_filler, f32_filler, f32_filler, f32_filler);
    let dec = Decoder::new(FEC_EXTRA_BYTES);

    let known_erasures = [0];

    let recovery_buffer = dec.correct(&mut example_packet, Some(&known_erasures)).unwrap();
    let recovered = recovery_buffer.data();

    let mut recovered_packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    for i in 0..recovered.len() {
        recovered_packet[i] = recovered[i];
    }

    recovered_packet

}