use crate::RNG;
use crate::Tools;
use std::mem::transmute;
use num_bigint::BigUint;
use num_bigint::{ToBigUint};
use ::std::*;

#[macro_export]
macro_rules! push_u16_to_u8_vec{
    ($buffer:expr,$number:expr)=>{
        $buffer.push((($number&0xff00)>>8) as u8);
        $buffer.push(($number&0x00ff) as u8);
    }
}

#[macro_export]
macro_rules! push_u_to_u8_vec{
    ($buffer:expr,$number:expr)=>{
        for byte in $number.to_be_bytes().iter(){
            $buffer.push(*byte);
        }
    }
}



const AMOUNT_OF_EXE_INSTRUCTIONS:usize = 42;
const AMOUNT_OF_INSTRUCTIONS:usize = 42;
static MAX_INSTRUCTION_SIZE:u64 = 65536;

pub fn generate_lookup_table<'a>(seed:&mut [u8]) -> [u16;AMOUNT_OF_INSTRUCTIONS]{
    let mut to_return:[u16;AMOUNT_OF_INSTRUCTIONS] = [0;AMOUNT_OF_INSTRUCTIONS];
    
    let mut rng = RNG::MD5RNG::get_generator(seed);

    for instruction_number in 0..AMOUNT_OF_INSTRUCTIONS{
        let instruction_code:u16 = Tools::biguint_into_u16(rng.generate()%MAX_INSTRUCTION_SIZE);
        let mut found = false;
        for i in 0..instruction_number{
            if to_return[i] == instruction_code{
                found = true;
                break;
            }
        }
        if found{
            continue;
        }

        to_return[instruction_number] = instruction_code;
    }
    return to_return;
}


pub fn create_algorithm(seed:&mut [u8],
                        min_inp_batch:u64,
                        result_size:u64,
                        minimal_instructions:u64
                    ) -> Result<Vec<u8>,&'static str>{

    let mut rng = RNG::MD5RNG::get_generator(seed);

    let input_batch_size:u64 = min_inp_batch+
        Tools::biguint_into_u64(rng.generate()%1000usize);

    let size_koef:u32 = 100;
    let static_values_max_size:u32 = 0xffff;

    let bitwise_right_shift_limit:u8 = 8;

    let bitwise_left_shift_limit:u8 = 8;
    let bitwise_max_size:u8 = 32;

    let max_static_values = 1;
    //let minimal_instructions:u64 = input_batch_size/200;
    println!("Minimal instructions: {}",minimal_instructions);

    //let mut bytes: [u8; 8];
    let mut algorithm: Vec<u8> = Vec::with_capacity(min_inp_batch as usize);
    //bytes = unsafe { transmute(input_batch_size) };

    push_u_to_u8_vec!(algorithm,input_batch_size);

    let mut current_stack_size:u64 = input_batch_size;
    
    let mut static_values_counter:u64 = 0;

    let mut instruction:u8 = 0;

    let mut instruction_number:u64 = 0;

    let mut N:u64 = 0;
    while instruction_number < minimal_instructions 
            && current_stack_size>result_size{
                
        instruction = Tools::biguint_into_u8(rng.generate()%AMOUNT_OF_EXE_INSTRUCTIONS);
        
        let instruction_opcode:u8 = instruction;

        match instruction{
            0|1 => {
                if static_values_counter == max_static_values{
                    continue;
                }
                static_values_counter += 1;

                algorithm.push(instruction_opcode);
                
                let mut N:u16 = 1 + Tools::biguint_into_u16(rng.generate()%static_values_max_size);
                //println!("N: {}",N);
                current_stack_size += N as u64;
                let converted_num:[u8;2] = unsafe{transmute(N)};
                //println!("converted: {:X?}",converted_num);
                algorithm.push(converted_num[1]);
                algorithm.push(converted_num[0]);
                //println!("alg: {:X?}",algorithm);

                while N >= 8{
                    let num:u64 = Tools::biguint_into_u64(rng.generate());
                    push_u_to_u8_vec!(algorithm,num);
                    N -= 8;
                }
                while N >= 4{
                    let num:u32 = Tools::biguint_into_u32(rng.generate());
                    push_u_to_u8_vec!(algorithm,num);
                    N -= 4;
                }
                while N >= 2{
                    let num:u16 = Tools::biguint_into_u16(rng.generate());
                    push_u16_to_u8_vec!(algorithm,num);
                    N -= 2;
                }
                while N >= 1{
                    let num:u8 = Tools::biguint_into_u8(rng.generate());
                    algorithm.push(num);
                    N -= 1;
                }

            }
            2|3|4|5 => {
                algorithm.push(instruction_opcode);
                //algorithm.push(instruction);

                let amount_of_bytes:u16 = 1 + Tools::biguint_into_u16(rng.generate()%size_koef);
                let N:u16 = amount_of_bytes*4;

                push_u16_to_u8_vec!(algorithm,N);

                let mut min_amount_of_bytes = N/8;
                if min_amount_of_bytes == 0
                        || min_amount_of_bytes%8 != 0{
                    min_amount_of_bytes += 1;
                }
                current_stack_size -= min_amount_of_bytes as u64;
            }
            6|7 => {
                algorithm.push(instruction_opcode);
                let N:u16 = 1 + Tools::biguint_into_u16(rng.generate()%size_koef);
                
                algorithm.push(((N&0xff00)>>8) as u8);
                algorithm.push((N&0xff) as u8);

                current_stack_size -= N as u64;
            }
            8|9|10|11|12|13|14|15 => {
                algorithm.push(instruction_opcode);
                let overflow_bytes:u32 = Tools::biguint_into_u32(rng.generate());
                push_u_to_u8_vec!(algorithm,overflow_bytes);

                current_stack_size -= 4;
            }
            16|17|18|19|20|21|22|23 => {
                algorithm.push(instruction_opcode);
                let overflow_bytes:u64 = Tools::biguint_into_u64(rng.generate());
                push_u_to_u8_vec!(algorithm,overflow_bytes);

                current_stack_size -= 8;
            }
            24|25|26|27|28|29 => {
                algorithm.push(instruction_opcode);
                current_stack_size -= bitwise_max_size as u64 + 2;
            }
            30|31 => {
                algorithm.push(instruction_opcode);
                current_stack_size -= 2;
            }
            32|33 => {
                algorithm.push(instruction_opcode);
                let mut S:u8 = Tools::biguint_into_u8(rng.generate())%bitwise_left_shift_limit as u8;

                algorithm.push(S);

                while S >= 8{
                    current_stack_size += 1;
                    S -= 8;
                }
            }
            34|35 => {
                algorithm.push(instruction_opcode);
                let mut S:u8 = Tools::biguint_into_u8(rng.generate())%bitwise_right_shift_limit;
                algorithm.push(S);
                while S >=8 {
                    current_stack_size -= 1;
                    S -= 8;
                }
            }
            36|37|38|39 => {
                algorithm.push(instruction_opcode);
                current_stack_size -= 4;
            }
            _ => {continue;}
        }
        instruction_number += 1;
    }   
    println!("Instruction number: {}",instruction_number);
    algorithm.push(40);
    // algorithm.push(lookup_table[40]);
    return Ok(algorithm);
}