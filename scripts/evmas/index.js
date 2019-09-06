const fs = require('fs');
const loader = require('assemblyscript/lib/loader')
const BN = require('bn.js')

const TWO_POW256 = new BN('10000000000000000000000000000000000000000000000000000000000000000', 16)

let BignumStackTop = 0;
let Memory;

let BignumStackStartOffset;
let EVMMemoryStartOffset;


// calldata: 0x26bceb59802431afcbce1fc194c9eaa417b2fb67dc75a95db0bc7ec6b1c8af11df6a1da9a1f5aac137876480252e5dcac62c354ec0d42b76b0642b6181ed099849ea1d57
let calldata = new Uint8Array([38, 188, 235, 89, 128, 36, 49, 175, 203, 206, 31, 193, 148, 201, 234, 164, 23, 178, 251, 103, 220, 117, 169, 93, 176, 188, 126, 198, 177, 200, 175, 17, 223, 106, 29, 169, 161, 245, 170, 193, 55, 135, 100, 128, 37, 46, 93, 202, 198, 44, 53, 78, 192, 212, 43, 118, 176, 100, 43, 97, 129, 237, 9, 152, 73, 234, 29, 87])

function arrayToBn(arr) {
  var hexstr = []
  arr.forEach((i) => {
    var h = i.toString(16)
    if (h.length % 2)
      h = '0' + h
    hexstr.push(h)
  })
  return new BN(hexstr.join(''), 16)
}

const obj = loader.instantiateBuffer(fs.readFileSync(__dirname + '/build/optimized.wasm'), {
  main: {
    setBignumStack(startData, len) {
      BignumStackStartOffset = startData;
    },
    setMemoryPtr(startData, len) {
      EVMMemoryStartOffset = startData
    },
    add256() {
      let stack_elem_a_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 1)
      let stack_elem_b_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 2)
      const arrayA = new Uint8Array(Memory.buffer, stack_elem_a_pos, 32)
      const arrayB = new Uint8Array(Memory.buffer, stack_elem_b_pos, 32)

      const elemA = arrayToBn(arrayA)
      const elemB = arrayToBn(arrayB)

      const result = elemA.add(elemB)
      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)

      let outOffset = stack_elem_b_pos
      // pop 2 push 1, top is reduced by 1
      BignumStackTop.value = BignumStackTop.value - 1
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)
    },
    mul256() {
      let a_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 1)
      let b_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 2)
      const arrA = new Uint8Array(Memory.buffer, a_pos, 32)
      const arrB = new Uint8Array(Memory.buffer, b_pos, 32)

      const elemA = arrayToBn(arrA)
      const elemB = arrayToBn(arrB)

      const result = elemA.mul(elemB).mod(TWO_POW256)
      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)

      let outOffset = b_pos
      BignumStackTop.value = BignumStackTop.value - 1
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)
    },
    sub256() {
      let a_pos = BignumStackStartOffset + 32 *(BignumStackTop.value - 1)
      let b_pos = BignumStackStartOffset + 32 *(BignumStackTop.value - 2)
      const arrA = new Uint8Array(Memory.buffer, a_pos, 32)
      const arrB = new Uint8Array(Memory.buffer, b_pos, 32)

      const elemA = arrayToBn(arrA)
      const elemB = arrayToBn(arrB)

      const result = elemA.sub(elemB)
      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)

      let outOffset = b_pos
      BignumStackTop.value = BignumStackTop.value - 1

      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)

    }, 
    div256() {
      let a_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 1)
      let b_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 2)
      const arrA = new Uint8Array(Memory.buffer, a_pos, 32)
      const arrB = new Uint8Array(Memory.buffer, b_pos, 32)

      const elemA = arrayToBn(arrA)
      const elemB = arrayToBn(arrB)

      const result = elemA.div(elemB)

      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)

      let outOffset = b_pos
      BignumStackTop.value = BignumStackTop.value - 1
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)

    },
    lt() {
      let stack_elem_a_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 1)
      let stack_elem_b_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 2)
      
      const arrayA = new Uint8Array(Memory.buffer, stack_elem_a_pos, 32)
      const arrayB = new Uint8Array(Memory.buffer, stack_elem_b_pos, 32)

      const elemA = arrayToBn(arrayA)
      const elemB = arrayToBn(arrayB)

      let result = new BN(0)
      if (elemA.lt(elemB)) {
        result = new BN(1)
      } 

      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)

      let outOffset = stack_elem_b_pos
      // pop 2 push 1, top is reduced by 1
      BignumStackTop.value = BignumStackTop.value - 1
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)
    },
    eq() {
      let a_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 1)
      let b_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 2)

      const arrA = new Uint8Array(Memory.buffer, a_pos, 32)
      const arrB = new Uint8Array(Memory.buffer, b_pos, 32)

      const elemA = arrayToBn(arrA)
      const elemB = arrayToBn(arrB)

      let result = new BN(0)
      if (elemA.eq(elemB)) {
        result = new BN(1)
      }

      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)
      let outOffset = b_pos
      BignumStackTop.value = BignumStackTop.value - 1
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)
    },
    isZero() {
      let stack_elem_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 1)
      const arrValue = new Uint8Array(Memory.buffer, stack_elem_pos, 32)
      const value = arrayToBn(arrValue)

      let result = new BN(0)
      if (value.eq(new BN(0))) {
        result = new BN(1)
      }

      const resultBytes = result.toArrayLike(Uint8Array, 'be', 32)
      let outOffset = stack_elem_pos
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(resultBytes)
      
    },
    not256() {
      let elem_pos = BignumStackStartOffset + 32 * (BignumStackTop.value - 1)
      const arrValue = new Uint8Array(Memory.buffer, elem_pos, 32)
      const result = new Uint8Array(32)

      for (var i=0; i < 32; i++)
        result[i] = ~ arrValue[i]

      let outOffset = elem_pos
      const outputBytes = new Uint8Array(Memory.buffer, outOffset, 32)
      outputBytes.set(result)
    },
    callvalue() {
      return 0
    },
    calldataload(offset) {
      let stack_elem_pos = BignumStackStartOffset + (32 * BignumStackTop.value)
      const elem = new Uint8Array(Memory.buffer, stack_elem_pos, 32)

      for (let i = offset; i < offset+32 ; i++) {
        elem[i - offset] = calldata[i]
      }

      BignumStackTop.value = BignumStackTop.value + 1
    },
    calldatasize() {
      return calldata.length
    },
    finish(returnOffset, len) {
      const returnVal = new Uint8Array(Memory.buffer, returnOffset, len)
      let returnHex = ''
      for (var i = 0; i < len; i++) {
        if (returnVal[i] < 16) returnHex += '0'
            returnHex += returnVal[i].toString(16)
      }
      console.log(`Return Data: 0x${returnHex}`)
    },
    calculatePC(pc) {
      let cond_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 2)
      const cond_array = new Uint8Array(Memory.buffer, cond_pos, 32)
      const cond = arrayToBn(cond_array)
      
      if (cond != 0) {
        let dest_pos = BignumStackStartOffset + 32*(BignumStackTop.value - 1)
        const dest_array = new Uint8Array(Memory.buffer, dest_pos, 32)
        const dest_bn = arrayToBn(dest_array)

        BignumStackTop.value = BignumStackTop.value - 2
        return Number(dest_bn)
      } else {
        BignumStackTop.value = BignumStackTop.value - 2
        return pc
      }
    },
  },
  env: {
    abort(_msg, _file, line, column) {
      console.error("[abort] abort called at main.ts:" + line + ":" + column);
    },
  },
})

Memory = obj.memory
BignumStackTop = obj.BignumStackTop

function run_evm() {
  obj.run_evm()
}

console.time('run_evm')
run_evm()
console.timeEnd('run_evm')

