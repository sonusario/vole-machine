# Vole-Machine

A simple virtual machine for educational purposes.

---
---

## [Table of Contents](#table-of-contents)

- [The Architecture](#the-architecture)
- [The Instruction Set](#the-instruction-set)
- [Program Walkthroughs](#program-walkthroughs)
  - [Program A](#program-a)

---
---

## [The Architecture](#table-of-contents)

The Vole-Machine is a simple virtual machine with a 16-bit instruction set. It has 16 general-purpose registers, 256 bytes of memory, and a program counter. The registers are indexed from 0 to 15, and the memory is indexed from 0x00 to 0xFF. The program counter is a single byte, and the instructions are also a single byte. A full instruction is 16-bits, with the first 4 bits representing the opcode and the remaining 12 bits representing the operands. The program counter increments by 2 after each instruction is executed, and it wraps around when overflow occurs.

```rust
struct Cpu {
    register: [u8; 16],
    memory: [u8; 256],
    pc: u8,
}
```

> [!NOTE]
> The actual struct code is not shown here, but the above code snippet is a simplified version of the struct that represents the Vole-Machine. The actual struct contains additional fields that where used for debugging, testing, and printing.

---
---

## [The Instruction Set](#table-of-contents)

The characters `R`, `S`, `T` are placeholders for register indices, `XY` is the placeholder for a memory address or an immediate value, a singular `X` is the placeholder for the number of bits to rotate, and the `0x` prefix indicates a hexadecimal value.

```rust
no_op       // 0x0000       :: No Operation
load_from   // 0x1[RXY]     :: Load from m0xXY into rR
load        // 0x2[RXY]     :: Load 0xXY into rR
store       // 0x3[RXY]     :: Store from rR into m0xXY
move_op     // 0x40[RS]     :: Move from rR to rS
add_tc      // 0x5[RST]     :: rS + rT into rR (Two's Complement)
add_fl      // 0x6[RST]     :: rS + rT into rR (Floating Point)
or          // 0x7[RST]     :: rS | rT into R
and         // 0x8[RST]     :: rS & rT into rR
xor         // 0x9[RST]     :: rS ^ rT into rR
rotate      // 0xA[R]0[X]   :: rR >> 0xX // Rotate Right X bits
jump        // 0xB[RXY]     :: if rR == r0 then PC = m0xXY
halt        // 0xC000       :: Halt
```

> [!WARNING]
> I've not verified the correctness of adding floating-point numbers with the `add_fl` instruction.

## [Program Walkthroughs](#table-of-contents)
<!--Time series table:: x-axis: iterations, y-axis: address and/or instructions-->

The following tables are iteration-series data for the number of instructions executed in each iteration of a program. The column headers contain the iteration numbers, and the row headers represent the addresses and instruction executed in that iteration.
If the instruction field is empty, it means that the row is mentioned for reference and is not a part of the program (unless the program counter winds up pointing to it).
The content of the iteration fields indicates what the instruction does in that iteration or the state of the memory and registers after the entire iteration is executed.

> [!NOTE]
> The end of an iteration should be assumed when the program counter follows
a jump instruction or when the program halts.

### [Program A](#table-of-contents)

<!--
| 0x2003 | // Load 0x03 into r0                               
| 0x2101 | // Load 0x01 into r1                               
| 0x2200 | // Load 0x00 into r2                               
| 0x2310 | // Load 0x10 into r3 // 0x10 == 16                 
| 0x1400 | // Load from m0x00 into r4                         
| 0x3410 | // Store from r4 into m0x10 // m0x10 == memory[16] 
| 0x5221 | // r2 + r1 into r2                                 
| 0x5331 | // r3 + r1 into r3                                 
| 0x3239 | // Store from r2 into m0x39 // m0x39 == memory[57] 
| 0x333B | // Store from r3 into m0x3B // m0x3B == memory[59] 
| 0xB248 | // Jump to m0x48 if r2 == r0 // m0x48 == memory[72]
| 0xB038 | // Jump to m0x38 if r0 == r0 // m0x38 == memory[56]
| 0xC000 | // Halt                                            
-->

| Addresses| Instruction | 1 | 2 | 3 |
|:---------:|:-----------:|:-:|:-:|:-:|
| ... | ... | ... | ... | ... |
| `m0x10` | ~ | `0x00` | ~ | ~ |
| `m0x11` | ~ | ~ | `0x00` | ~ |
| `m0x12` | ~ | ~ | ~ | `0x00` |
| ... | ... | ... | ... | ... |
| `m0x30`, `m0x31` | `0x2003` | `r0` = `0x03` | ~ | ~ |
| `m0x32`, `m0x33` | `0x2101` | `r1` = `0x01` | ~ | ~ |
| `m0x34`, `m0x35` | `0x2200` | `r2` = `0x00` | ~ | ~ |
| `m0x36`, `m0x37` | `0x2310` | `r3` = `0x10` | ~ | ~ |
| `m0x38`, `m0x39` | `0x1400`* | `r4` = `m0x00` <hr> `0x1401`* | `r4` = `m0x01` <hr> `0x1402`* | `r4` = `m0x02` <hr> `0x1403`* |
| `m0x3A`, `m0x3B` | `0x3410`* | `m0x10` = `r4` <hr> `0x3411`* | `m0x11` = `r4` <hr> `0x3412`* | `m0x12` = `r4` <hr> `0x3413`* |
| `m0x3C`, `m0x3D` | `0x5221` | `r2` = `r2 + r1` | `r2` = `r2 + r1` | `r2` = `r2 + r1` |
| `m0x3E`, `m0x3F` | `0x5331` | `r3` = `r3 + r1` | `r3` = `r3 + r1` | `r3` = `r3 + r1` |
| `m0x40`, `m0x41` | `0x3239` | `m0x39` = `r2` | `m0x39` = `r2` | `m0x39` = `r2` |
| `m0x42`, `m0x43` | `0x333B` | `m0x3B` = `r3` | `m0x3B` = `r3` | `m0x3B` = `r3` |
| `m0x44`, `m0x45` | `0xB248` | r2 `!=` r0 | r2 `!=` r0 | jump `pc` = `0x48` |
| `m0x46`, `m0x47` | `0xB038` | jump `pc` = `0x38` | jump `pc` = `0x38` | ~ |
| `m0x48`, `m0x49` | `0xC000` | ~ | ~ | ~ |
| ... | ... | ... | ... | ... |
| **Registers** | **Initial Value** ||||
| `r0` | `0x00` | `0x03` | ~ | ~ |
| `r1` | `0x00` | `0x01` | ~ | ~ |
| `r2` | `0x00` | `0x01` | `0x02` | `0x03` |
| `r3` | `0x00` | `0x11` | `0x12` | `0x13` |
| `r4` | `0x00` | `0x00` | `0x00` | `0x00` |
| ... | ... | ... | ... | ... |
| **Program Counter** | **Initial Address** ||||
| `pc` | `m0x30` | `0x38` | `0x38` | `0x48` |

> [!IMPORTANT]
> The program counter (`pc`) starts at pointing at `m0x30` (i.e. `pc` = `0x30`), and the program halts if it reaches `m0x48`.
>
> Instructions marked with an asterisk `*` are updated at some point during the execution of the program.
>
> Unless otherwise specified all memory addresses and registers are initialized to `0x00`.

### [Program B](#table-of-contents)

<!--
| 0x2004 | // Load 0x04 into r0
| 0x2101 | // Load 0x01 into r1
| 0x4012 | // Move from r1 to r2
| 0x5112 | // r1 + r2 into r1
| 0xB10C | // Jump to m0x0C if r1 == r0
| 0xB006 | // Jump to m0x06 if r0 == r0
| 0xC000 | // Halt
-->

| Addresses | Instruction | 1 | 2 | 3 |
|:---------:|:-----------:|:-:|:-:|:-:|
| `m0x00`, `m0x01` | `0x2004` | `r0` = `0x04` | ~ | ~ |
| `m0x02`, `m0x03` | `0x2101` | `r1` = `0x01` | ~ | ~ |
| `m0x04`, `m0x05` | `0x4012` | `r2` = `r1` | ~ | ~ |
| `m0x06`, `m0x07` | `0x5112` | `r1` = `r1 + r2` | `r1` = `r1 + r2` | `r1` = `r1 + r2` |
| `m0x08`, `m0x09` | `0xB10C` | r1 `!=` r0 | r1 `!=` r0 | jump `pc` = `0x0C` |
| `m0x0A`, `m0x0B` | `0xB006` | jump `pc` = `0x06` | jump `pc` = `0x06` | ~ |
| `m0x0C`, `m0x0D` | `0xC000` | ~ | ~ | ~ |
| ... | ... | ... | ... | ... |
| **Registers** | **Initial Value** ||||
| `r0` | `0x00` | `0x04` | ~ | ~ |
| `r1` | `0x00` | `0x01` | `0x02` | `0x03` |
| `r2` | `0x00` | `0x00` | `0x01` | `0x01` |
| ... | ... | ... | ... | ... |
| **Program Counter** | **Initial Address** ||||
| `pc` | `m0x00` | `0x06` | `0x06` | `0x0C` |

> [!IMPORTANT]
> The program counter (`pc`) starts at pointing at `m0x00` (i.e. `pc` = `0x00`), and the program halts if it reaches `m0x06`.
