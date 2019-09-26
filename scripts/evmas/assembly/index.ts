@external("env", "eth2_loadPreStateRoot")
export declare function eth2_loadPreStateRoot(offset: u32): void;

@external("env", "eth2_blockDataSize")
export declare function eth2_blockDataSize(): u32;

@external("env", "eth2_blockDataCopy")
export declare function eth2_blockDataCopy(outputOffset: u32, offset: u32, length: u32): void;

@external("env", "eth2_savePostStateRoot")
export declare function eth2_savePostStateRoot(offset: u32): void;

@external("env", "eth2_pushNewDeposit")
export declare function eth2_pushNewDeposit(offseT: u32): void;

@external("env", "eth2_setBignumStack")
export declare function eth2_setBignumStack(startData: u32): void;

@external("env", "eth2_setMemoryPtr")
export declare function eth2_setMemoryPtr(startData: u32): void;

@external("env", "eth2_log")
export declare function eth2_log(value: i32): void;

@external("env", "eth2_add256")
export declare function eth2_add256(stackTop: u32): u32;

@external("env", "eth2_mul256")
export declare function eth2_mul256(stackTop: u32): u32;

@external("env", "eth2_sub256")
export declare function eth2_sub256(stackTop: u32): u32;

@external("env", "eth2_lt256")
export declare function eth2_lt256(stackTop: u32): u32;

@external("env", "eth2_div256")
export declare function eth2_div256(stackTop: u32): u32;

@external("env", "eth2_jumpi")
export declare function eth2_jumpi(stackTop: u32, pc: i32): u32;

export function main(): void {
  // bignum stack size is 100 elements
  // each stack element is 32 bytes
  let BignumStackSize = 100
  let BignumElementSize = 32
  let BignumStack = new ArrayBuffer(BignumElementSize * BignumStackSize)

  let BignumStackPtr = changetype<usize>(BignumStack)
  eth2_setBignumStack(BignumStackPtr)

  let BignumStackElements = Array.create<Uint8Array>(100)

  for (let i = 0; i < BignumStackSize; i++) {
    BignumStackElements[i] = Uint8Array.wrap(BignumStack, i*BignumElementSize, 32)
  }

  let MemorySize = 100
  let MemoryElementSize = 16
  let Memory = new ArrayBuffer(MemoryElementSize * MemorySize)
  let MemoryPtr = changetype<usize>(Memory)
  eth2_setMemoryPtr(MemoryPtr)
  let MemoryElements = Array.create<Uint8Array>(100)

  for (let i = 0; i < MemorySize; i++) {
    MemoryElements[i] = Uint8Array.wrap(Memory, i * MemoryElementSize, 16)
  }

  let BignumStackTop: i32 = 0

  // get evm bytecode size
  let bc_size_ptr: usize = __alloc(4, 0)
  eth2_blockDataCopy(bc_size_ptr, 0, 4)
  let bc_size: i32 = load<i32>(bc_size_ptr)
  
  // get evm bytecode
  let evm_bytecode_ptr: usize = __alloc(bc_size, 0)
  eth2_blockDataCopy(evm_bytecode_ptr, 4, bc_size)

  let evm_bytecode_buff = new ArrayBuffer(bc_size)
  memory.copy((evm_bytecode_buff as usize), evm_bytecode_ptr, bc_size)

  let evm_bytecode = Uint8Array.wrap(evm_bytecode_buff, 0, bc_size)

  // get data size
  let data_size_offset = 4 + bc_size
  let dt_size_ptr: usize = __alloc(4, 0)
  eth2_blockDataCopy(dt_size_ptr, data_size_offset, data_size_offset + 4)
  let dt_size: i32 = load<i32>(dt_size_ptr)
  
  // get data offset
  let data_offset = data_size_offset + 4
  
  const stop: u8 = 0x00
  const add: u8 = 0x01
  const mul: u8 = 0x02
  const sub: u8 = 0x03
  const div: u8 = 0x04
  const lt: u8 = 0x10
  const eq: u8 = 0x14
  const iszero: u8 = 0x15
  const opnot: u8 = 0x19
  const callvalue: u8 = 0x34
  const calldataload: u8 = 0x35
  const calldatasize: u8 = 0x36
  const codecopy: u8 = 0x39
  const pop: u8 = 0x50
  const mload: u8 = 0x51
  const mstore: u8 = 0x52
  const sstore: u8 = 0x55
  const jump: u8 = 0x56
  const jumpi: u8 = 0x57
  const jumpdest: u8 = 0x5b
  const push1: u8 = 0x60
  const push2: u8 = 0x61
  const push3: u8 = 0x62
  const push4: u8 = 0x63
  const push29: u8 = 0x7c
  const dup1: u8 = 0x80
  const dup2: u8 = 0x81
  const dup3: u8 = 0x82
  const swap1: u8 = 0x90
  const swap2: u8 = 0x91
  const swap3: u8 = 0x92
  const opreturn: u8 = 0xf3
  const revert: u8 = 0xfd
  const invalid: u8 = 0xfe

  let pc: i32 = 0
  let preStateRootPtr: usize = __alloc(32, 0)
  eth2_loadPreStateRoot(preStateRootPtr)

  let postStateRootPtr: usize = __alloc(32, 0)
  memory.copy(postStateRootPtr, preStateRootPtr, 32)

  let lastByte: u8 = load<u8>(preStateRootPtr, 31)

  store<u8>(postStateRootPtr, lastByte + 1, 31)

  eth2_savePostStateRoot(postStateRootPtr)

  while (pc < evm_bytecode.length) {
    let opcode: u8 = evm_bytecode[pc]
    pc++

    eth2_log(opcode)
    switch (opcode) {
    case push1: // 0x60
      let push_val = evm_bytecode[pc]
      pc++
      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot.fill(0, 0, 32)

      // 1 byte goes in the last byte of the 32-byte stack slot
      stack_slot[31] = push_val

      BignumStackTop++

      break

    case push2: // 0x61
      let push_val1 = evm_bytecode[pc]
      pc++
      let push_val2 = evm_bytecode[pc]
      pc++
      
      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot.fill(0, 0, 32)
      
      stack_slot[30] = push_val1
      stack_slot[31] = push_val2
      BignumStackTop++
      
      break

    case push4: // 0x63
      let push_val1 = evm_bytecode[pc]
      pc++
      let push_val2 = evm_bytecode[pc]
      pc++
      let push_val3 = evm_bytecode[pc]
      pc++
      let push_val4 = evm_bytecode[pc]
      pc++

      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot.fill(0, 0, 32)

      stack_slot[28] = push_val1
      stack_slot[29] = push_val2
      stack_slot[30] = push_val3
      stack_slot[31] = push_val4
      BignumStackTop++

      break
    case push29: // 0x7c
      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot.fill(0, 0, 32)
      for (let i = 0; i < 29; i++) {
        stack_slot[i+3] = evm_bytecode[pc]
        pc++
      }

      BignumStackTop++
      break
    case add: // 0x01
      BignumStackTop = eth2_add256(BignumStackTop)
      break
    case mul: // 0x02
      BignumStackTop = eth2_mul256(BignumStackTop)
      break
    case sub: // 0x03
      BignumStackTop = eth2_sub256(BignumStackTop)
      break
    case div: // 0x04
      BignumStackTop = eth2_div256(BignumStackTop)
      
      break
      /*
    case sstore: // 0x55
      BignumStackTop = BignumStackTop - 3
      let result_slot = BignumStackElements[BignumStackTop]
      finish(result_slot.dataStart, 32)
      break
      */
    case pop: // 0x50
      BignumStackTop--
      break
      /*
    case mload: // 0x51
      // pop memid
      let memid_slot = BignumStackElements[BignumStackTop - 1]
      let memid = memid_slot[31]

      // get value from memory
      memid = memid / 16 + 1
      let mem_slot = MemoryElements[memid]
      let value = mem_slot[15]

      let stack_slot = BignumStackElements[BignumStackTop - 1]

      stack_slot.fill(0, 0, 32)

      stack_slot[31] = value

      break
      */
    case mstore: // 0x52
      // pop memid
      BignumStackTop--
      let memid_slot = BignumStackElements[BignumStackTop]
      let memid = memid_slot[31]

      // pop memval
      BignumStackTop--

      let memval_slot = BignumStackElements[BignumStackTop]
      let memval = memval_slot[31]

      memid = memid / 16

      let mem_slot1 = MemoryElements[memid]
      let mem_slot2 = MemoryElements[memid + 1]

      // set value in memory
      for(let i = 0; i < 32; i++) {
        if (i > 15) {
          mem_slot2[i-16] = memval_slot[i]
        } else {
          mem_slot1[i] = memval_slot[i]
        }
      }

      break
    case callvalue: // 0x34
      let call_value = 0 

      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot[31] = call_value

      BignumStackTop++
      break
    case calldataload: // 0x35
      // pop position
      BignumStackTop--
      let pos_slot = BignumStackElements[BignumStackTop]
      let pos = pos_slot[31]

      let data_ptr: usize = __alloc(32, 0)
      eth2_blockDataCopy(data_ptr, data_offset + pos, data_offset + pos + 32)

      let data_buff = new ArrayBuffer(32)
      memory.copy((data_buff as usize), data_ptr, 32)
      let data = Uint8Array.wrap(data_buff, 0, 32)

      let stack_slot = BignumStackElements[BignumStackTop]
      for (let i = 0; i < 32; i++) {
        stack_slot[i] = data[i]
      }
      
      BignumStackTop++

      break

    case calldatasize: // 0x36
      let data_size = eth2_blockDataSize()

      let stack_slot = BignumStackElements[BignumStackTop]
      stack_slot[31] = data_size

      BignumStackTop++

      break

      /*
    case codecopy: // 0x39
      break
      */

    case lt:      // 0x10
      BignumStackTop = eth2_lt256(BignumStackTop)
      break

    case eq:     // 0x14
      let a_pos = BignumStackElements[BignumStackTop - 1]
      let b_pos = BignumStackElements[BignumStackTop - 2]

      let diff = false

      for (let i = 0; i < 32; i++) {
        if (a_pos[i] != b_pos[i]) {
          diff = true
          break
        }
      }

      BignumStackTop--
      let res_pos = BignumStackElements[BignumStackTop]
      res_pos.fill(0, 0, 32)

      if (diff)
        res_pos[31] = 1
      break
    case iszero: // 0x15
      let result_pos = BignumStackElements[BignumStackTop]
      let elem_pos = BignumStackElements[BignumStackTop - 1]

      result_pos.fill(0, 0, 32)

      for (let i = 0; i < 32; i++) {
        if (elem_pos[i] != 0) {
          result_pos[31] = 1
          break
        }
      }
      break
      /*
    case opnot: // 0x19
      notFunc()
      break
      */
      /*
    case stop: // 0x00
      pc = code_array.length     // finish execution
      break
      */
      /*
    case jump: // 0x56
      // pop destination
      BignumStackTop--
      let dest_slot = BignumStackElements[BignumStackTop]
      let dest = dest_slot[31]
      pc = dest
      break
      */
    case jumpi: // 0x57
      eth2_log(pc)
      pc = eth2_jumpi(BignumStackTop, pc)
      eth2_log(pc)
      BignumStackTop = BignumStackTop - 2
      break
    case jumpdest: // 0x5b
      break
    case dup1:    // 0x80
      // get value
      let value_slot = BignumStackElements[BignumStackTop - 1]

      // push value
      let dup_slot = BignumStackElements[BignumStackTop]
      for (let i = 0; i < 32; i++) {
        dup_slot[i] = value_slot[i]
      }

      BignumStackTop++
      break
    case dup2:  // 0x81
      // get value
      let value_slot = BignumStackElements[BignumStackTop - 2]

      // push value
      let dup_slot = BignumStackElements[BignumStackTop]
      for (let i = 0; i < 32; i++) {
        dup_slot[i] = value_slot[i]
      }

      BignumStackTop++
      break
      /*
    case dup3:  // 0x82
      // get value
      let value_slot = BignumStackElements[BignumStackTop - 3]

      // push value
      let dup_slot = BignumStackElements[BignumStackTop]
      for (var i= 0; i < 32; i++) {
        dup_slot[i] = value_slot[i]
      }

      BignumStackTop++
      break
      */
    case swap1: // 0x90
      // get stack top
      let top_slot = BignumStackElements[BignumStackTop - 1]

      // get value
      let value = BignumStackElements[BignumStackTop - 2]

      let temp = new Uint8Array(32)

      for (let i = 0; i < 32; i++) {
        temp[i] = value[i]
      }

      for (let i = 0; i < 32; i++) {
        value[i] = top_slot[i]
      }

      for (let i = 0; i < 32; i++) {
        top_slot[i] = temp[i]
      }

      break
      /*
    case swap2: // 0x91
      // get stack top
      let top_slot = BignumStackElements[BignumStackTop - 1]

      // get value
      let value = BignumStackElements[BignumStackTop - 3]

      let temp = new Uint8Array(32)

      for (let i = 0; i < 32; i++) {
        temp[i] = value[i]
      }

      for (let i = 0; i < 32; i++) {
        value[i] = top_slot[i]
      }

      for (let i = 0; i < 32; i++) {
        top_slot[i] = temp[i]
      }
      break
      */
      /*
    case swap3: // 0x92
      // get stack top
      let top_slot = BignumStackElements[BignumStackTop - 1]

      // get value
      let value = BignumStackElements[BignumStackTop - 4]

      // temp
      let temp = new Uint8Array(32)

      for (let i = 0; i < 32; i++) {
        temp[i] = value[i]
      }

      for (let i = 0; i < 32; i++) {
        value[i] = top_slot[i]
      }

      for (let i = 0; i < 32; i++) {
        top_slot[i] = temp[i]
      }

      break
      */
      /*
    case opreturn:  // 0xf3
      // pop offset
      let offset_slot = BignumStackElements[BignumStackTop - 1]
      let offset = offset_slot[31]

      // pop length
      let length_slot = BignumStackElements[BignumStackTop - 2]
      let length = length_slot[31]

      offset = offset / 16

      let mem_slot = MemoryElements[offset]
      finish(mem_slot.dataStart, length)
      pc = code_array.length // finish execution
      break
      */
    case revert: // 0xfd
      pc = evm_bytecode.length      // finish execution
      break
      /*
    case invalid:
      pc = code_array.length     // finish execution
      break
      */
    default:
      pc = evm_bytecode.length  // unknown opcode, finish execution
      eth2_log(31337)
      break
    }
  }
}

