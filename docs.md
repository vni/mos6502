## 6502 ISA
* https://en.wikibooks.org/wiki/6502_Assembly
* http://www.oxyron.de/html/opcodes02.html
* https://www.pagetable.com/c64ref/6502/?tab=3

## Assembly
%00001111 -- binary
$FA       -- hex
123       -- decimal

## Registers
A -- Accumulator (A) -- 8 bit
X -- X register (X) -- 8 bit
Y -- Y register (Y) -- 8 bit
PC -- Program Counter (PC) -- 16 bit // next instruction to be executed
S -- Stack Pointer (S) -- 8 bit // points to the empty cell in stack
P -- Status (P)

## Status Flags
bit 7 -- N -- Negative -- // bit 7 of the result
bit 6 -- V -- Overflow
bit 5 -- - -- Unused
bit 4 -- B -- Break // Interrupt by a BRK instruction
bit 3 -- D -- Decimal // mathematical instructions will treat the inputs and outputs as "Binary Coded Decimal" (BCD) numbers
bit 2 -- I -- Interrupt Disable // Disable IRQ interrupts while set
bit 1 -- Z -- Zero
bit 0 -- C -- Carry

## Memory Layout
$0000 - $00FF -- Zero Page --
$0100 - $01FF -- Stack -- Growth backwards from $01FF to $0100
$0200 - $FFFF -- General Purpose memory

## Hardware vectors
$FFFA - NMI Vector (NMI = not maskable interrupts)
$FFFC - Reset Vector
$FFFE - IRQ Vector

