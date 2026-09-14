# Stack Machine Version 2

<img src="version_1/images/completed_back.jpeg"></img>

Photo of Version 1

## Changes from previous version

* The timing will be completely redone: A single normal clock signal, not the stupid "AB Alternating" I made up for the original.
* The PC will be incremented and a new instruction fetched during any non-flow-control instructions except when the GPRAM is used and the PC is in the same RAM chip (the RAM chips have 15 address lines each) as whatever is being accessed. The bus timing will also be very flexible and will be faster when possible but still slow when the memory is being accessed twice for example.
* Bus read/write addresses are both now 5 bits. The WRITE/MOVE instruction formats will remain the same and the extra bit (MSB to be exact) will be set based on duplicate opcodes for WRITE/MOVE that are otherwise interpreted the same. 1 extra WRITE opcode and 3 extra MOVE opcodes.
* The opcode decoder is actually 3 bits. There will be 4 bits used for extra MOVE/WRITE instructions (see above).
* Interrupts: There will be seperate interrupt GOTO latches that go to a dedicated interrupt handler function. That function should save the state of everything else (including the regular GOTO latches, which is why interrupt GOTO latches are seperate) onto the stack then do whatever depending on the interrupt code.
* GOTO latches can be written to the bus (see above). The goto decider doesn't need to be because it's simple to get its value from an IF-ELSE.
* The control unit will be able to fetch instructions from the GPRAM: Program addresses starting at 2^15 (32,768, halfway through the old flash memory space) will load 2 consecutive bytes (an instruction is 2 bytes) from the GPRAM. The GPRAM address of the first byte (lower byte of the instruction) will be determined by: `(prog_addr - 32,768) * 2`. Since that will only happen at or above 32,768 then the MSB will not be used in the math and will be used to determine whether to read from flash or GPRAM.
* Flash program space is selectable: The flash chips have 17 address lines, while 15 are used (see above about half the program address space being mapped to the GPRAM). This means that the top 2 lines can be connected to jumpers to select between 4 different programs.
* Flash program can be written to from the bus: The programming interface takes 8 data inputs + power and GND. The old interface to plug an Aduino in to program it will still be there of course but it can be selected by a jumper to be either the external pins (to Arduino) or from the bus.
* The stack is 32k instead of 64k so only one memory chip is used, although 32k is still incredibly excessive.

### GPRAM Parallelization

The RAM chips have 15 address lines each so there will be two of them (Lower and Upper Domains). This means that they can be used seperately at the same time, for example fetching the new instruction (If the PC is in the RAM) and also doing GPRAM read/write, as long as these operations are using seperate domains.
Memory users will use either (or both) of the lower or upper RAM domains:

* Program: PC and data read into instruction registers A & B
* Heap R/W: Exactly the same as the original

# Program instructions

Each instruction will be 16 bits and interpreted by the control unit. The first 4 bits (0 - 3) of the instruction will address the below list of operations. The rest of the instruction (bits 4 - 15) may be ignored or used for different things depending on the specific operation.

Here's the current list of the operation codes (opcodes):

0. `MOVE` - Bus usage - The rest of the instruction will be interpreted as follows: bits 8 - 11 address the device which will set the state of the bus (TX) and bits 12 - 15 will address the device to read from it (RX). Bits 4 - 7 are sent to the ALU as it's opcode incase the data is comming from it.
1. `WRITE` - Writes instruction bits 4 - 11 to the bus. Bits 12 - 15 address the device to read from it.
2. `GOTO` - Saves the 2 execution pointer GOTO latches (each of them are 1 byte) to the program counter
3. `GOTO-IF` - Reads the LSB of the value in the goto decider latch and does a GOTO only if it is 1, otherwise does nothing
4. `HALT` - Stops the clock, usefull for debugging
5. `CALL` - Effectively the same as `GOTO` but also pushes the return address (current value of program counter) onto the call stack. Note: The return address can be copied as-is and does not need to be incremented because it will be incremented normally after each `RETURN` instruction when it is used. If bit 4 is 1 this means that this call is calling the interrupt handler using the Interrupt GOTO latches.
6. `RETURN` - The program counter will be set to the return address popped off the top of the call stack. If bit 4 is 1 this means that this return instruction is returning from the interrupt handler function and will allow interrupts again.
7. `CONFIG-INT` - Configure interrupt. Bit 4 enables or disables interrupts. Interrupts are disabled on startup.

The following are treated as either MOVE or WRITE by the sequencer and set the bus TX and RX MSBs. The original MOVE and WRITE have all MSBs = 0.

8. `MOVE` - MSBs TX=1, RX=0
9. `MOVE` - MSBs TX=0, RX=1
10. `MOVE` - MSBs TX=1, RX=1
11. `WRITE` - MSBs RX=1

# Bus

Up to 15 devices can read the bus and 16 write to it.
Devices that can read the bus (RX):

0. `NONE` - Nothing reads this so that bytes can be popped from the stack without them going anywhere.
1. `STACK-PUSH` - Stack controller (Push)
2. `ALU-A` - ALU latch A
3. `ALU-B` - ALU latch B
4. `GOTO-A` - Control unit - Program counter GOTO latch A (first byte)
5. `GOTO-B` - Control unit - Program counter GOTO latch B (second byte)
6. `GOTO-DECIDER` - Control unit - GOTO decider latch (For GOTO-IF)
7. `GPRAM` - GPRAM - Write
8. `GPRAM-INC-ADDR` - GPRAM - Write (++ address)
9. `GPRAM-ADDR-A` - GPRAM - Address bits 0 - 7
10. `GPRAM-ADDR-B` - GPRAM - Address bits 8 - 15
11. `GPIO-WRITE-A` - Writes to GPIO output pins 0 - 7
12. `OFFSET-WRITE` - Replaces value in stack at `ToS - offset`
13. `SET-STACK-OFFSET` - Sets the stack offset byte
14. `ALU-C-IN` - ALU latch for carry and borrow, only LSB is used. Make sure this is always zero when not being used otherwise additions will be wrong.
15. `GPIO-WRITE-B` - Writes to GPIO output pins 8 - 15
16. `INT-GOTO-A` - Interrupt GOTO A
17. `INT-GOTO-B` - Interrupt GOTO B
18. `VECTORS-A` - Vector graphics A input
19. `VECTORS-B` - Vector graphics B input
20. `VECTORS-C` - Vector graphics C input
21. `VECTORS-D` - Vector graphics D input
22. `INT-AND-MAIN-TIMER-ADDRESS` - Bits 0 - 1 address 1 of the 4 interrupt timers. Bit 2 will reset the addressed counter to 0.
23. `INT-TIMER-CONFIG-MAX` - Sets the start value of one of the interrupt timers
24. `INT-TIMER-CONFIG-TIMEBASE-AND-ENABLE` - Configures one of the interrupt timers, see `timer_board.md`
25. `FLASH` - Flash programming interface, same as the arduino connection
26. `RX-26` - Expansion RX 25

...

31. `RX-31` - Expansion RX 31

Devices that can write to the bus (TX):

0. `STACK-POP` - Stack controller (pop)
1. `OFFSET-READ` - Reads value from stack at `ToS - offset`, does not affect the ToS
2. `ALU` - ALU output
3. Control unit instruction bits 4 - 11, used for the `WRITE` instruction
4. `GPRAM` - GPRAM - Read
5. `GPRAM-INC-ADDR` - GPRAM - Read (++ address)
6. `GPRAM-ADDR-A` - GPRAM - Address bits 0 - 7
7. `GPRAM-ADDR-B` - GPRAM - Address bits 8 - 15
8. `GPIO-READ-A` - Reads GPIO input pins 0 - 7
9. `MAIN-TIMER` - 8 bits of the main timer, addressed by `INT-AND-MAIN-TIMER-ADDRESS`
10. `INT-TIMER` - Current value of 1 of the 4 interrupt timers, addressed by `INT-AND-MAIN-TIMER-ADDRESS`
11. `GPIO-READ-B` - Reads GPIO input pins 8 - 15
12. `INT-CODE` - Most recent interrupt code, will be cleared upon read
13. `INT-COUNT` - Number of interrupts in queue, will overflow past 255
14. `GET-STACK-OFFSET` - Retrieves stack offset, needed to save state during interrupt
15. `GET-GOTO-A` - Read GOTO Latch A
16. `GET-GOTO-B` - Read GOTO Latch B
17. `TX-17` - Expansion TX 17

...

31. `TX-31` - Expansion TX 31

# The Stack

The stack will simply be a piece of memory seperate from the GPRAM and managed by hardware, its has 32k x 8-bit words.

There will be two ways to access the stack:

1. Push and Pop
2. Using the current ToS pointer - stack offset (`OFFSET-WRITE` and `OFFSET-READ`) (does not change ToS). The stack offset will be a 1-byte number set from the bus (`SET-STACK-OFFSET`) and will not affect pushes and pops.

# ALU

The ALU uses 2 8-bit latches for input and has 1 output. The specific operation it does is controlled by 4 bits (2^4 = 16 operations). All operations that result in a boolean output all `0`s except for the result which is the LSB.

0. `ADD` - Add
1. `ADD-C` - Addition carry output (just the LSB will be set), this can be written directly into `ALU-C-IN`
2. `NOT` - Bitwise NOT (latch A)
3. `OR` - Bitwise OR
4. `AND` - Bitwise AND
5. `XNOR` - Bitwise XNOR, whether each pair of bits are equal
6. `SHIFT` - Bitshifts A left by the first 3 bits of B, bits that go off the end DO wrap around.
7. `EQ` - Whether bytes are equal
8. `A` - Contents of A latch
9. `B` - Contents of B latch
10. `EXT-10` - Extension (not implemented)
11. `EXT-11` - Extension (not implemented)
12. `EXT-12` - Extension (not implemented)
13. `EXT-13` - Extension (not implemented)
14. `EXT-14` - Extension (not implemented)
15. `EXT-15` - Extension (not implemented)

# Flow Control

There are four instructions for flow control: `GOTO`, `GOTO-IF`, `CALL`, and `RETURN`.

## GOTO and GOTO-IF

A `GOTO` instruction is used to jump to anywhere in the program. it uses the two 8-bit latches (`GOTO-A` and `GOTO-B`) to set the 16-bit program counter. Due to how the hardware is set up, the PC will be incremented after each goto instruction. This means that the previous address to where the execution should jump to must be used.

`GOTO-IF` is a conditional goto which will do what is described above only if the bit set by `GOTO-DECIDER` from the LSB of the bus is 1, otherwise it does nothing. This is the only instruction that allows the computer to "branch" or "make decisions".

There are macros, `@goto(anchor_name)` and `@goto_if(anchor_name)` that make GOTOs a lot easier.

## Functions and the call stack

Functions in the machine code are defined by `CALL` and `RETURN` instructions. To call a function, make sure the correct pointer to the beginning of the function is stored in the control unit goto A and B latches. Then use the `CALL` instruction which is basically a GOTO but will first put the current program counter value onto the call stack as the return address. The return instruction pops a return address off of the call stack and continues execution.

There is a macro in the assembly language, `@call(function_name)` that makes calling a lot easier.

The call stack is a seperate stack memory containing 256 (1 byte address size) 16-bit words. Each word will be a return address - where to set the program counter during a `RETURN` instruction.

# I/O

There are 16 input and 16 seperate output pins accessible with the `GPIO-READ-A`, `-B` and `GPIO-WRITE-A`, `-B` bus sources/destinations.

## Writing to the display

The display has 32 x 32 LEDs, any given row of 8 pixels are written all at once with one byte. These rows of 8 are addressed by a 7-bit number starting at the top-left and going across for 4 per row (32 / 8 = 4) for each of the 32 rows of the display. The display currently is attached to the gpio-write a and b outputs with gpio-write-a being the address (MSB is ignored) and gpio-write-b being the data. The signal to write the address/data to the display's memory is sent by the hardware when gpio-write-b is written to, meaning the address should be written first and then the data.

# General purpose static-RAM (GPRAM)

This piece of memory does not have any hardware protection like the stack and can be writen to and read from at any location. It will have a 16-bit address by 8-bit word size (65,536 bytes) just like the stack.

Its address can be optionally incremented upon reads/writes and can be directly set from the bus (`GPRAM-ADDR-A` and `GPRAM-ADDR-B`). For the instructions where the address is incremented, that will happen after the read/write.

# Design software

First, everything was built and tested in a digital circuit simulator [Rusty Logic](https://github.com/emperor-hadrian/rusty_logic_core) which allowed my to check that all the timing diagrams would be followed.

Once everything was working in Rusty Logic, KiCad was used to layout all of the actual PCBs.