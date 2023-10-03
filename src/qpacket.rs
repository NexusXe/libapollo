use core::mem;

use crate::parameters::*;

const TOTAL_DATA_BLOCKS: usize = 6;
const FEC_BYTES: usize = 16;

type BlockLabelType = u8;
const FIRST_BLOCK_LABEL: BlockLabelType = 128;

const BLOCK_CFG_STACK: BlockCfgStack = [
    BlockCfg {
        block_type: BlockType::CALLSIGN,
        do_transmit_label: false,
    },
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

#[derive(Clone, Copy)]
pub struct BlockIdent {
    block_type: BlockType,
    label: BlockLabelType,
    position: (usize, usize),
}

impl BlockIdent {
    pub const fn new() -> Self {
        Self {
            block_type: BlockType::NONE,
            label: 0 as BlockLabelType,
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
    let mut output: BlockIdentStack = [BlockIdent::new(); TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;
    let mut current_packet_position: usize = 1usize; // bytes

    while i < output.len() {
        if i > BlockLabelType::MAX as usize || i >= cfg_stack.len() {
            unreachable!();
        }

        let _block_type = cfg_stack[i].block_type;
        let _do_transmit_label = cfg_stack[i].do_transmit_label;
        let _block_label: BlockLabelType = FIRST_BLOCK_LABEL + i as BlockLabelType;
        let end_position = current_packet_position
            + _block_type.len()
            + {
                if _do_transmit_label {
                    mem::size_of_val::<_>(&_block_label)
                } else {
                    0
                }
            }
            + BLOCK_DELIMITER_SIZE;
        output[i] = BlockIdent {
            block_type: cfg_stack[i].block_type,
            label: _block_label,
            position: (current_packet_position, end_position),
        };
        current_packet_position = end_position + 1;
        i += 1;
    }
    output
}

pub const BLOCK_IDENT_STACK: BlockIdentStack = cfg_stack_to_ident_stack(BLOCK_CFG_STACK);
pub const QPACKET_DATA_LEN: usize = BLOCK_IDENT_STACK[BLOCK_IDENT_STACK.len() - 1].position.1;

struct QPacketBlock<'a> {
    identity: BlockIdent,
    data: &'a [u8],
}
