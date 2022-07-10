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



const AMOUNT_OF_EXE_INSTRUCTIONS:usize = 49;
const AMOUNT_OF_INSTRUCTIONS:usize = 49;
static MAX_INSTRUCTION_SIZE:u64 = 65536;
static MAX_POP_SIZE_BITWISE:usize = 16;
static MAX_SHIFT_LIMIT:usize = 16;

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
                        mut block_size:usize,
                        min_block_size:usize,
                        max_instructions:usize,
                        statics_amount:usize,
                        statics_max_size:usize
                    ) -> Result<Vec<u8>,&'static str>{

    let mut rng = RNG::MD5RNG::get_generator(seed);
    
    let mut algorithm: Vec<u8> = Vec::with_capacity(max_instructions);

    let mut instructions:usize = 0;
    let mut static_counter:usize = 0;

    //&& block_size >= min_block_size
    while instructions < max_instructions
            {

        let instruction = Tools::biguint_into_u8(rng.generate()%AMOUNT_OF_EXE_INSTRUCTIONS);
    
        match instruction{
            0|1 =>{
                if static_counter >= statics_amount{
                    continue;
                }
                static_counter += 1;

                algorithm.push(instruction);

                let mut N:u16 = 1 + Tools::biguint_into_u16(rng.generate()%statics_max_size);
            
                block_size += N as usize;

                let converted_num:[u8;2] = unsafe{transmute(N)};
                algorithm.push(converted_num[1]);
                algorithm.push(converted_num[0]);

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
                algorithm.push(instruction);

                let amount_of_bytes:u16 = 1 + Tools::biguint_into_u16(rng.generate()%100u32);
                let N:u16 = amount_of_bytes*4;

                push_u16_to_u8_vec!(algorithm,N);

                let mut min_amount_of_bytes = N/8;
                if min_amount_of_bytes == 0
                        || min_amount_of_bytes%8 != 0{
                    min_amount_of_bytes += 1;
                }
                block_size -= min_amount_of_bytes as usize;
            }
            6|7 => {
                algorithm.push(instruction);
                let N:u16 = 1 + Tools::biguint_into_u16(rng.generate()%100u32);
                
                algorithm.push(((N&0xff00)>>8) as u8);
                algorithm.push((N&0xff) as u8);

                block_size -= N as usize;
            }
            8|9|10|11|12|13|14|15 => {
                algorithm.push(instruction);
                let overflow_bytes:u32 = Tools::biguint_into_u32(rng.generate());
                push_u_to_u8_vec!(algorithm,overflow_bytes);

                block_size -= 4;
            }
            16|17|18|19|20|21|22|23 => {
                algorithm.push(instruction);
                let overflow_bytes:u64 = Tools::biguint_into_u64(rng.generate());
                push_u_to_u8_vec!(algorithm,overflow_bytes);

                block_size -= 8;
            }
            24|25|26|27|28|29 => {
                algorithm.push(instruction);
                block_size -= MAX_POP_SIZE_BITWISE;
            }
            30|31 => {
                algorithm.push(instruction);
                block_size -= 2;
            }
            32|33 => {
                algorithm.push(instruction);
                let mut S:u8 = Tools::biguint_into_u8(rng.generate())%MAX_SHIFT_LIMIT as u8;

                algorithm.push(S);

                while S >= 8{
                    block_size += 1;
                    S -= 8;
                }
            }
            34|35 => {
                algorithm.push(instruction);
                let mut S:u8 = Tools::biguint_into_u8(rng.generate())%MAX_SHIFT_LIMIT as u8;
                algorithm.push(S);
                while S >=8 {
                    block_size -= 1;
                    S -= 8;
                }
            }
            36|37|38|39 => {
                algorithm.push(instruction);
                block_size -= 4;
            }
            40|41 =>{
                algorithm.push(instruction);
                block_size -= 16;
            }
            42|43 =>{
                algorithm.push(instruction);
                block_size -= 24;
            }
            44|45 =>{
                algorithm.push(instruction);
                block_size -= 32;
            }
            46 =>{
                algorithm.push(instruction);
            }
            _ => {continue;}
        }

        instructions += 1;
    }

    algorithm.push(46);
    algorithm.push(47);

    println!("Instructions count: {:?}",instructions);

    return Ok(algorithm);
}