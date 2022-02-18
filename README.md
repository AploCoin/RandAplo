# APLORandom
Random hashing algorithm with variable memory usage.

Designed to be much slower to execute on GPUs and FPGA(with slow ram)

Algorithm uses custom VM

VM's instrution opcode is 2 byte long, instructions have different sizes

# VM instruction set
# Now it generates random opcodes for instructions

Number | Arguments | Description 
-------|-----------|------------
||**Static data operations**|
|||
0|N - 2 bytes, N bytes of data|0 - N - (N bytes of data), pushes to the back N bytes, specified after N
1|N - 2 bytes, N bytes of data|Same with instruction 0, but pops/pushes from/to the front
|||
||**Integer operations**|
|||
2|N - 2 bytes|Performing sum on integers, pops 2 N bit intgers from front, sums them, pushes N bit result to the front
3|N - 2 bytes|Same with instruction 2, but pops/pushes from/to the back
4|N - 2 bytes|Performs subtraction on integers, pops 2 N bit signed integers from front, subtracts them, pushes N bit result to the front
5|N - 2 bytes|Same with 4, but pops/pushes from/to the back
6|N - 2 bytes|Performs unsigned integer multiplication, pops 2 N byte unsigned integers from front, multiplies them, and pushes to the front
7|N - 2 bytes|Same with 6, but pops/pushes from/to the back
|||
||**Float operations**|
|||
8|4 - bytes|performs sum of floats(4bytes), pops 2 operands from front, sum them push to front, if encountered overflow or underflow, then 4 bytes specified after opcode are used
9|4 - bytes|Same with 8, but pops/pushes from/to back
10|4 - bytes|Same with 8, but multiplication
11|4 - bytes|Same with 10, but pops/pushes from/to back
12|4 - bytes|Same with 8, but subtraction
13|4 - bytes|Same with 12, but pops/pushes from/to back
14|4 - bytes|Same with 8, but division
15|4 - bytes|Same with 14, but pops/pushes from/to back
|||
||**Double operations**|
|||
16-23||Same operations as for floats, but with doubles
|||
||**Bitwise operations**|For all bitwise operations N = 1+N % bitwise_max_size, where bitwise_max_size is a static predefined number and N - 2 bytes poped from stack, back/front same with instruction's operands
|||
24||Pops 2 operands with size of N bytes from front, performs logical OR(|) pushes result back to front
25||Same with 24, but pops/pushes from/to back
26||Same with 24, but logical AND(&)
27||Same with 26, but pops/pushes from/to back
28||Same with 24, but logical XOR(^)
29||Same with 28, but pops/pushes from/to back
30||Pops N bytes from front, performs bitwise NOT, pushes back to front
31||Same with 32, but pops/pushes from/to back
32|S - 1 byte|Bitwise shift left, shifts N bytes, poped from front, by S bits
33|S - 1 byte|Same with 32, but pops/pushes from/to back
34|S - 1 byte|Bitwise shift right, front
35|S - 1 byte|Bitwise shift right, back
|||
||**Operands via "addresses"**|Address in that context is 64 bit unsigned integer, that specifies position in stack
|||
36||(Address):Pops 64 bit unsigned integer(big-endian) from front, performs: address = address%(stack.len()-8), takes 2 32bit integers from specified address(big-endian) sums them, psuhes 32bit result to front
37||Pops address from back(big-endian), performs sum on operands, pushes to back
38||Same with 36, but subtraction(Front)
39||Same with 47, but subtraction(Back)
|||
||**Digest operations**|
|||
40||Pops 32bit integer from back(big-endian), pushes to final state
41||Same with 41, but pushes/pops to/from front


# Algorithm Execution

Algorithm executes, while length of stack is greater, than specified maximum output size.

After execution is finished, VM pops 32 bit integers(big-endian) from front of the stack and pushes them to the final state. If there left 1/2/3 bytes in stack, they are converted to 32 bit integer and pushed to the final state.

# Algorithm Creation

Algorithm creation process can be changed, but specified algorithm creator acts like that:

1.Get RNG, RNG is based on MD5 hash, get random number in range [0,1000), plus that number with min_inp_batch, resulting number is minimal amount of bytes, that algorithm will require to pass to it for execution.

2.Push result from 1'st step to the algorithm(unsigned 64 bit integer, little-endian)

3.Check AlgorithmCreator.rs for steps on actual algorithm creation

4.After algorithm is created, instruction 40 added to the end, that instruction ensures, that stack will be processed from 2 sides, in 1 algorith cycle

# Padding

Can be used any padding scheme, Padding used in implemetation is based on murmur2(32bit) hash, m = 0x5bd1e995

Actual data to be hashed is used as seed for RNG based on murmur2.

Data is getting padded until desired size, if initial_size <= desired_size-4, then push all 4 bytes of random number(from RNG), number is pushed in big-endian order

if initial_size > desired_size-4, then pushed 1/2/3 bytes of number, starting from least significant byte







