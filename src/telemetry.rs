use core::intrinsics::*;
use core::option::Option::Some;

use crate::parameters::*;
use crate::{generate_packet, generate_packet_no_fec};

use reed_solomon::{Decoder, Encoder};

/// Makes a blank packet (with valid FEC) of size [TotalMessage] that is all-zeroes if `false` and all-ones if `true`.
/// If a packet without FEC is desired or can be used, [make_packet_skeleton_nofec] will generate such a packet (sans
/// FEC) at compile-time.
pub fn make_packet_skeleton(_type: bool) -> TotalMessage {
    let _blockstackdata = match _type {
        true => MAX_BLOCKSTACKDATA,
        false => MIN_BLOCKSTACKDATA,
    };
    generate_packet(_blockstackdata)
}

/// Like [make_packet_skeleton], generates an all-zeroes if `false` or all-ones if `true` [BareMessage] (without FEC).
/// Because it does not have FEC, this packet can be generated at compile-time.
pub const fn make_packet_skeleton_nofec(_type: bool) -> BareMessage {
    let _blockstackdata = match _type {
        true => MAX_BLOCKSTACKDATA,
        false => MIN_BLOCKSTACKDATA,
    };
    generate_packet_no_fec(_blockstackdata)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum BlockData {
    DynData(Option<[u8; 4]>),
    StaticData(Option<&'static [u8]>),
}

impl BlockData {
    /// Returns the length of the **data contained**.
    pub fn len(&self) -> usize {
        match *self {
            BlockData::DynData(data) => data.as_ref().map(|d| d.len()).unwrap_or(0),
            BlockData::StaticData(data) => data.as_ref().map(|d| d.len()).unwrap_or(0),
        }
    }

    /// Returns `true` if [self] is of variant type [BlockData::DynData],
    /// otherwise returns `false` if `[self] is of variant type [BlockData::StaticData].
    pub const fn which_type(&self) -> bool {
        match *self {
            BlockData::DynData(_) => true,
            BlockData::StaticData(_) => false,
        }
    }

    /// Returns the contained data as a `&[u8]`, regardless of variant.
    pub const fn get_data(&self) -> &[u8] {
        match self {
            BlockData::DynData(data) => data.as_ref().unwrap(),
            BlockData::StaticData(data) => data.as_ref().unwrap(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Block {
    pub label: u8,
    pub data: BlockData,
    pub do_transmit_label: bool,
}

impl Block {
    /// Returns what the length of the block will be
    /// after being processed into a packet.
    ///
    /// len() = [self].label.len() (if do_transmit_label is true) + [self].data.len() + [BLOCK_DELIMITER_SIZE]
    pub fn len(&self) -> usize {
        (if likely(self.do_transmit_label) {
            BLOCK_LABEL_SIZE
        } else {
            0
        }) + self.data.len()
            + BLOCK_DELIMITER_SIZE
    }
}

pub type BlockStackData = [[u8; 4]; BLOCK_STACK_DATA_COUNT];

pub type PacketDecodedData = [f32; BLOCK_STACK_DATA_COUNT];

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlockStack {
    blocks: [Block; BARE_MESSAGE_LENGTH_BLOCKS],
}

impl BlockStack {
    /// Returns [BARE_MESSAGE_LENGTH_BLOCKS].
    pub const fn len(&self) -> usize {
        self.blocks.len()
    }
}

const MAX_BLOCKSTACKDATA: BlockStackData = [[0xFF; 4]; BLOCK_STACK_DATA_COUNT];

const _MAX_BLOCKSTACK: BlockStack = construct_blocks(&MAX_BLOCKSTACKDATA);

const MIN_BLOCKSTACKDATA: BlockStackData = [[0x00; 4]; BLOCK_STACK_DATA_COUNT];

const _MIN_BLOCKSTACK: BlockStack = construct_blocks(&MIN_BLOCKSTACKDATA);

/// Given a reference to a [BlockStackData] object, construct a [BlockStack] that is
/// ready to be constructed into a packet.
///
/// TODO: make this dynamic
pub const fn construct_blocks(_data: &BlockStackData) -> BlockStack {
    let mut data_location: usize = 0;
    const FIRST_BLOCK_LABEL: u8 = 128;

    const _START_HEADER_BLOCK: Block = Block {
        label: FIRST_BLOCK_LABEL,
        data: BlockData::StaticData(Some(&START_HEADER_DATA)),
        do_transmit_label: false,
    };
    data_location += 1;
    let _packed_status_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    data_location += 1;
    let _altitude_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    data_location += 1;
    let _voltage_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    data_location += 1;
    let _temperature_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    data_location += 1;
    let _latitude_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    data_location += 1;
    let _longitude_block = Block {
        label: FIRST_BLOCK_LABEL + data_location as u8,
        data: BlockData::DynData(Some(_data[data_location - 1])),
        do_transmit_label: true,
    };
    const _END_HEADER_BLOCK: Block = Block {
        label: FIRST_BLOCK_LABEL + BLOCK_STACK_DATA_COUNT as u8,
        data: BlockData::StaticData(Some(&END_HEADER_DATA)),
        do_transmit_label: true,
    };

    BlockStack {
        blocks: [
            _START_HEADER_BLOCK,
            _packed_status_block,
            _altitude_block,
            _voltage_block,
            _temperature_block,
            _latitude_block,
            _longitude_block,
            _END_HEADER_BLOCK,
        ],
    }
}

/// Constructs a packet of shape [u8; [BARE_MESSAGE_LENGTH_BYTES]] from a [BlockStack] object.
/// Each block begins with its 1 byte label attribute (if do_transmit_label is true), followed by the data.
/// Blocks are delimited by [BLOCK_DELIMITER].
///
/// TODO: make fn const
#[rustc_do_not_const_check]
pub const fn construct_packet(_blockstack: BlockStack) -> BareMessage {
    // Constructs a packet from the given blocks. Each block begins with its 1 byte label attribute (if do_transmit_label is true), followed by the data. Blocks are delimited by BLOCK_DELIMITER.
    let mut packet: BareMessage = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut packet_index: usize = 0;

    let mut i = 0;
    while i < _blockstack.len() {
        let block = _blockstack.blocks[i];
        if likely(block.do_transmit_label) {
            // afaict this has genuinely no effect on AVR. too bad!
            packet[packet_index] = block.label.to_be();
            packet_index += 1;
        }

        let _blockdata = block.data.get_data();

        packet[packet_index..(packet_index + block.data.len() as usize)]
            .copy_from_slice(_blockdata);
        packet_index += block.data.len() as usize;
        //packet_index += block.length as usize;

        packet[packet_index] = BLOCK_DELIMITER.to_le_bytes()[0];
        packet[packet_index + 1] = BLOCK_DELIMITER.to_le_bytes()[1];
        packet_index += 2;
        i += 1;
    }

    packet
}

/// Encodes the given packet using the reed_solomon crate. Returns the encoded packet.
pub fn encode_packet(_bare_packet: &BareMessage) -> TotalMessage {
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
        unsafe { floorf32(*self) }
    }
    fn decimal_minutes(&self) -> f32 {
        unsafe { fmul_fast(fsub_fast(*self, self.degrees()), 60.0) }
    }
    fn minutes(&self) -> f32 {
        unsafe { floorf32(self.decimal_minutes()) }
    }
    fn seconds(&self) -> f32 {
        unsafe { fmul_fast(fsub_fast(self.decimal_minutes(), self.minutes()), 60.0) }
    }
}

/// As a framework for decoding a packet, let's base everything off of
/// the same code that is generating the packets.
/// since the block sizes, labels, and positions are always constant, this gives us some help.
///
/// TODO: figure out how to make this function constant, so it all can be constant. there's no reason this can't be calculated at compile time
pub fn find_packet_similarities() -> (BareMessage, BareMessage) {
    let bare_packet = construct_packet(construct_blocks(&MIN_BLOCKSTACKDATA));

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

        packet_bitmask[i] = _mask_max & !_mask_min;
    }

    (packet_bitmask, bare_packet)
}

pub fn decode_packet(_packet: TotalMessage, _known_erasures: &[u8]) -> BareMessage {
    // TODO: this entire function needs to be completely refactored

    let mut _packet_fec: [u8; FEC_EXTRA_BYTES] = [0u8; FEC_EXTRA_BYTES];
    let mut _packet_data: BareMessage = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut _reconstructed_array: BareMessage = [0u8; BARE_MESSAGE_LENGTH_BYTES];
    let mut _mask: u8;

    _packet_data.clone_from_slice(&_packet[..BARE_MESSAGE_LENGTH_BYTES]);
    _packet_fec.clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..]); // there is probably a better way of doing this

    debug_assert_eq!(_packet_data.len() + _packet_fec.len(), _packet.len());
    if (_packet_data.len() + _packet_fec.len()) != _packet.len() {
        unreachable!()
    }

    let (_packet_bitmask, _bare_packet): (BareMessage, BareMessage) = find_packet_similarities();

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        _mask = _packet_bitmask[i];

        match _mask {
            0 => _reconstructed_array[i] = _bare_packet[i],
            _ => _reconstructed_array[i] = _packet[i],
        }
    }

    let mut _packet_data_full: TotalMessage = [0u8; TOTAL_MESSAGE_LENGTH_BYTES];
    _packet_data_full[0..BARE_MESSAGE_LENGTH_BYTES].clone_from_slice(&_reconstructed_array);
    _packet_data_full[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES]
        .clone_from_slice(&_packet[BARE_MESSAGE_LENGTH_BYTES..TOTAL_MESSAGE_LENGTH_BYTES]);

    // now we theoretically have packet that we have reconstructed as well as we can.

    for i in 0..BARE_MESSAGE_LENGTH_BYTES {
        _packet_data_full[i] = _reconstructed_array[i];
    }

    let recovery_buffer = Decoder::new(FEC_EXTRA_BYTES)
        .correct(&mut _packet_data_full, Some(_known_erasures))
        .unwrap();
    let recovered = recovery_buffer.data();

    let mut recovered_packet: BareMessage = [0u8; BARE_MESSAGE_LENGTH_BYTES];

    for i in 0..recovered.len() {
        recovered_packet[i] = recovered[i];
    }

    recovered_packet
    //_packet_data_full

    // let _packet_values = DecodedDataPacket { altitude: 0.0f32, voltage: 0.0f32, temperature: 0.0f32, latitude: 0.0f32, longitude: 0.0f32 };

    // _packet_values
}

pub fn values_from_packet(_packet: BareMessage) -> PacketDecodedData {
    let mut packet_decoded_data: PacketDecodedData = [0.0f32; BLOCK_STACK_DATA_COUNT];
    for i in 0..BLOCK_IDENT_STACK.len() {
        packet_decoded_data[i] = f32::from_be_bytes(
            _packet[BLOCK_IDENT_STACK[i].beginning_location..BLOCK_IDENT_STACK[i].end_location]
                .try_into()
                .unwrap(),
        );
    }

    // TODO: this can be done with a for loop based on parameters
    // let _altitude: f32 = f32::from_be_bytes(_packet[BLOCK_IDENT_STACK[0].beginning_location..BLOCK_IDENT_STACK[0].end_location].try_into().unwrap());
    // let _voltage: f32 = f32::from_be_bytes(_packet[BLOCK_IDENT_STACK[1].beginning_location..BLOCK_IDENT_STACK[1].end_location].try_into().unwrap());
    // let _temperature: f32 = f32::from_be_bytes(_packet[BLOCK_IDENT_STACK[2].beginning_location..BLOCK_IDENT_STACK[2].end_location].try_into().unwrap());
    // let _latitude: f32 = f32::from_be_bytes(_packet[BLOCK_IDENT_STACK[3].beginning_location..BLOCK_IDENT_STACK[3].end_location].try_into().unwrap());
    // let _longitude: f32 = f32::from_be_bytes(_packet[BLOCK_IDENT_STACK[4].beginning_location..BLOCK_IDENT_STACK[4].end_location].try_into().unwrap());

    packet_decoded_data
}

// fn decode_packet_test() -> BareMessage {

//     let mut example_packet: TotalMessage = make_packet_skeleton(true);
//     let dec = Decoder::new(FEC_EXTRA_BYTES);

//     let known_erasures = [0];

//     let recovery_buffer = dec.correct(&mut example_packet, Some(&known_erasures)).unwrap();
//     let recovered = recovery_buffer.data();

//     let mut recovered_packet: BareMessage = [0u8; BARE_MESSAGE_LENGTH_BYTES];

//     for i in 0..recovered.len() {
//         recovered_packet[i] = recovered[i];
//     }

//     recovered_packet

// }

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
