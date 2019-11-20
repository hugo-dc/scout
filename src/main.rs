extern crate rustc_hex;
extern crate wasmi;

use rustc_hex::FromHex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use wasmi::memory_units::Pages;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryInstance,
    MemoryRef, Module, ModuleImportResolver, ModuleInstance, NopExternals, RuntimeArgs,
    RuntimeValue, Signature, Trap, TrapKind, ValueType,
};

mod types;
use crate::types::*;

use ethereum_types::U256;

const LOADPRESTATEROOT_FUNC_INDEX: usize = 0;
const BLOCKDATASIZE_FUNC_INDEX: usize = 1;
const BLOCKDATACOPY_FUNC_INDEX: usize = 2;
const SAVEPOSTSTATEROOT_FUNC_INDEX: usize = 3;
const PUSHNEWDEPOSIT_FUNC_INDEX: usize = 4;
const USETICKS_FUNC_INDEX: usize = 5;
const SETBIGNUMSTACK_FUNC_INDEX: usize = 6;
const SETMEMPTR_FUNC_INDEX: usize = 7;
const ADD256_FUNC_INDEX: usize = 8;
const MUL256_FUNC_INDEX: usize = 9;
const SUB256_FUNC_INDEX: usize = 10;
const LT256_FUNC_INDEX: usize = 11;
const DIV256_FUNC_INDEX: usize = 12;
const JUMPI_FUNC_INDEX: usize = 13;
const LOG_FUNC_INDEX: usize = 14;
const PRINTMEM_FUNC_INDEX: usize = 15;

static mut BignumStackOffset: u32 = 0;     // EVM
static mut EVMMemoryStartOffset: u32 = 0;  // EVM

fn get_opcode(value: u32) -> String {
    match value {
        0x00 => String::from("STOP"),
        0x01 => String::from("ADD"),
        0x02 => String::from("MUL"),
        0x03 => String::from("SUB"),
        0x04 => String::from("DIV"),
        0x10 => String::from("LT"),
        0x14 => String::from("EQ"),
        0x15 => String::from("ISZERO"),
        0x19 => String::from("NOT"),
        0x34 => String::from("CALLVALUE"),
        0x35 => String::from("CALLDATALOAD"),
        0x36 => String::from("CALLDATASIZE"),
        0x39 => String::from("CODECOPY"),
        0x50 => String::from("POP"),
        0x51 => String::from("MLOAD"),
        0x52 => String::from("MSTORE"),
        0x55 => String::from("SSTORE"),
        0x56 => String::from("JUMP"),
        0x57 => String::from("JUMPI"),
        0x5b => String::from("JUMPDEST"),
        0x60 => String::from("PUSH1"),
        0x61 => String::from("PUSH2"),
        0x62 => String::from("PUSH3"),
        0x63 => String::from("PUSH4"),
        0x7c => String::from("PUSH29"),
        0x80 => String::from("DUP1"),
        0x81 => String::from("DUP2"),
        0x82 => String::from("DUP3"),
        0x90 => String::from("SWAP1"),
        0x91 => String::from("SWAP2"),
        0x92 => String::from("SWAP3"),
        0xf3 => String::from("RETURN"),
        0xfd => String::from("REVERT"),
        0xfe => String::from("INVALID"),
        _ => String::from("UNK"),
    }
}

struct Runtime<'a> {
    ticks_left: u32,
    memory: Option<MemoryRef>,
    pre_state: &'a Bytes32,
    block_data: &'a ShardBlockBody,
    post_state: Bytes32,
}

impl<'a> Runtime<'a> {
    fn new(
        pre_state: &'a Bytes32,
        block_data: &'a ShardBlockBody,
        memory: Option<MemoryRef>,
    ) -> Runtime<'a> {
        Runtime {
            ticks_left: 10_000_000, // FIXME: make this configurable
            memory: if memory.is_some() {
                memory
            } else {
                // Allocate a single page if no memory was exported.
                Some(MemoryInstance::alloc(Pages(1), Some(Pages(1))).unwrap())
            },
            pre_state: pre_state,
            block_data: block_data,
            post_state: Bytes32::default(),
        }
    }

    fn get_post_state(&self) -> Bytes32 {
        self.post_state
    }
}

impl<'a> Externals for Runtime<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            USETICKS_FUNC_INDEX => {
                let ticks: u32 = args.nth(0);
                if self.ticks_left < ticks {
                    // FIXME: use TrapKind::Host
                    return Err(Trap::new(TrapKind::Unreachable));
                }
                self.ticks_left -= ticks;
                Ok(None)
            }
            LOADPRESTATEROOT_FUNC_INDEX => {
                let ptr: u32 = args.nth(0);
                println!("loadprestateroot to {}", ptr);

                // TODO: add checks for out of bounds access
                let memory = self.memory.as_ref().expect("expects memory object");
                memory
                    .set(ptr, &self.pre_state.bytes)
                    .expect("expects writing to memory to succeed");

                Ok(None)
            }
            SAVEPOSTSTATEROOT_FUNC_INDEX => {
                let ptr: u32 = args.nth(0);
                println!("savepoststateroot from {}", ptr);

                // TODO: add checks for out of bounds access
                let memory = self.memory.as_ref().expect("expects memory object");
                memory
                    .get_into(ptr, &mut self.post_state.bytes)
                    .expect("expects reading from memory to succeed");

                Ok(None)
            }
            BLOCKDATASIZE_FUNC_INDEX => {
                let ret: i32 = self.block_data.data.len() as i32;
                println!("blockdatasize {}", ret);
                Ok(Some(ret.into()))
            }
            BLOCKDATACOPY_FUNC_INDEX => {
                let ptr: u32 = args.nth(0);
                let offset: u32 = args.nth(1);
                let length: u32 = args.nth(2);
                println!(
                    "blockdatacopy to {} from {} for {} bytes",
                    ptr, offset, length
                );

                // TODO: add overflow check
                let offset = offset as usize;
                let length = length as usize;

                // TODO: add checks for out of bounds access
                let memory = self.memory.as_ref().expect("expects memory object");

                println!("offset: {}, length: {}", offset, length);
                
                memory
                    .set(ptr, &self.block_data.data[offset..length])
                    .expect("expects writing to memory to succeed");


                Ok(None)
            }
            PUSHNEWDEPOSIT_FUNC_INDEX => unimplemented!(),
            SETBIGNUMSTACK_FUNC_INDEX => {
                let startData: u32 = args.nth(0);
                unsafe {
                    BignumStackOffset = startData;
                }

                Ok(None)
            }
            SETMEMPTR_FUNC_INDEX => {
                let startData: u32 = args.nth(0);

                unsafe {
                    EVMMemoryStartOffset = startData;
                }

                Ok(None)
            }
            ADD256_FUNC_INDEX => {
                let memory = self.memory.as_ref().expect("expects memory object");
                
                let a_pos = args.nth(0);
                let b_pos = args.nth(1);
                let result_pos = args.nth(2);

                let mut bytes_a: [u8; 32] = [0;32];
                let mut bytes_b: [u8; 32] = [0;32];

                memory
                    .get_into(a_pos, &mut bytes_a)
                    .expect("expects reading from memory to succeed");

                memory
                    .get_into(b_pos, &mut bytes_b)
                    .expect("expects reading from memory to succeed");

                let elem_a = U256::from(bytes_a);
                let elem_b = U256::from(bytes_b);

                let (result, ov) = elem_a.overflowing_add(elem_b);

                let mut bytes_result: [u8; 32] = [0;32];

                result.to_big_endian(&mut bytes_result);

                memory
                    .set(result_pos, &bytes_result)
                    .expect("expects writing to memory to succeed");
                Ok(None)
            }
            MUL256_FUNC_INDEX => {
                let mut BignumStackTop: u32 = args.nth(0);
                let memory = self.memory.as_ref().expect("expects memory object");
                let mut a_pos: u32 = 0;
                let mut b_pos: u32 = 0;

                unsafe {
                    a_pos = BignumStackOffset + 32 * (BignumStackTop - 1);
                    b_pos = BignumStackOffset + 32 * (BignumStackTop - 2);
                }

                let mut bytes_a: [u8; 32] = [0; 32];
                let mut bytes_b: [u8; 32] = [0; 32];

                memory
                    .get_into(a_pos, &mut bytes_a)
                    .expect("expects reading from memory to succeed");
                memory
                    .get_into(b_pos, &mut bytes_b)
                    .expect("expects reading from memory to succeed");

                let elem_a = U256::from(bytes_a);
                let elem_b = U256::from(bytes_b);

                let (result, ov) =  elem_a.overflowing_mul(elem_b);

                let mut bytes_result: [u8; 32] = [0; 32];
                result.to_big_endian(&mut bytes_result);

                memory
                    .set(b_pos, &bytes_result)
                    .expect("expects writing to memory to succeed");

                BignumStackTop = BignumStackTop - 1;
                Ok(Some(BignumStackTop.into()))
            }
            SUB256_FUNC_INDEX => {
                let memory = self.memory.as_ref().expect("expects memory object");
                
                let a_pos = args.nth(0);
                let b_pos = args.nth(1);
                let result_pos = args.nth(2);

                let mut bytes_a: [u8; 32] = [0;32];
                let mut bytes_b: [u8; 32] = [0;32];

                memory
                    .get_into(a_pos, &mut bytes_a)
                    .expect("expects reading from memory to succeed");

                memory
                    .get_into(b_pos, &mut bytes_b)
                    .expect("expects reading from memory to succeed");

                let elem_a = U256::from(bytes_a);
                let elem_b = U256::from(bytes_b);

                let (result, ov) = elem_a.overflowing_sub(elem_b);

                let mut bytes_result: [u8; 32] = [0;32];

                result.to_big_endian(&mut bytes_result);

                memory
                    .set(result_pos, &bytes_result)
                    .expect("expects writing to memory to succeed");
                Ok(None)
            }
            LT256_FUNC_INDEX => {
                let mut BignumStackTop: u32 = args.nth(0);
                let memory = self.memory.as_ref().expect("expects memory object");
                let mut a_pos: u32 = 0;
                let mut b_pos: u32 = 0;

                unsafe {
                    a_pos = BignumStackOffset + 32 * (BignumStackTop - 1);
                    b_pos = BignumStackOffset + 32 * (BignumStackTop - 2);
                }

                let mut bytes_a: [u8; 32] = [0; 32];
                let mut bytes_b: [u8; 32] = [0; 32];

                memory
                    .get_into(a_pos, &mut bytes_a)
                    .expect("expects reading from memory to succeed");
                memory
                    .get_into(b_pos, &mut bytes_b)
                    .expect("expects reading from memory to succeed");

                let elem_a = U256::from(bytes_a);
                let elem_b = U256::from(bytes_b);

                let mut result = U256::from(0);
                if elem_a < elem_b {
                    result = U256::from(1);
                }

                let mut bytes_result: [u8; 32] = [0; 32];
                result.to_big_endian(&mut bytes_result);

                memory
                    .set(b_pos, &bytes_result)
                    .expect("expects writing to memory to succeed");

                BignumStackTop = BignumStackTop - 1;

                Ok(Some(BignumStackTop.into()))
            }
            DIV256_FUNC_INDEX => {
                let mut BignumStackTop: u32 = args.nth(0);
                let memory = self.memory.as_ref().expect("expects memory object");

                let mut a_pos: u32 = 0;
                let mut b_pos: u32 = 0;

                unsafe {
                    a_pos = BignumStackOffset + 32 * (BignumStackTop - 1);
                    b_pos = BignumStackOffset + 32 * (BignumStackTop - 2);
                }

                let mut bytes_a: [u8; 32] = [0; 32];
                let mut bytes_b: [u8; 32] = [0; 32];

                memory
                    .get_into(a_pos, &mut bytes_a)
                    .expect("expects reading from memory to succeed");
                memory
                    .get_into(b_pos, &mut bytes_b)
                    .expect("expects reading from memory to succeed");

                let elem_a = U256::from(bytes_a);
                let elem_b = U256::from(bytes_b);

                let result = elem_a.checked_div(elem_b).unwrap();

                let mut bytes_result: [u8; 32] = [0; 32];
                result.to_big_endian(&mut bytes_result);

                memory
                    .set(b_pos, &bytes_result)
                    .expect("expects writing to memory to succeed");

                BignumStackTop = BignumStackTop - 1;
                Ok(Some(BignumStackTop.into()))
            }
            JUMPI_FUNC_INDEX => {
                let bn_stack_top: u32 = args.nth(0);
                let pc: i32 = args.nth(1);


                let memory = self.memory.as_ref().expect("expects memory object");

                let mut cond_pos: u32 = 0;
                let mut dest_pos: u32 = 0;

                unsafe {
                    dest_pos = BignumStackOffset + 32 * (bn_stack_top - 1);
                    cond_pos = BignumStackOffset + 32 * (bn_stack_top - 2);
                }

                let mut bytes_cond: [u8; 32] = [0; 32];
                let mut bytes_dest: [u8; 32] = [0; 32];

                memory
                    .get_into(cond_pos, &mut bytes_cond)
                    .expect("expects reading from memory to succeed");
                memory
                    .get_into(dest_pos, &mut bytes_dest)
                    .expect("expects reading from memory to succeed");

                let cond = U256::from(bytes_cond);

                if cond != U256::from(0) {
                    let mut dest_arr: [u8; 4] = Default::default();
                    dest_arr.copy_from_slice(&bytes_dest[28..32]);
                    let dest = i32::from_be_bytes(dest_arr);
                    Ok(Some(dest.into()))
                } else {
                    Ok(Some(pc.into()))
                }
            }
            LOG_FUNC_INDEX => {
                let value: u32 = args.nth(0);
                let BignumStackTop: u32 = args.nth(1);
                let memory = self.memory.as_ref().expect("expects memory object");
                let opcode = get_opcode(value);
                println!(">>> {} - {}", value, opcode);

                let mut i = 0;
                while (i < BignumStackTop) {
                    let mut elem_pos: u32 = 0;

                    unsafe {
                        elem_pos = BignumStackOffset + 32 * i;
                    }

                    let mut elem_bytes: [u8; 32] = [0; 32];

                    memory
                        .get_into(elem_pos, &mut elem_bytes)
                        .expect("expects reading from memory to succeed");

                    println!(">>> {:?}", elem_bytes);
                    i = i + 1;
                }
                Ok(None)
            }
            PRINTMEM_FUNC_INDEX => {
                let max = args.nth(0);
                let memory = self.memory.as_ref().expect("expects memory object");
                let mut i = 0;

                println!(">>> ==============================");

                while i < max {
                    let mut elem_pos: u32 = 0;
                    unsafe {
                        elem_pos = EVMMemoryStartOffset + 16 * i;    
                    }
                    let mut elem_bytes: [u8; 16] = [0; 16];

                    memory
                        .get_into(elem_pos, &mut elem_bytes)
                        .expect("expects reading from memory to succeed");

                    println!(">>> {:?}", elem_bytes);
                    i = i + 1;
                }
                println!(">>> --------------------------------");
                Ok(None)
            }
            _ => panic!("unknown function index"),
        }
    }
}

struct RuntimeModuleImportResolver;

impl<'a> ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "eth2_useTicks" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                USETICKS_FUNC_INDEX,
            ),
            "eth2_loadPreStateRoot" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                LOADPRESTATEROOT_FUNC_INDEX,
            ),
            "eth2_blockDataSize" => FuncInstance::alloc_host(
                Signature::new(&[][..], Some(ValueType::I32)),
                BLOCKDATASIZE_FUNC_INDEX,
            ),
            "eth2_blockDataCopy" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32, ValueType::I32][..], None),
                BLOCKDATACOPY_FUNC_INDEX,
            ),
            "eth2_savePostStateRoot" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                SAVEPOSTSTATEROOT_FUNC_INDEX,
            ),
            "eth2_pushNewDeposit" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                PUSHNEWDEPOSIT_FUNC_INDEX,
            ),
            "eth2_setBignumStack" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                SETBIGNUMSTACK_FUNC_INDEX,
            ),
            "eth2_setMemoryPtr" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                SETMEMPTR_FUNC_INDEX,
            ),
            "eth2_add256" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32, ValueType::I32][..], None),
                ADD256_FUNC_INDEX,
            ),
            "eth2_mul256" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                MUL256_FUNC_INDEX,
            ),
            "eth2_sub256" => FuncInstance::alloc_host(
                //Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                // a, b, result
                Signature::new(&[ValueType::I32, ValueType::I32, ValueType::I32][..], None),
                SUB256_FUNC_INDEX,
            ),
            "eth2_lt256" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                LT256_FUNC_INDEX,
            ),
            "eth2_div256" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                DIV256_FUNC_INDEX,
            ),
            "eth2_jumpi" => FuncInstance::alloc_host(
                //Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                JUMPI_FUNC_INDEX,
            ),
            "eth2_log" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                LOG_FUNC_INDEX,
            ),
            "eth2_printMem" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                PRINTMEM_FUNC_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )))
            }
        };
        Ok(func_ref)
    }
}

const BYTES_PER_SHARD_BLOCK_BODY: usize = 16384;
const ZERO_HASH: Bytes32 = Bytes32 { bytes: [0u8; 32] };

/// These are Phase 0 structures.
/// https://github.com/ethereum/eth2.0-specs/blob/dev/specs/core/0_beacon-chain.md
#[derive(Default, PartialEq, Clone, Debug)]
pub struct Deposit {}

/// These are Phase 2 Proposal 2 structures.

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ExecutionScript {
    code: Vec<u8>,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct BeaconState {
    execution_scripts: Vec<ExecutionScript>,
}

/// Shards are Phase 1 structures.
/// https://github.com/ethereum/eth2.0-specs/blob/dev/specs/core/1_shard-data-chains.md

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ShardBlockHeader {}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ShardBlockBody {
    data: Vec<u8>,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ShardBlock {
    env: u64, // This is added by Phase 2 Proposal 2
    data: ShardBlockBody,
    // TODO: add missing fields
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ShardState {
    exec_env_states: Vec<Bytes32>,
    slot: u64,
    parent_block: ShardBlockHeader,
    // TODO: add missing field
    // latest_state_roots: [bytes32, LATEST_STATE_ROOTS_LEMGTH]
}

pub fn execute_code(
    code: &[u8],
    pre_state: &Bytes32,
    block_data: &ShardBlockBody,
) -> (Bytes32, Vec<Deposit>) {
    println!(
        "Executing codesize({}) and data: {:#?}",
        code.len(),
        block_data
    );

    let module = Module::from_buffer(&code).expect("Module loading to succeed");
    let mut imports = ImportsBuilder::new();
    // FIXME: use eth2
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    let instance = ModuleInstance::new(&module, &imports)
        .expect("Module instantation expected to succeed")
        .assert_no_start();

    let internal_mem = instance
        .export_by_name("memory")
        .expect("Module expected to have 'memory' export")
        .as_memory()
        .cloned()
        .expect("'memory' export should be a memory");

    let mut runtime = Runtime::new(pre_state, block_data, Some(internal_mem));

    let result = instance
        .invoke_export("main", &[], &mut runtime)
        .expect("Executed 'main'");

    println!("Result: {:?}", result);
    println!("Execution finished");

    (runtime.get_post_state(), vec![Deposit {}])
}

pub fn process_shard_block(
    state: &mut ShardState,
    beacon_state: &BeaconState,
    block: Option<ShardBlock>,
) {
    // println!("Beacon state: {:#?}", beacon_state);
    println!("Executing block: {:#?}", block);

    println!("Pre-execution: {:#?}", state);

    // TODO: implement state root handling

    if let Some(block) = block {
        // The execution environment identifier
        let env = block.env as usize; // FIXME: usize can be 32-bit
        let code = &beacon_state.execution_scripts[env].code;

        // Set post states to empty for any holes
        // for x in 0..env {
        //     state.exec_env_states.push(ZERO_HASH)
        // }
        let pre_state = &state.exec_env_states[env];
        let (post_state, deposits) = execute_code(code, pre_state, &block.data);
        state.exec_env_states[env] = post_state
    }

    // TODO: implement state + deposit root handling

    println!("Post-execution: {:#?}", state)
}

fn load_file(filename: &str) -> Vec<u8> {
    use std::io::prelude::*;
    let mut file = File::open(filename).expect("loading file failed");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("reading file failed");
    buf
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestBeaconState {
    execution_scripts: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestShardBlock {
    env: u64,
    data: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestShardState {
    exec_env_states: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestFile {
    beacon_state: TestBeaconState,
    shard_blocks: Vec<TestShardBlock>,
    shard_pre_state: TestShardState,
    shard_post_state: TestShardState,
}

impl From<TestBeaconState> for BeaconState {
    fn from(input: TestBeaconState) -> Self {
        BeaconState {
            execution_scripts: input
                .execution_scripts
                .iter()
                .map(|x| ExecutionScript { code: load_file(x) })
                .collect(),
        }
    }
}

impl From<TestShardBlock> for ShardBlock {
    fn from(input: TestShardBlock) -> Self {
        ShardBlock {
            env: input.env,
            data: ShardBlockBody {
                data: input.data.from_hex().expect("invalid hex data"),
            },
        }
    }
}

impl From<TestShardState> for ShardState {
    fn from(input: TestShardState) -> Self {
        ShardState {
            exec_env_states: input
                .exec_env_states
                .iter()
                .map(|x| {
                    let state = x.from_hex().expect("invalid hex data");
                    assert!(state.len() == 32);
                    let mut ret = Bytes32::default();
                    ret.bytes.copy_from_slice(&state[..]);
                    ret
                })
                .collect(),
            slot: 0,
            parent_block: ShardBlockHeader {},
        }
    }
}

fn process_yaml_test(filename: &str) {
    println!("Process yaml!");
    let content = load_file(&filename);
    let test_file: TestFile =
        serde_yaml::from_slice::<TestFile>(&content[..]).expect("expected valid yaml");
    println!("{:#?}", test_file);

    let beacon_state: BeaconState = test_file.beacon_state.into();
    let pre_state: ShardState = test_file.shard_pre_state.into();
    let post_state: ShardState = test_file.shard_post_state.into();

    let mut shard_state = pre_state;
    for block in test_file.shard_blocks {
        process_shard_block(&mut shard_state, &beacon_state, Some(block.into()))
    }
    println!("{:#?}", shard_state);
    assert_eq!(shard_state, post_state);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    process_yaml_test(if args.len() != 2 {
        "test.yaml"
    } else {
        &args[1]
    });
}
