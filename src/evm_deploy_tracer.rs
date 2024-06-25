use std::collections::HashMap;

use zk_evm::{
    ethereum_types::{Address, H160, H256, U256},
    reference_impls::{
        decommitter::SimpleDecommitter, event_sink::InMemoryEventSink, memory::SimpleMemory,
    },
    testing::storage::InMemoryStorage,
    vm_state::VmState,
    zk_evm_abstractions::precompiles::DefaultPrecompilesProcessor,
    zkevm_opcode_defs::{
        decoding::VmEncodingMode,
        system_params::{
            DEPLOYER_SYSTEM_CONTRACT_ADDRESS, KNOWN_CODE_FACTORY_SYSTEM_CONTRACT_ADDRESS,
        },
        BlobSha256Format, FatPointer, VersionedHashLen32, CALL_IMPLICIT_CALLDATA_FAT_PTR_REGISTER,
    },
};

use crate::{
    publish_evm_bytecode_interface,
    runners::{
        hashmap_based_memory::SimpleHashmapMemory, simple_witness_tracer::MemoryLogWitnessTracer,
    },
};

// In zk_evm@1.5.0 the "deployer address" constant is incorrect and it points to the account code storage.
// so we duplicate those here.
const CONTRACT_DEPLOYER_ADDRESS: Address = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x80, 0x06,
]);

const KNOWN_CODES_STORAGE_ADDRESS: Address = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x80, 0x04,
]);

pub(crate) fn record_deployed_evm_bytecode<const B: bool, const N: usize, E: VmEncodingMode<N>>(
    state: &mut VmState<
        InMemoryStorage,
        SimpleHashmapMemory,
        InMemoryEventSink,
        DefaultPrecompilesProcessor<B>,
        SimpleDecommitter<B>,
        MemoryLogWitnessTracer,
        N,
        E,
    >,
) {
    // We check if ContractDeployer was called with provided evm bytecode.
    // It is assumed that by that time the user has already paid for its size.
    // So even if we do not revert the addition of the this bytecode it is not a ddos vector, since
    // the payment is the same as if the bytecode publication was reverted.

    let current_callstack = &state.local_state.callstack.current;

    // Here we assume that the only case when PC is 0 at the start of the execution of the contract.
    let known_code_storage_call = current_callstack.this_address == KNOWN_CODES_STORAGE_ADDRESS
        && current_callstack.pc == E::PcOrImm::default()
        && current_callstack.msg_sender == CONTRACT_DEPLOYER_ADDRESS;

    if !known_code_storage_call {
        // Leave
        return;
    }

    // Now, we need to check whether it is indeed a call to publish EVM code.
    let calldata_ptr =
        state.local_state.registers[CALL_IMPLICIT_CALLDATA_FAT_PTR_REGISTER as usize];

    let data = read_pointer(&state.memory, FatPointer::from_u256(calldata_ptr.value));

    let contract = publish_evm_bytecode_interface();

    if data.len() < 4 {
        // Not interested
        return;
    }

    let (signature, data) = data.split_at(4);

    if signature
        != contract
            .function("publishEVMBytecode")
            .unwrap()
            .short_signature()
    {
        // Not interested
        return;
    }

    let Ok(call_params) = contract
        .function("publishEVMBytecode")
        .unwrap()
        .decode_input(data)
    else {
        // Not interested
        return;
    };

    let published_bytecode = call_params[0].clone().into_bytes().unwrap();

    let hash = hash_evm_bytecode(&published_bytecode);
    let as_words = bytes_to_be_words(published_bytecode);

    let (_, normalized) = BlobSha256Format::normalize_for_decommitment(hash.as_fixed_bytes());
    if state
        .decommittment_processor
        .get_preimage_by_hash(normalized)
        .is_none()
    {
        state
            .decommittment_processor
            .populate(vec![(h256_to_u256(hash), as_words.clone())]);
    }
}

pub fn h256_to_u256(num: H256) -> U256 {
    U256::from_big_endian(num.as_bytes())
}

pub(crate) fn hash_evm_bytecode(bytecode: &[u8]) -> H256 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let len = bytecode.len() as u16;
    hasher.update(bytecode);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output[..].copy_from_slice(&result.as_slice());
    output[0] = BlobSha256Format::VERSION_BYTE;
    output[1] = 0;
    output[2..4].copy_from_slice(&len.to_be_bytes());

    H256(output)
}

fn bytes_to_be_words(vec: Vec<u8>) -> Vec<U256> {
    assert!(vec.len() % 32 == 0, "Invalid bytecode length");

    vec.chunks(32).map(U256::from_big_endian).collect()
}

/// Reads the memory slice represented by the fat pointer.
/// Note, that the fat pointer must point to the accessible memory (i.e. not cleared up yet).
pub(crate) fn read_pointer(memory: &SimpleHashmapMemory, pointer: FatPointer) -> Vec<u8> {
    let FatPointer {
        offset,
        length,
        start,
        memory_page,
    } = pointer;

    // The actual bounds of the returndata ptr is [start+offset..start+length]
    let mem_region_start = start + offset;
    let mem_region_length = length - offset;

    read_unaligned_bytes(memory, memory_page, mem_region_start, mem_region_length)
}

// This method should be used with relatively small lengths, since
// we don't heavily optimize here for cases with long lengths
pub fn read_unaligned_bytes(
    memory: &SimpleHashmapMemory,
    page: u32,
    start: u32,
    length: u32,
) -> Vec<u8> {
    if length == 0 {
        return vec![];
    }

    let end = start + length - 1;

    let mut current_word = start / 32;
    let mut result = vec![];
    while current_word * 32 <= end {
        let word_value = memory.read_slot(page, current_word).value;
        let word_value = {
            let mut bytes: Vec<u8> = vec![0u8; 32];
            word_value.to_big_endian(&mut bytes);
            bytes
        };

        result.extend(extract_needed_bytes_from_word(
            word_value,
            current_word as usize,
            start as usize,
            end as usize,
        ));

        current_word += 1;
    }

    assert_eq!(result.len(), length as usize);

    result
}

// It is expected that there is some intersection between `[word_number*32..word_number*32+31]` and `[start, end]`
fn extract_needed_bytes_from_word(
    word_value: Vec<u8>,
    word_number: usize,
    start: usize,
    end: usize,
) -> Vec<u8> {
    let word_start = word_number * 32;
    let word_end = word_start + 31; // Note, that at `word_start + 32` a new word already starts

    let intersection_left = std::cmp::max(word_start, start);
    let intersection_right = std::cmp::min(word_end, end);

    if intersection_right < intersection_left {
        vec![]
    } else {
        let start_bytes = intersection_left - word_start;
        let to_take = intersection_right - intersection_left + 1;

        word_value
            .into_iter()
            .skip(start_bytes)
            .take(to_take)
            .collect()
    }
}
