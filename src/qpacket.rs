#![allow(dead_code)]

use core::mem;

use crate::parameters::*;


const FEC_BYTES: usize = 19;

type BlockLabelType = u8;
const FIRST_BLOCK_LABEL: BlockLabelType = 128;

pub const TOTAL_DATA_BLOCKS: usize = 5;
const BLOCK_CFG_STACK: BlockCfgStack = [
    BlockCfg {
        block_type: BlockType::I32, // latitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // longitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // voltage
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // temperature
        do_transmit_label: true,
    },
];

struct BlockCfg {
    block_type: BlockType,
    do_transmit_label: bool,
}

type BlockCfgStack = [BlockCfg; TOTAL_DATA_BLOCKS];

#[derive(Clone, Copy)]
pub struct BlockIdent {
    block_type: BlockType,
    label: BlockLabelType,
    do_transmit_label: bool,
    position: (usize, usize),
}

impl BlockIdent {
    const BLANK: Self = Self::new();

    const fn new() -> Self {
        Self {
            block_type: BlockType::NONE,
            label: 0 as BlockLabelType,
            do_transmit_label: false,
            position: (0usize, 0usize + BlockType::NONE.len()),
        }
    }

    const fn data_len(&self) -> usize {
        self.block_type.len()
    }

    const fn total_len(&self) -> usize {
        debug_assert!(self.position.1 >= self.position.0);
        self.position.1 - self.position.0
    }
}

type BlockIdentStack = [BlockIdent; TOTAL_DATA_BLOCKS];

const fn cfg_stack_to_ident_stack(cfg_stack: BlockCfgStack) -> BlockIdentStack {
    // a block is its label, its data, and the following delimiter
    let mut output: BlockIdentStack = [BlockIdent::BLANK; TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;
    let mut current_packet_position: usize = START_HEADER_DATA.len() + BLOCK_DELIMITER_SIZE; // bytes, accounting for START_HEADER_DATA and its delimiter

    while i < output.len() {
        if i > BlockLabelType::MAX as usize || i >= cfg_stack.len() {
            unreachable!();
        }

        let _block_type = cfg_stack[i].block_type;
        let _do_transmit_label = cfg_stack[i].do_transmit_label;
        let _block_label: BlockLabelType = FIRST_BLOCK_LABEL + i as BlockLabelType;
        let _block_size: usize = _block_type.len()
            + {
                if _do_transmit_label {
                    mem::size_of::<BlockLabelType>()
                } else {
                    0
                }
            }
            + BLOCK_DELIMITER_SIZE;
        let end_position = current_packet_position + _block_size - 1;
        output[i] = BlockIdent {
            block_type: cfg_stack[i].block_type,
            label: _block_label,
            do_transmit_label: _do_transmit_label,
            position: (current_packet_position, end_position),
        };
        current_packet_position = end_position + 1;
        i += 1;
    }
    output
}

const BLOCK_IDENT_STACK: BlockIdentStack = cfg_stack_to_ident_stack(BLOCK_CFG_STACK);
pub const QPACKET_DATA_LEN: usize = BLOCK_IDENT_STACK[BLOCK_IDENT_STACK.len() - 1].position.1 - 1;
const START_END_HEADER_SIZE: usize = mem::size_of_val::<_>(&START_END_HEADER);
pub const QPACKET_BARE_LEN: usize =
    START_HEADER_DATA.len() + QPACKET_DATA_LEN + START_END_HEADER_SIZE;
pub const QPACKET_FULL_LEN: usize = QPACKET_BARE_LEN + FEC_BYTES;
const BLOCK_SIZE_STACK: [usize; TOTAL_DATA_BLOCKS] = {
    let mut output: [usize; TOTAL_DATA_BLOCKS] = [0usize; TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;

    while i < TOTAL_DATA_BLOCKS {
        output[i] = BLOCK_IDENT_STACK[i].total_len();
        i += 1;
    }

    output
};

pub struct QPacketBlock<'a> {
    identity: BlockIdent,
    data: &'a [u8],
}

impl<'a> QPacketBlock<'a> {
    pub const BLANK: Self = Self {
        identity: BlockIdent::BLANK,
        data: &[u8::MAX],
    };

    pub const fn new(_ident: &'static BlockIdent, _data: &'a [u8]) -> Self {
        Self {
            identity: *_ident,
            data: _data,
        }
    }

    pub const fn len(&self) -> usize {
        self.identity.total_len()
    }

    pub const fn as_bytes<const LEN: usize>(&self) -> [u8; LEN] {
        let mut output: [u8; LEN] = [0u8; LEN];

        let mut i: usize = 0usize; // output position
        let mut x: usize = 0usize; // data position (can't use an iterator because this is a const fn)

        // first two bytes are always delimiter
        output[i] = BLOCK_DELIMITER.to_be_bytes()[0];
        i += 1;
        output[i] = BLOCK_DELIMITER.to_be_bytes()[1];
        i += 1;

        if self.identity.do_transmit_label {
            output[i] = self.identity.label;
            i += 1;
        }

        while i < self.len() {
            if i >= self.len() || x >= self.data.len() {
                unreachable!()
            }
            output[i] = self.data[x];
            i += 1;
            x += 1;
        }

        output
    }
}

pub type QPacketBlockStack<'a> = [QPacketBlock<'a>; TOTAL_DATA_BLOCKS];

const fn construct_blank_packet<const DATA: u8>(
    _blockstack: BlockIdentStack,
) -> [u8; QPACKET_BARE_LEN] {
    let mut output: [u8; QPACKET_BARE_LEN] = [0u8; QPACKET_BARE_LEN];
    let mut i: usize = 0;
    let mut x: usize = 0;

    while x < START_HEADER_DATA.len() {
        output[x] = START_HEADER_DATA[x];
        x += 1;
    }

    let mut x: usize = 0;

    // manually insert first delimiter
    while x < BLOCK_DELIMITER_SIZE {
        output[x + START_HEADER_DATA.len()] = BLOCK_DELIMITER.to_be_bytes()[x];
        x += 1;
    }

    while i < _blockstack.len() {
        let _block = &_blockstack[i];

        let mut left = _block.position.0;

        // first is (maybe) the label,
        if _block.do_transmit_label {
            output[left] = _block.label;
            left += 1;
        }

        // then the data,
        while left <= _block.position.1 - BLOCK_DELIMITER_SIZE {
            output[left] = DATA;
            left += 1;
        }

        output[left] = BLOCK_DELIMITER.to_be_bytes()[0]; // and finally the delimiter.
        left += 1;
        output[left] = BLOCK_DELIMITER.to_be_bytes()[1];

        i += 1;
    }

    let mut x: usize = 0;

    while x < END_HEADER_DATA.len() {
        output[output.len() - (END_HEADER_DATA.len() - x)] = END_HEADER_DATA[x];
        x += 1;
    }

    output
}

pub fn construct_packet(_block_stack: QPacketBlockStack) -> [u8; QPACKET_FULL_LEN] {
    let mut unencoded_output: [u8; QPACKET_BARE_LEN] = [0u8; QPACKET_BARE_LEN];
    let mut packet_position: usize = 0;
    unencoded_output[0..START_HEADER_DATA.len()].copy_from_slice(&START_HEADER_DATA);
    packet_position += START_HEADER_DATA.len();
    unencoded_output[packet_position..packet_position + BLOCK_DELIMITER_SIZE].copy_from_slice(&BLOCK_DELIMITER.to_be_bytes());
    
    for _block in _block_stack {
        unencoded_output[_block.identity.position.0.._block.identity.position.1].copy_from_slice(_block.data); // TODO: THIS IS NOT CORRECT! USE DATA BOUNDS INSTEAD
    }

    // todo: configure reed solomon for new configurable packet size
    todo!()
}

pub const MIN_QPACKET: [u8; QPACKET_BARE_LEN] = construct_blank_packet::<0x00u8>(BLOCK_IDENT_STACK);
pub const MAX_QPACKET: [u8; QPACKET_BARE_LEN] = construct_blank_packet::<0xFFu8>(BLOCK_IDENT_STACK);

#[cfg(test)]
mod tests {
    use super::*;

    fn check_packet_formation<const T: usize>(packet: [u8; T]) {
        let mut _packet_head = [0u8; (START_HEADER_DATA.len() + BLOCK_DELIMITER_SIZE)];
        let mut i: usize = 0;
        _packet_head[0..START_HEADER_DATA.len()].copy_from_slice(&START_HEADER_DATA);
        i += START_HEADER_DATA.len();
        _packet_head[i..i + BLOCK_DELIMITER_SIZE].copy_from_slice(&BLOCK_DELIMITER.to_be_bytes());
        assert_eq!(packet[0..i + BLOCK_DELIMITER_SIZE], _packet_head);

        for block in BLOCK_IDENT_STACK {
            assert_eq!(packet[block.position.0], block.label);
            for x in 0..BLOCK_DELIMITER.to_be_bytes().len() {
                const BLOCK_DELIMITER_BYTES: [u8; BLOCK_DELIMITER_SIZE] =
                    BLOCK_DELIMITER.to_be_bytes();
                assert_eq!(
                    packet[(block.position.1 - (BLOCK_DELIMITER_SIZE - 1)) + x],
                    BLOCK_DELIMITER_BYTES[x]
                );
            }
        }
    }

    #[test]
    pub fn check_blank_packet_formation() {
        check_packet_formation(MIN_QPACKET);
        check_packet_formation(MAX_QPACKET);
    }
}
