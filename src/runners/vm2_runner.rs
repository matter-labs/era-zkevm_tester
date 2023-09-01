use vm2::{decode::decode_program, State};
use zk_evm::contract_bytecode_to_words;
use zkevm_assembly::Assembly;

pub struct Vm2;

impl super::compiler_tests::TestableVM for Vm2 {
    fn run(
        entry_address: zk_evm::ethereum_types::Address,
        calldata: Option<Vec<zk_evm::ethereum_types::U256>>,
        context: super::compiler_tests::VmExecutionContext,
        initial_pc: usize,
        r2_to_r5: [zk_evm::ethereum_types::U256; 4],
        contracts: std::collections::HashMap<
            zk_evm::ethereum_types::Address,
            zkevm_assembly::Assembly,
        >,
        factory_deps: std::collections::HashMap<zk_evm::ethereum_types::U256, Assembly>,
        storage: std::collections::HashMap<
            super::compiler_tests::StorageKey,
            zk_evm::ethereum_types::H256,
        >,
        block_properties: zk_evm::block_properties::BlockProperties,
        cycles_limit: usize,
    ) -> anyhow::Result<super::compiler_tests::VmSnapshot> {
        let bytecode = contracts[&entry_address]
            .clone()
            .compile_to_bytecode()
            .unwrap();
        let instructions = bytecode
            .iter()
            .flat_map(|word| {
                [
                    u64::from_be_bytes(word[0..8].try_into().unwrap()),
                    u64::from_be_bytes(word[8..16].try_into().unwrap()),
                    u64::from_be_bytes(word[16..24].try_into().unwrap()),
                    u64::from_be_bytes(word[24..32].try_into().unwrap()),
                ]
            })
            .collect::<Vec<_>>();
        let program = decode_program(&instructions);

        let mut state = State::default();
        for (reg, value) in state.registers[2..].iter_mut().zip(r2_to_r5) {
            *reg = value;
        }
        state.code_page = contract_bytecode_to_words(&bytecode);

        state.run(&program);

        todo!()
    }
}
