use core::mem;

use crate::parameters::*;

const TOTAL_DATA_BLOCKS: usize = 5;
const FEC_BYTES: usize = 19;

type BlockLabelType = u8;
const FIRST_BLOCK_LABEL: BlockLabelType = 128;

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

pub struct BlockCfg {
    block_type: BlockType,
    do_transmit_label: bool,
}

type BlockCfgStack = [BlockCfg; TOTAL_DATA_BLOCKS];

#[derive(Clone, Copy, PartialEq, Eq)]
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

    pub const fn data_len(&self) -> usize {
        self.block_type.len()
    }

    pub const fn total_len(&self) -> usize {
        debug_assert!(self.position.1 >= self.position.0);
        self.position.1 - self.position.0
    }

    pub const fn len(&self) -> usize {
        self.total_len()
    }
}

type BlockIdentStack = [BlockIdent; TOTAL_DATA_BLOCKS];

const fn cfg_stack_to_ident_stack(cfg_stack: BlockCfgStack) -> BlockIdentStack {
    // a block is its label, its data, and the following delimiter
    let mut output: BlockIdentStack = [BlockIdent::BLANK; TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;
    let mut current_packet_position: usize = 1usize; // bytes

    while i < output.len() {
        if i > BlockLabelType::MAX as usize || i >= cfg_stack.len() {
            unreachable!();
        }

        let _block_type = cfg_stack[i].block_type;
        let _do_transmit_label = cfg_stack[i].do_transmit_label;
        let _block_label: BlockLabelType = FIRST_BLOCK_LABEL + i as BlockLabelType;
        let _block_size: usize = _block_type.len() + {if _do_transmit_label {mem::size_of::<BlockLabelType>()} else {0}} + BLOCK_DELIMITER_SIZE;
        let end_position = current_packet_position + _block_size;
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

pub const BLOCK_IDENT_STACK: BlockIdentStack = cfg_stack_to_ident_stack(BLOCK_CFG_STACK);
pub const QPACKET_DATA_LEN: usize = BLOCK_IDENT_STACK[BLOCK_IDENT_STACK.len() - 1].position.1;
const START_END_HEADER_SIZE: usize = mem::size_of_val::<_>(&START_END_HEADER);
pub const QPAKCET_BARE_LEN: usize = START_HEADER_DATA.len() + QPACKET_DATA_LEN + START_END_HEADER_SIZE;
pub const QPACKET_FULL_LEN: usize = QPAKCET_BARE_LEN + FEC_BYTES;
pub const BLOCK_SIZE_STACK: [usize; TOTAL_DATA_BLOCKS] = {
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

pub const fn construct_bare_packet(_blockstack: QPacketBlockStack) -> [u8; BARE_MESSAGE_LENGTH_BYTES] {
    
    todo!()
}
