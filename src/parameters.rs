pub const NOMINAL_VOLTAGE: f32 = 3.6; // nominal battery voltage, in volts
pub const OLC_PRECISION: usize = 8; // number of significant digits in the Open Location Code
pub const OLC_CODE_LENGTH: usize = OLC_PRECISION + 8; // length of the Open Location Code, in characters
pub const CALLSIGN: &[u8] = b"KD9TFA"; // callsign of the balloon. MUST be an even number of characters, Space padding at the end is OK.
pub const FLOAT_PRECISION: usize = 8; // number of significant digits in the floating point data
pub const BAUDRATE: u16 = 1;

pub type BareMessage = [u8; BARE_MESSAGE_LENGTH_BYTES];
pub type TotalMessage = [u8; TOTAL_MESSAGE_LENGTH_BYTES];

const BLOCK_LENGTH: usize = 1; // Packet length = 2^BLOCK_LENGTH bytes
pub const BLOCK_DELIMITER: u16 = 0xF0F0; // Delimiter between blocks

pub const START_END_HEADER: u16 = 0x1BE4;
pub const MAX_KISS_FRAME_SIZE: usize = 128; // bytes

pub const FIGURES_FRAME_SIZE: usize = 1024; // bytes

// packet related constants

pub const BLOCK_DELIMITER_SIZE: usize = core::mem::size_of_val(&BLOCK_DELIMITER);
pub const BARE_MESSAGE_LENGTH_BYTES: usize = 64; // Total message length, in bytes.
pub const BARE_MESSAGE_LENGTH_BLOCKS: usize = (BARE_MESSAGE_LENGTH_BYTES) >> (2 ^ BLOCK_LENGTH); // Message length, in blocks, omitting the FEC
pub const PACKET_LENGTH_BYTES: usize = usize::pow(2, BLOCK_LENGTH as u32); // Packet length, in bytes

pub const FEC_EXTRA_PACKETS: usize = 5; // Number of extra packets to send for FEC
pub const FEC_K: usize = BARE_MESSAGE_LENGTH_BYTES >> BLOCK_LENGTH; // Ensures that each packet is 2^BLOCK_LENGTH bytes
pub const FEC_M: usize = FEC_K + FEC_EXTRA_PACKETS;

const _: () = assert!(
    FEC_EXTRA_BYTES == TOTAL_MESSAGE_LENGTH_BYTES - BARE_MESSAGE_LENGTH_BYTES,
    "FEC_BYTES_math_err"
);

const MESSAGE_PREFIX_BLOCKS: usize = 1; // CONSTANT Prefix blocks
const MESSAGE_SUFFIX_BLOCKS: usize = 1; // CONSTANT Suffix blocks

const MESSAGE_NON_DATA_BLOCKS: usize = MESSAGE_PREFIX_BLOCKS + MESSAGE_SUFFIX_BLOCKS;

pub const BLOCK_STACK_DATA_COUNT: usize = BARE_MESSAGE_LENGTH_BLOCKS - MESSAGE_NON_DATA_BLOCKS;

// packet data constants

pub const F64_DATA_SIZE: usize = core::mem::size_of::<f64>();
pub const F32_DATA_SIZE: usize = core::mem::size_of::<f32>();

pub const BLOCK_LABEL_SIZE: usize = 1;

// K is blocks in, M is blocks out. Also, only K blocks are needed to reconstruct the message.

pub const FEC_EXTRA_BYTES: usize = FEC_EXTRA_PACKETS * PACKET_LENGTH_BYTES; // Number of extra bytes to send for FEC
pub const TOTAL_MESSAGE_LENGTH_BYTES: usize = BARE_MESSAGE_LENGTH_BYTES + FEC_EXTRA_BYTES; // Total message length, in bytes

// Start of message header, in binary it is 00 01 10 11 11 10 01 00

pub const START_HEADER_DATA: [u8; CALLSIGN.len() + 2] = [
    START_END_HEADER.to_le_bytes()[0],
    START_END_HEADER.to_le_bytes()[1],
    CALLSIGN[0],
    CALLSIGN[1],
    CALLSIGN[2],
    CALLSIGN[3],
    CALLSIGN[4],
    CALLSIGN[5],
]; // Start of message header data
pub const END_HEADER_DATA: [u8; CALLSIGN.len() + 2] = [
    CALLSIGN[0],
    CALLSIGN[1],
    CALLSIGN[2],
    CALLSIGN[3],
    CALLSIGN[4],
    CALLSIGN[5],
    START_END_HEADER.to_le_bytes()[0],
    START_END_HEADER.to_le_bytes()[1],
]; // End of message header data

const PACKET_BEGINNING_OFFSET: usize = START_HEADER_DATA.len() + BLOCK_DELIMITER_SIZE;

// pub const ALTITUDE_LOCATION_START: usize = PACKET_BEGINNING_OFFSET + BLOCK_LABEL_SIZE; // for the sake of consistency
// pub const ALTITUDE_LOCATION_END: usize = ALTITUDE_LOCATION_START + ALTITUDE_SIZE;

// pub const VOLTAGE_LOCATION_START: usize = ALTITUDE_LOCATION_END + BLOCK_DELIMITER_SIZE + BLOCK_LABEL_SIZE;
// pub const VOLTAGE_LOCATION_END: usize = VOLTAGE_LOCATION_START + VOLTAGE_SIZE;

// pub const TEMPERATURE_LOCATION_START: usize = VOLTAGE_LOCATION_END + BLOCK_DELIMITER_SIZE + BLOCK_LABEL_SIZE;
// pub const TEMPERATURE_LOCATION_END: usize = TEMPERATURE_LOCATION_START + TEMPERATURE_SIZE;

// pub const LATITUDE_LOCATION_START: usize = TEMPERATURE_LOCATION_END + BLOCK_DELIMITER_SIZE + BLOCK_LABEL_SIZE;
// pub const LATITUDE_LOCATION_END: usize = LATITUDE_LOCATION_START + LATITUDE_SIZE;

// pub const LONGITUDE_LOCATION_START: usize = LATITUDE_LOCATION_END + BLOCK_DELIMITER_SIZE + BLOCK_LABEL_SIZE;
// pub const LONGITUDE_LOCATION_END: usize = LONGITUDE_LOCATION_START + LONGITUDE_SIZE;

type BlockConfig = usize;
type BlockConfigStack = [BlockConfig; BLOCK_STACK_DATA_COUNT];

#[derive(Clone, Copy)]
pub struct BlockIdent {
    pub size: BlockConfig,
    pub beginning_location: usize,
    pub end_location: usize,
}

pub type BlockIdentStack = [BlockIdent; BLOCK_STACK_DATA_COUNT];

pub const fn calculate_block_starts_ends(blockconfigs: BlockConfigStack) -> BlockIdentStack {
    let mut _blockidentstack: BlockIdentStack = [BlockIdent {
        size: 4,
        beginning_location: 0,
        end_location: 0,
    }; BLOCK_STACK_DATA_COUNT];

    _blockidentstack[0] = BlockIdent {
        size: blockconfigs[0],
        beginning_location: PACKET_BEGINNING_OFFSET + BLOCK_LABEL_SIZE,
        end_location: PACKET_BEGINNING_OFFSET + BLOCK_LABEL_SIZE + blockconfigs[0],
    };
    let mut _block_in_hand: usize = 1;

    while _block_in_hand < BLOCK_STACK_DATA_COUNT {
        _blockidentstack[_block_in_hand] = BlockIdent {
            size: blockconfigs[_block_in_hand],
            beginning_location: _blockidentstack[_block_in_hand - 1].end_location
                + BLOCK_DELIMITER_SIZE
                + BLOCK_LABEL_SIZE,
            end_location: _blockidentstack[_block_in_hand - 1].end_location
                + BLOCK_DELIMITER_SIZE
                + BLOCK_LABEL_SIZE
                + blockconfigs[_block_in_hand],
        };
        _block_in_hand += 1;
    }
    _blockidentstack
}

pub type U24Arr = [u8; 3];
const U24ARR_SIZE: usize = 24;
pub type U48Arr = [U24Arr; 2];
const U48ARR_SIZE: usize = U24ARR_SIZE * 2;

const PACKED_STATUS_SIZE: usize = U48ARR_SIZE + 8usize + 8usize;
const ALTITUDE_SIZE: usize = F32_DATA_SIZE as usize;
const VOLTAGE_SIZE: usize = F32_DATA_SIZE as usize;
const TEMPERATURE_SIZE: usize = F32_DATA_SIZE as usize;
const LATITUDE_SIZE: usize = F32_DATA_SIZE as usize;
const LONGITUDE_SIZE: usize = F32_DATA_SIZE as usize;

const BLOCK_CONFIG_STACK: BlockConfigStack = [
    PACKED_STATUS_SIZE,
    ALTITUDE_SIZE,
    VOLTAGE_SIZE,
    TEMPERATURE_SIZE,
    LATITUDE_SIZE,
    LONGITUDE_SIZE,
];

const _: () = assert!(BLOCK_CONFIG_STACK.len() == BLOCK_STACK_DATA_COUNT);

pub const BLOCK_IDENT_STACK: BlockIdentStack = calculate_block_starts_ends(BLOCK_CONFIG_STACK);

// const _: () = assert!(BLOCK_IDENT_STACK[0].beginning_location == ALTITUDE_LOCATION_START);
// const _: () = assert!(BLOCK_IDENT_STACK[0].end_location == ALTITUDE_LOCATION_END);

// const _: () = assert!(BLOCK_IDENT_STACK[1].beginning_location == VOLTAGE_LOCATION_START);
// const _: () = assert!(BLOCK_IDENT_STACK[1].end_location == VOLTAGE_LOCATION_END);

// const _: () = assert!(BLOCK_IDENT_STACK[2].beginning_location == TEMPERATURE_LOCATION_START);
// const _: () = assert!(BLOCK_IDENT_STACK[2].end_location == TEMPERATURE_LOCATION_END);

// const _: () = assert!(BLOCK_IDENT_STACK[3].beginning_location == LATITUDE_LOCATION_START);
// const _: () = assert!(BLOCK_IDENT_STACK[3].end_location == LATITUDE_LOCATION_END);

// const _: () = assert!(BLOCK_IDENT_STACK[4].beginning_location == LONGITUDE_LOCATION_START);
// const _: () = assert!(BLOCK_IDENT_STACK[4].end_location == LONGITUDE_LOCATION_END);

// APRS related constants
const APRS_SOFTWARE_VERSION_TXT: &str = "0.0.2";
pub const APRS_SOFTWARE_VERSION: &[u8] = APRS_SOFTWARE_VERSION_TXT.as_bytes();

pub const APRS_FLAG: u8 = 0x7e;

const APRS_DST_ADDR_TXT: &str = "APZNEX";
pub const APRS_DST_ADDR: &[u8] = APRS_DST_ADDR_TXT.as_bytes();

const APRS_SRC_SSID: &[u8] = b"-11";

pub const APRS_SRC_ADDR: [u8; CALLSIGN.len() + APRS_SRC_SSID.len()] = *b"KD9TFA-11";

const APRS_PATH_TXT: &str = "WIDE1-1,WIDE2-1";
pub const APRS_PATH: &[u8] = APRS_PATH_TXT.as_bytes();

pub const APRS_CTRL_FIELD: u8 = 0x03;
pub const APRS_PRTCL_ID: u8 = 0xf0;

pub const APRS_INFO_FIELD_MAX: usize = 256;
pub const APRS_FCS_SIZE: usize = 2;

pub const UI_FRAME_MAX: usize = 1
    + APRS_DST_ADDR.len()
    + APRS_SRC_ADDR.len()
    + APRS_PATH.len()
    + 1
    + 1
    + APRS_INFO_FIELD_MAX
    + APRS_FCS_SIZE;

// TNC parameters

// Figures parameters

pub type FiguresFrameArray = [u8; FIGURES_FRAME_SIZE];
