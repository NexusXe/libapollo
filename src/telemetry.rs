use crate::{generate_packet, generate_packet_no_fec};
use crate::parameters::*;
use reed_solomon::{Encoder, Decoder};
use zerocopy::AsBytes;
use zerocopy::FromBytes;
use zerocopy::FromZeroes;
use core::intrinsics::*;
use serde::*;
use core::option::Option::Some;

#[rustc_do_not_const_check] // TODO: extremely bad idea
const fn make_packet_skeleton(_type: bool) -> [u8; TOTAL_MESSAGE_LENGTH_BYTES] {
    let _blockstackdata = match _type {
        true => MAX_BLOCKSTACKDATA,
        false => MIN_BLOCKSTACKDATA,
    };
    generate_packet(_blockstackdata)
}

const fn make_packet_skeleton_nofec(_type: bool) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    let _blockstackdata = match _type {
        true => MAX_BLOCKSTACKDATA,
        false => MIN_BLOCKSTACKDATA,
    };
    generate_packet_no_fec(_blockstackdata)
}


#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BlockData {
    DynData(Option<[u8; 4]>),
    StaticData(Option<&'static [u8]>),
}

impl BlockData {
    #[rustc_do_not_const_check]
    pub const fn len(&self) -> usize {
        match self {
            BlockData::DynData(data) => data.as_ref().map(|d| d.len()).unwrap_or(0),
            BlockData::StaticData(data) => data.as_ref().map(|d| d.len()).unwrap_or(0),
        }
    }

    pub const fn which_type(&self) -> Option<bool> {
        match self {
            BlockData::DynData(_) => Some(true),
            BlockData::StaticData(_) => Some(false),
        }
    }

    pub const fn get_data(&self) -> &[u8] {
        match self {
            BlockData::DynData(data) => data.as_ref().unwrap(),
            BlockData::StaticData(data) => data.as_ref().unwrap(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Block {
    pub label: u8,
    pub data: BlockData,
    pub do_transmit_label: bool,
}

impl Block {
    pub const fn len(&self) -> usize {
        // each block is its label, data, and then the delimiter
        (if likely(self.do_transmit_label) {BLOCK_LABEL_SIZE} else {0}) + self.data.len() + BLOCK_DELIMITER_SIZE
    }
    // pub const fn transmit_sections_to_bool(&self) -> [bool; 8] {
    //     let mut bool_stack = [false; 8];

    //     let mut n: u8 = 0b00000001;
    //     let mut i = 0;

    //     while i < 8 {
    //         bool_stack[i] = (self.transmit_sections & n) != 0;
    //         n = n << 1;
    //         i += 1;
    //     }
    //     // label before, label after, BLOCK_DELIMITER before, BLOCK_DELIMITER after, CALLSIGN before, data, START_END_HEADER before, START_END_HEADER after
    //     bool_stack
    // }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockStackData {
    pub data_arr: [[u8; 4]; BLOCK_STACK_DATA_COUNT],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromZeroes, FromBytes, Serialize, Deserialize)]
pub struct PacketDecodedData {
    pub data_arr: [f32; BLOCK_STACK_DATA_COUNT],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize)]
pub struct BlockStack {
    blocks: [Block; BARE_MESSAGE_LENGTH_BLOCKS],
    // altitude_block: Block<ALTITUDE_SIZE>,
    // voltage_block: Block<VOLTAGE_SIZE>,
    // temperature_block: Block<TEMPERATURE_SIZE>,
    // latitude_block: Block<LATITUDE_SIZE>,
    // longitude_block: Block<LONGITUDE_SIZE>,
}

impl BlockStack {
    pub const fn len(&self) -> usize {
        self.blocks.len()
    }
}

const MAX_BLOCKSTACKDATA: BlockStackData = BlockStackData {
    data_arr: [[0xFF; 4]; BLOCK_STACK_DATA_COUNT],
};
const _MAX_BLOCKSTACK: BlockStack = construct_blocks(&MAX_BLOCKSTACKDATA);
// const MAX_PACKET: [u8; BARE_MESSAGE_LENGTH_BYTES] = construct_packet(MAX_BLOCKSTACK);
const MIN_BLOCKSTACKDATA: BlockStackData = BlockStackData {
    data_arr: [[0x00; 4]; BLOCK_STACK_DATA_COUNT],
};
const _MIN_BLOCKSTACK: BlockStack = construct_blocks(&MIN_BLOCKSTACKDATA);
// const MIN_PACKET: [u8; BARE_MESSAGE_LENGTH_BYTES] = construct_packet(MIN_BLOCKSTACK);
pub const fn construct_blocks(_data: &BlockStackData) -> BlockStack {

    const _START_HEADER_BLOCK: Block = Block {
        label: 128,
        data: BlockData::StaticData(Some(&START_HEADER_DATA)),
        do_transmit_label: false,
    };
    let _altitude_block = Block {
        label: 129,
        data: BlockData::DynData(Some(_data.data_arr[0])),
        do_transmit_label: true,
    };
    let _voltage_block = Block {
        label:  130,
        data: BlockData::DynData(Some(_data.data_arr[1])),
        do_transmit_label: true
    };
    let _temperature_block = Block {
        label: 131,
        data: BlockData::DynData(Some(_data.data_arr[2])),
        do_transmit_label: true
    };
    let _latitude_block = Block {
        label: 132,
        data: BlockData::DynData(Some(_data.data_arr[3])),
        do_transmit_label: true
    };
    let _longitude_block = Block {
        label: 133,
        data: BlockData::DynData(Some(_data.data_arr[4])),
        do_transmit_label: true
    };
    const _END_HEADER_BLOCK: Block = Block {
        label: 134,
        data: BlockData::StaticData(Some(&END_HEADER_DATA)),
        do_transmit_label: true
    };

    BlockStack {
        blocks: [
            _START_HEADER_BLOCK,
            _altitude_block,
            _voltage_block,
            _temperature_block,
            _latitude_block,
            _longitude_block,
            _END_HEADER_BLOCK,
        ]
    }
}

/**
Constructs a packet of shape `[u8; BARE_MESSAGE_LENGTH_BYTES]` from a `BlockStack` object.

TODO: make fn const
*/
#[rustc_do_not_const_check]
pub const fn construct_packet(_blockstack: BlockStack) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    // Constructs a packet from the given blocks. Each block begins with its 1 byte label attribute (if do_transmit_label is true), followed by the data. Blocks are delimited by BLOCK_DELIMITER.
    let mut packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0; BARE_MESSAGE_LENGTH_BYTES];
    let mut packet_index: usize = 0;

    let mut i = 0;
    while i < _blockstack.len() {
        let block = _blockstack.blocks[i];
        if likely(block.do_transmit_label) { // afaict this has genuinely no effect on AVR. too bad!
            packet[packet_index] = block.label.to_be();
            packet_index += 1;
        }

        let _blockdata = block.data.get_data();

        packet[packet_index..(packet_index + block.data.len() as usize)].copy_from_slice(_blockdata);
        packet_index += block.data.len() as usize;
        //packet_index += block.length as usize;
        
        packet[packet_index] = BLOCK_DELIMITER.to_le_bytes()[0];
        packet[packet_index + 1] = BLOCK_DELIMITER.to_le_bytes()[1];
        packet_index += 2;
        i += 1;
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
        static DECIMAL_DEGREES: f32 = 123.4567;
        assert_eq!(DECIMAL_DEGREES.degrees() as i16, 123i16);
        assert_eq!(DECIMAL_DEGREES.minutes() as u8, 27u8);
        //assert_eq!(DECIMAL_DEGREES.seconds(), 24.12f32);
        assert!(DECIMAL_DEGREES.seconds() - 24.12f32 <= 0.15); // silly little floating point numbers
    }

    #[test]
    fn test_decode_packet() {
        let mut _packet = make_packet_skeleton(true);
        let mut _torture_packet = _packet.clone();
        for i in 0..18 {
            _torture_packet[i] = 0x00;
        }
        assert_eq!(decode_packet(_torture_packet, &[0u8]), _packet[0..BARE_MESSAGE_LENGTH_BYTES], "\ndecoded packets were not the same:\nleft    : {:02x?}\nright   : {:02x?}\noriginal: {:02x?}", decode_packet(_packet, &[0u8]), &_packet[0..BARE_MESSAGE_LENGTH_BYTES], _torture_packet);
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
        // TODO: there has to be a better way to do this
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

/// We shouldn't have to rely on an external crate for something
/// as simple as a checksum, but CRC16 is a fucking mess.
/// 
/// https://www.reddit.com/r/amateurradio/comments/8o3hlk/aprs_crcfcs_bytes/
const fn build_fcs(_frame: &[u8]) -> [u8; 2] {
    use crc::{Crc, NoTable, CRC_16_IBM_3740};
    const X25: Crc<NoTable<u16>> = Crc::<NoTable<u16>>::new(&CRC_16_IBM_3740);
    X25.checksum(_frame).to_be_bytes()
}

pub fn build_aprs_data(_latitude: f32, _longitude: f32) -> [u8; UI_FRAME_MAX] {
    // todo!();
    let _mic_e_data: [u8; 7];
    let mut current_ui_frame: AX25Block = AX25Block { information_field: [0u8; 256], frame_check_sequence: [0u8; 2] };
    current_ui_frame.frame_check_sequence = build_fcs(&current_ui_frame.to_frame());

    //println!("{}° {}' {}\"", degrees, minutes, seconds);
    current_ui_frame.to_frame()
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromZeroes, FromBytes, Serialize, Deserialize)]
pub struct DecodedDataPacket {
    pub altitude: f32,
    pub voltage: f32,
    pub temperature: f32,
    pub latitude: f32,
    pub longitude: f32
}

pub fn find_packet_similarities() -> ([u8; BARE_MESSAGE_LENGTH_BYTES], [u8; BARE_MESSAGE_LENGTH_BYTES]) {
    // as a framework for decoding a packet, let's base everything off of
    // the same code that is generating the packets.
    // since the block sizes, labels, and positions are always constant, this gives us some help.

    // TODO: figure out how to make this function constant, so it all can be constant. there's no reason this can't be calculated at compile time
    let bare_packet = construct_packet(construct_blocks( &MIN_BLOCKSTACKDATA ));

    debug_assert_eq!(bare_packet.len(), BARE_MESSAGE_LENGTH_BYTES);
    if bare_packet.len() != BARE_MESSAGE_LENGTH_BYTES {
        unreachable!()
    }
    

    // we can't rely on our delimiters or labels solely to split up the packet, as data may interfere with that
    // however, in this bare packet, this won't happen.
    // this isn't the best solution, as use-cases with troublesome callsigns or larger/different block
    // configurations may interfere.

    let mut _max_example_packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    _max_example_packet.clone_from_slice(&make_packet_skeleton_nofec(true));

    let mut _min_example_packet: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    _min_example_packet.clone_from_slice(&make_packet_skeleton_nofec(false));

    // create a bitmask, showing what's different between our maxed example packet and our bare packet
    // 0 will indicate that the XOR was 0, thus meaning the values are static. we do this between both a max-ed and min-ed packet to ensure we don't have flukes.
    // if, however, there are in fact flukes, FEC *should* take care of it.

    let mut packet_bitmask: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut _mask_max: u8;
    let mut _mask_min: u8;

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        _mask_max = bare_packet[i] ^ _max_example_packet[i];
        _mask_min = bare_packet[i] ^ _min_example_packet[i];

        packet_bitmask[i] = _mask_max &! _mask_min;
    }

    (packet_bitmask, bare_packet)

}

pub fn decode_packet(_packet: [u8; TOTAL_MESSAGE_LENGTH_BYTES], _known_erasures: &[u8]) -> [u8; BARE_MESSAGE_LENGTH_BYTES] { // TODO: this entire function needs to be completely refactored

    let mut _packet_fec: [u8; FEC_EXTRA_BYTES] = [0u8; FEC_EXTRA_BYTES];
    let mut _packet_data: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut _reconstructed_array: [u8; BARE_MESSAGE_LENGTH_BYTES] = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut _mask: u8;

    _packet_data.clone_from_slice(&_packet[..BARE_MESSAGE_LENGTH_BYTES]);
    _packet_fec.clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..]); // there is probably a better way of doing this

    debug_assert_eq!(_packet_data.len() + _packet_fec.len(), _packet.len());
    if (_packet_data.len() + _packet_fec.len()) != _packet.len() {
        unreachable!()
    }
    
    
    

    let (_packet_bitmask, _bare_packet): ([u8; BARE_MESSAGE_LENGTH_BYTES], [u8; BARE_MESSAGE_LENGTH_BYTES]) = find_packet_similarities();
    

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        
        _mask = _packet_bitmask[i];

        match _mask {
            0 => _reconstructed_array[i] = _bare_packet[i],
            _ => _reconstructed_array[i] = _packet[i],
        }
    }
    



    let mut _packet_data_full: [u8; TOTAL_MESSAGE_LENGTH_BYTES] = [0u8; TOTAL_MESSAGE_LENGTH_BYTES];
    _packet_data_full[0..BARE_MESSAGE_LENGTH_BYTES].clone_from_slice(&_reconstructed_array);
    _packet_data_full[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES].clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES]);

    // now we theoretically have packet that we have reconstructed as well as we can.

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        _packet_data_full[i] = _reconstructed_array[i];
    }

    let recovery_buffer = Decoder::new(FEC_EXTRA_BYTES).correct(&mut _packet_data_full, Some(_known_erasures)).unwrap();
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

pub fn values_from_packet(_packet: [u8; BARE_MESSAGE_LENGTH_BYTES]) -> PacketDecodedData {
    

    // TODO: this can be done with a for loop based on parameters
    let _altitude: f32 = f32::from_be_bytes(_packet[ALTITUDE_LOCATION_START..ALTITUDE_LOCATION_END].try_into().unwrap());
    let _voltage: f32 = f32::from_be_bytes(_packet[VOLTAGE_LOCATION_START..VOLTAGE_LOCATION_END].try_into().unwrap());
    let _temperature: f32 = f32::from_be_bytes(_packet[TEMPERATURE_LOCATION_START..TEMPERATURE_LOCATION_END].try_into().unwrap());
    let _latitude: f32 = f32::from_be_bytes(_packet[LATITUDE_LOCATION_START..LATITUDE_LOCATION_END].try_into().unwrap());
    let _longitude: f32 = f32::from_be_bytes(_packet[LONGITUDE_LOCATION_START..LONGITUDE_LOCATION_END].try_into().unwrap());

    PacketDecodedData {
        data_arr: [_altitude, _voltage, _temperature, _latitude, _longitude],
    }
}


pub fn decode_packet_test() -> [u8; BARE_MESSAGE_LENGTH_BYTES] {

    let mut example_packet: [u8; TOTAL_MESSAGE_LENGTH_BYTES] = make_packet_skeleton(true);
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
