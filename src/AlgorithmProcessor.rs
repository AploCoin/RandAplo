use std::io::{stdin, stdout, Read, Write};
use std::collections::VecDeque;
use std::collections::HashMap;
use num_bigint::{BigInt,Sign,BigUint};
use std::mem::transmute;
use crate::RNG::MurMur2RNG;
use std::iter::FromIterator;

use aes::Aes128;
use aes::Aes192;
use aes::Aes256;
use aes::cipher::{
    BlockCipher, BlockEncrypt, BlockDecrypt, KeyInit,
    generic_array::GenericArray,
};

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

pub struct VM{
    final_state:Vec<u32>,
    leading_zeros:u32
}

pub fn get_VM() -> VM{
    return VM{final_state:Vec::new(),
                leading_zeros:0};
}

fn shift_4_bits_left_stricted(input_vector:&mut Vec<u8>){
    let mut carry:u8 = 0x00;
    for byte in input_vector.iter_mut().rev(){
        let new_bytes:u16 = (*byte as u16) << 4;
        *byte = (new_bytes&0x00ff) as u8 | carry;
        carry = (new_bytes>>8) as u8;
    }
}

pub fn pad_data(data:&mut Vec<u8>,size:u64){
    let mut initial_size:u64 = data.len() as u64;
    let mut data_vector:Vec<u8> = Vec::with_capacity(data.len());
    for byte in data.iter(){
        data_vector.push(*byte);
    }
    let mut rng = MurMur2RNG::get_generator(&mut data_vector, initial_size);

    let mut bytes:[u8;4];

    while initial_size <= size-4{
        bytes = unsafe{transmute(rng.generate())};
        for byte in bytes.iter().rev(){
            data.push(*byte);
        }
        initial_size += 4;
    }

    match size-initial_size{
        1 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push(bytes[0]);
        }
        2 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push(bytes[0]);
            data.push(bytes[1]);
        }
        3 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push(bytes[0]);
            data.push(bytes[1]);
            data.push(bytes[2]);
        }
        _ => {}
    }

}

static lookup_table:[u8;16] = [b'0',b'1',b'2',b'3',
                                 b'4',b'5',b'6',b'7',
                                 b'8',b'9',b'A',b'B',
                                 b'C',b'D',b'E',b'F'];

static MAX_POP_SIZE_BITWISE:usize = 16;

impl VM{
    pub fn execute_from_buffer(&mut self,
                        buffer:&[u8],
                        stack:&mut Vec<u8>,
                        block_size:usize,
                        min_block_size:usize,
                        output_max_size:usize
                    ) -> Result<(),&'static str>{
        // will change stack variable
        
        let mut block_start:usize = 0;
        while stack.len() > output_max_size{
            let mut instruction_index:usize = 0;
            let mut block_end = block_start + block_size;
            if block_end > stack.len(){
                block_end = stack.len();
            }
            let mut block:VecDeque<u8> = VecDeque::from_iter(stack[block_start..block_end].iter().copied());
            
            let mut start_index = block_start;
            let mut end_index = block_end;

            while instruction_index != buffer.len()
                    && block.len() > min_block_size{

                let instruction_opcode = buffer[instruction_index];
                instruction_index += 1;

                match instruction_opcode{
                    0 => {
                        let mut N:u16 = (buffer[instruction_index] as u16) << 8;
                        N += buffer[instruction_index+1] as u16;

                        instruction_index += 2;

                        for i in 0..N{
                            block.push_back(buffer[instruction_index]);
                            instruction_index += 1;
                        }
                    }
                    1 => {
                        let mut N:u16 = (buffer[instruction_index] as u16) << 8;
                        N += buffer[instruction_index+1] as u16;

                        instruction_index += 2;

                        if block.capacity()-block.len() < N as usize{
                            block.reserve(N as usize - block.capacity());
                        }

                        for i in 0..N as usize{
                            block.push_front(buffer[instruction_index+i]);
                        }
                        instruction_index += N as usize;

                    }
                    2 => {
                        // int plus front
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;
                        let bytes_to_pop:u16 = N/4;

                        instruction_index += 2;
                
                        let mut min_amount_of_bytes = N/8;
                        if min_amount_of_bytes == 0
                                || min_amount_of_bytes%8 != 0{
                            min_amount_of_bytes += 1;
                        }

                        let mut poped_bytes:Vec<u8> = Vec::with_capacity(bytes_to_pop as usize);

                        for i in 0..bytes_to_pop{
                            poped_bytes.push(block.pop_front().unwrap());
                        }
                        
                        let mut first_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);

                        // getting first operand
                        let mut counter = N;
                        let mut index = 0;
                        while counter >= 8{
                            first_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }

                        if counter > 0{
                            first_operand.push(poped_bytes[index]&0xf0);
                            second_operand.push(poped_bytes[index]&0x0f);
                            index += 1;
                            counter = N - 4;
                        }else{
                            counter = N;
                        }

                        while counter >= 8{
                            second_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }
                        shift_4_bits_left_stricted(&mut second_operand);

                        
                        let mut carry:u8 = 0;
                        for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                            let result:u16 = *first as u16 
                                            + *second as u16 
                                            + carry as u16;
                            carry = (result>>8) as u8;
                            block.push_front((result&0x00ff) as u8);
                        }
                        

                    }
                    3 => {
                        // int plus back
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;
                        let bytes_to_pop:u16 = N/4;

                        instruction_index += 2;
                
                        let mut min_amount_of_bytes = N/8;
                        if min_amount_of_bytes == 0
                                || min_amount_of_bytes%8 != 0{
                            min_amount_of_bytes += 1;
                        }

                        let mut poped_bytes:Vec<u8> = Vec::with_capacity(bytes_to_pop as usize);

                        for i in 0..bytes_to_pop{
                            poped_bytes.push(block.pop_back().unwrap());
                        }
                        poped_bytes.reverse();
                        
                        let mut first_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);

                        let mut counter = N;
                        let mut index = 0;
                        while counter >= 8{
                            first_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }

                        if counter > 0{
                            first_operand.push(poped_bytes[index]&0xf0);
                            second_operand.push(poped_bytes[index]&0x0f);
                            index += 1;
                            counter = N - 4;
                        }else{
                            counter = N;
                        }

                        while counter >= 8{
                            second_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }
                        shift_4_bits_left_stricted(&mut second_operand);
                        
                        let mut resulting_bytes:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);


                        
                        let mut carry:u8 = 0;
                        for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                            let result:u16 = *first as u16 
                                            + *second as u16 
                                            + carry as u16;
                            carry = (result>>8) as u8;
                            resulting_bytes.push((result&0x00ff) as u8);
                        }
                        
                        for byte in resulting_bytes.iter().rev(){
                            block.push_back(*byte);
                        }
                        
                    }
                    4 => {
                        // int subtraction front
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;
                        let bytes_to_pop:u16 = N/4;

                        instruction_index += 2;
                
                        let mut min_amount_of_bytes = N/8;
                        if min_amount_of_bytes == 0
                                || min_amount_of_bytes%8 != 0{
                            min_amount_of_bytes += 1;
                        }

                        let mut poped_bytes:Vec<u8> = Vec::with_capacity(bytes_to_pop as usize);

                        for i in 0..bytes_to_pop{
                            poped_bytes.push(block.pop_front().unwrap());
                        }
                        
                        let mut first_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);

                        let mut counter = N;
                        let mut index = 0;
                        while counter >= 8{
                            first_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }

                        if counter > 0{
                            first_operand.push(poped_bytes[index]&0xf0);
                            second_operand.push(poped_bytes[index]&0x0f);
                            index += 1;
                            counter = N - 4;
                        }else{
                            counter = N;
                        }

                        while counter >= 8{
                            second_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }
                        shift_4_bits_left_stricted(&mut second_operand);

                        let mut first_operand_big:BigInt;
                        let mut second_operand_big:BigInt;

                        if first_operand[0]&0b10000000 != 0{
                            first_operand[0] = first_operand[0]&0b01111111;
                            first_operand_big = BigInt::from_bytes_be(Sign::Minus, &first_operand[..]);
                        }
                        else{
                            first_operand_big = BigInt::from_bytes_be(Sign::Plus, &first_operand[..]);
                        }

                        if second_operand[0]&0b10000000 != 0{
                            second_operand[0] = second_operand[0]&0b01111111;
                            second_operand_big = BigInt::from_bytes_be(Sign::Minus, &second_operand[..]);
                        }
                        else{
                            second_operand_big = BigInt::from_bytes_be(Sign::Plus, &second_operand[..]);
                        }

                        let result = first_operand_big - second_operand_big;

                        let (sign,mut resulting_bytes) = result.to_bytes_be();
                        
                        let res_bytes_length = resulting_bytes.len();
                        
                        for i in res_bytes_length..min_amount_of_bytes as usize{
                            resulting_bytes.push(0);
                        }

                        if res_bytes_length < min_amount_of_bytes as usize{
                            for (first,second) in (0..res_bytes_length).rev().zip((min_amount_of_bytes as usize-res_bytes_length..min_amount_of_bytes as usize).rev()){
                                resulting_bytes[second] = resulting_bytes[first];
                            }
                            for i in 0..min_amount_of_bytes as usize-res_bytes_length{
                                resulting_bytes[i] = 0;
                            }
                        }

                        if sign == Sign::Minus{
                            resulting_bytes[0] = resulting_bytes[0]|0b10000000;
                        }else{
                            resulting_bytes[0] = resulting_bytes[0]&0b01111111;
                        }
                        
                        if res_bytes_length > min_amount_of_bytes as usize{
                            for i in (res_bytes_length-min_amount_of_bytes as usize..res_bytes_length).rev(){
                                block.push_front(resulting_bytes[i]);
                            }
                        }else{
                            for byte in resulting_bytes.iter().rev(){
                                block.push_front(*byte);
                            }
                        }
                        
                    }
                    5 => {
                        // int subtraction back
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;
                        let bytes_to_pop:u16 = N/4;

                        instruction_index += 2;
                
                        let mut min_amount_of_bytes = N/8;
                        if min_amount_of_bytes == 0
                                || min_amount_of_bytes%8 != 0{
                            min_amount_of_bytes += 1;
                        }

                        let mut poped_bytes:Vec<u8> = Vec::with_capacity(bytes_to_pop as usize);

                        for i in 0..bytes_to_pop{
                            poped_bytes.push(block.pop_back().unwrap());
                        }
                        
                        poped_bytes.reverse();

                        let mut first_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(min_amount_of_bytes as usize);

                        let mut counter = N;
                        let mut index = 0;
                        while counter >= 8{
                            first_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }
                        if counter > 0{
                            first_operand.push(poped_bytes[index]&0xf0);
                            second_operand.push(poped_bytes[index]&0x0f);
                            index += 1;
                            counter = N - 4;
                        }else{
                            counter = N;
                        }

                        while counter >= 8{
                            second_operand.push(poped_bytes[index]);
                            counter -= 8;
                            index += 1;
                        }
                        shift_4_bits_left_stricted(&mut second_operand);

                        let mut first_operand_big:BigInt;
                        let mut second_operand_big:BigInt;

                        if first_operand[0]&0b10000000 != 0{
                            first_operand[0] = first_operand[0]&0b01111111;
                            first_operand_big = BigInt::from_bytes_be(Sign::Minus, &first_operand[..]);
                        }
                        else{
                            first_operand_big = BigInt::from_bytes_be(Sign::Plus, &first_operand[..]);
                        }

                        if second_operand[0]&0b10000000 != 0{
                            second_operand[0] = second_operand[0]&0b01111111;
                            second_operand_big = BigInt::from_bytes_be(Sign::Minus, &second_operand[..]);
                        }
                        else{
                            second_operand_big = BigInt::from_bytes_be(Sign::Plus, &second_operand[..]);
                        }

                        let result = first_operand_big - second_operand_big;

                        let (sign,mut resulting_bytes) = result.to_bytes_be();

                        let res_bytes_length = resulting_bytes.len();
                        
                        for i in res_bytes_length..min_amount_of_bytes as usize{
                            resulting_bytes.push(0);
                        }

                        if res_bytes_length < min_amount_of_bytes as usize{
                            for (first,second) in (0..res_bytes_length).rev().zip((min_amount_of_bytes as usize-res_bytes_length..min_amount_of_bytes as usize).rev()){
                                resulting_bytes[second] = resulting_bytes[first];
                            }
                            for i in 0..min_amount_of_bytes as usize-res_bytes_length{
                                resulting_bytes[i] = 0;
                            }
                        }

                        if sign == Sign::Minus{
                            resulting_bytes[0] = resulting_bytes[0]|0b10000000;
                        }else{
                            resulting_bytes[0] = resulting_bytes[0]&0b01111111;
                        }

                        if res_bytes_length > min_amount_of_bytes as usize{
                            for i in (res_bytes_length-min_amount_of_bytes as usize..res_bytes_length){
                                block.push_back(resulting_bytes[i]);
                            }
                        }else{
                            for byte in resulting_bytes.iter(){
                                block.push_back(*byte);
                            }
                        }
                        
                    }
                    6 => {
                        // int multiplication front
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;

                        instruction_index += 2;
                        
                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(block.pop_front().unwrap());
                        }
                        // println!("len: {}",block.len());
                        // println!("N: {}",N);
                        for i in 0..N{
                            second_operand.push(block.pop_front().unwrap());
                        }

                        let mut first_operand_big = BigUint::from_bytes_be(&first_operand[..]);
                        let mut second_operand_big = BigUint::from_bytes_be(&second_operand[..]);

                        let result = first_operand_big * second_operand_big;

                        let mut resulting_bytes = result.to_bytes_be();
                        
                        let res_bytes_length = resulting_bytes.len();
                        
                        for i in res_bytes_length..N as usize{
                            resulting_bytes.push(0);
                        }

                        if res_bytes_length < N as usize{
                            for (first,second) in (0..res_bytes_length).rev().zip((N as usize-res_bytes_length..N as usize).rev()){
                                resulting_bytes[second] = resulting_bytes[first];
                            }
                            for i in 0..N as usize-res_bytes_length{
                                resulting_bytes[i] = 0;
                            }
                        }

                        if res_bytes_length > N as usize{
                            for i in (res_bytes_length-N as usize..res_bytes_length).rev(){
                                block.push_front(resulting_bytes[i]);
                            }
                        }else{
                            for byte in resulting_bytes.iter().rev(){
                                block.push_front(*byte);
                            }
                        }
                        
                    }
                    7 => {
                        // int multiplication back
                        let N:u16 = ((buffer[instruction_index] as u16)<<8)+buffer[instruction_index+1] as u16;

                        instruction_index += 2;
                        
                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(block.pop_back().unwrap());
                        }     

                        for i in 0..N{
                            second_operand.push(block.pop_back().unwrap());
                        }

                        let mut first_operand_big = BigUint::from_bytes_le(&first_operand[..]);
                        let mut second_operand_big = BigUint::from_bytes_le(&second_operand[..]);

                        let result = first_operand_big * second_operand_big;

                        let mut resulting_bytes = result.to_bytes_be();
                        
                        let res_bytes_length = resulting_bytes.len();
                        
                        for i in res_bytes_length..N as usize{
                            resulting_bytes.push(0);
                        }

                        if res_bytes_length < N as usize{
                            for (first,second) in (0..res_bytes_length).rev().zip((N as usize-res_bytes_length..N as usize).rev()){
                                resulting_bytes[second] = resulting_bytes[first];
                            }
                            for i in 0..N as usize-res_bytes_length{
                                resulting_bytes[i] = 0;
                            }
                        }

                        if res_bytes_length > N as usize{
                            for i in res_bytes_length-N as usize..res_bytes_length{
                                block.push_back(resulting_bytes[i]);
                            }
                        }else{
                            for byte in resulting_bytes.iter(){
                                block.push_back(*byte);
                            }
                        }
                        
                    }
                    8|10|12|14 => {
                        // float operations front
                        let mut static_array:[u8;4] = [0,0,0,0];
                        for i in 0..4{
                            static_array[i] = block.pop_front().unwrap();
                        }
                        let first_operand:f32 = f32::from_be_bytes(static_array);

                        for i in 0..4{
                            static_array[i] = block.pop_front().unwrap();
                        }
                        let second_operand:f32 = f32::from_be_bytes(static_array);
                        let mut result:f32;

                        match instruction_opcode{
                            8 =>{result = first_operand+second_operand}
                            10 =>{result = first_operand*second_operand}
                            12 =>{result = first_operand-second_operand}
                            14 =>{result = first_operand/second_operand}
                            _ => {result=0.0}
                        }
                        
                        if !result.is_finite() || result == 0.0{
                            //overflow
                            for i in (instruction_index..instruction_index+4).rev(){
                                block.push_front(buffer[i]);
                            }  
                        }else{
                            static_array = unsafe{transmute(result)};
                            for byte in static_array.iter(){
                                block.push_front(*byte);
                            }
                        }
                        
                        instruction_index += 4;
                    }
                    9|11|13|15 => {
                        // float operations back
                        let mut static_array:[u8;4] = [0,0,0,0];
                        for i in 0..4{
                            static_array[i] = block.pop_back().unwrap();
                        }
                        let first_operand:f32 = f32::from_le_bytes(static_array);

                        for i in 0..4{
                            static_array[i] = block.pop_back().unwrap();
                        }
                        let second_operand:f32 = f32::from_le_bytes(static_array);

                        let mut result:f32;

                        match instruction_opcode{
                            9 =>{result = first_operand+second_operand}
                            11 =>{result = first_operand*second_operand}
                            13 =>{result = first_operand-second_operand}
                            15 =>{result = first_operand/second_operand}
                            _ => {result=0.0}
                        }
                        
                        if !result.is_finite() || result == 0.0{
                            //overflow
                            for i in instruction_index..instruction_index+4{
                                block.push_back(buffer[i]);
                            }  
                        }else{
                            static_array = unsafe{transmute(result)};
                            for byte in static_array.iter().rev(){
                                block.push_back(*byte);
                            }
                        }
                        
                        instruction_index += 4;
                    }
                    16|18|20|22 => {
                        // double operations front
                        let mut static_array:[u8;8] = [0,0,0,0,0,0,0,0];
                        for i in 0..8{
                            static_array[i] = block.pop_front().unwrap();
                        }
                        let first_operand:f64 = f64::from_be_bytes(static_array);

                        for i in 0..8{
                            static_array[i] = block.pop_front().unwrap();
                        }
                        let second_operand:f64 = f64::from_be_bytes(static_array);
                        let mut result:f64;

                        match instruction_opcode{
                            16 =>{result = first_operand+second_operand}
                            18 =>{result = first_operand*second_operand}
                            20 =>{result = first_operand-second_operand}
                            22 =>{result = first_operand/second_operand}
                            _ => {result=0.0}
                        }
                        
                        if !result.is_finite() || result == 0.0{
                            //overflow
                            for i in (instruction_index..instruction_index+8).rev(){
                                block.push_front(buffer[i]);
                            }  
                        }else{
                            static_array = unsafe{transmute(result)};
                            for byte in static_array.iter(){
                                block.push_front(*byte);
                            }
                        }
                        
                        instruction_index += 8;
                    }
                    17|19|21|23 => {
                        // double operations back
                        let mut static_array:[u8;8] = [0,0,0,0,0,0,0,0];
                        for i in 0..8{
                            static_array[i] = block.pop_back().unwrap();
                        }
                        let first_operand:f64 = f64::from_le_bytes(static_array);

                        for i in 0..8{
                            static_array[i] = block.pop_back().unwrap();
                        }
                        let second_operand:f64 = f64::from_le_bytes(static_array);

                        let mut result:f64;

                        match instruction_opcode{
                            17 =>{result = first_operand+second_operand}
                            19 =>{result = first_operand*second_operand}
                            21 =>{result = first_operand-second_operand}
                            23 =>{result = first_operand/second_operand}
                            _ => {result=0.0}
                        }
                        
                        if !result.is_finite() || result == 0.0{
                            //overflow
                            for i in instruction_index..instruction_index+8{
                                block.push_back(buffer[i]);
                            }  
                        }else{
                            static_array = unsafe{transmute(result)};
                            for byte in static_array.iter().rev(){
                                block.push_back(*byte);
                            }
                        }
                        
                        instruction_index += 8;
                    }
                    24|26|28 => {
                        // bitwise operations front
                        let mut N:u16 = ((block.pop_front().unwrap() as u16)<<8)+block.pop_front().unwrap() as u16;
                        N = N%MAX_POP_SIZE_BITWISE as u16;

                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(block.pop_front().unwrap());
                        }
                        
                        for i in 0..N{
                            second_operand.push(block.pop_front().unwrap());
                        }

                        for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                            match instruction_opcode{
                                24 =>{block.push_front(*first|*second)}
                                26 =>{block.push_front(*first&*second)}
                                28 =>{block.push_front(*first^*second)}
                                _=>{}
                            }
                        }
                        
                    }
                    25|27|29 => {
                        let mut N:u16 = block.pop_back().unwrap() as u16+((block.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(block.pop_back().unwrap());
                        }
                        
                        for i in 0..N{
                            second_operand.push(block.pop_back().unwrap());
                        }
                        
                        for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                            match instruction_opcode{
                                25 =>{block.push_back(*first|*second)}
                                27 =>{block.push_back(*first&*second)}
                                29 =>{block.push_back(*first^*second)}
                                _=>{}
                            }
                        }
                        
                    }
                    30 => {
                        //pop front biwise not
                        let mut N:u16 = ((block.pop_front().unwrap() as u16)<<8)+block.pop_front().unwrap() as u16;
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            operand.push(block.pop_front().unwrap());
                        }
                        
                        for byte in operand.iter().rev(){
                            block.push_front(!*byte);
                        }
                    
                    }
                    31 => {
                        //pop back bitwise not
                        let mut N:u16 = block.pop_back().unwrap() as u16+((block.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            operand.push(block.pop_back().unwrap());
                        }
                        
                        for byte in operand.iter().rev(){
                            block.push_back(!*byte);
                        }
                    
                    }
                    32 => {
                        // pop front shift left
                        let mut N:u16 = ((block.pop_front().unwrap() as u16)<<8)+block.pop_front().unwrap() as u16;
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(block.pop_front().unwrap());
                        }

                        if S < 8{
                            let mask:u8 = (2<<(S-1))-1; 
                            let shift_left_offset = 8 - S;
                            let mut carry = 0;
                            for byte in operand.iter_mut(){
                                let c = (*byte&mask)<<shift_left_offset;
                                *byte = *byte>>S;
                                *byte |= carry;
                                carry = c;
                            }
                        }else{
                            let mut num = BigUint::from_bytes_be(&operand);
                            num = num<<S as usize;
                            operand = num.to_bytes_le();
                        }
                        
                        for byte in operand.iter(){
                            block.push_front(*byte);
                        }
                    }
                    33 => {
                        // pop back shift left
                        let mut N:u16 = block.pop_back().unwrap() as u16+((block.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(block.pop_back().unwrap());
                        }

                        let mut num = BigUint::from_bytes_le(&operand);
                        num = num<<S as usize;
                        operand = num.to_bytes_be();

                        for byte in operand.iter(){
                            block.push_back(*byte);
                        }
                    }
                    34 => {
                        // pop front shift right
                        let mut N:u16 = ((block.pop_front().unwrap() as u16)<<8)+block.pop_front().unwrap() as u16;
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(block.pop_front().unwrap());
                        }

                        let mut num = BigUint::from_bytes_be(&operand);
                        num = num>>S as usize;
                        operand = num.to_bytes_le();
                        
                        for byte in operand.iter(){
                            block.push_front(*byte);
                        }

                    }
                    35 => {
                        // pop back shift right
                        let mut N:u16 = block.pop_back().unwrap() as u16+((block.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%MAX_POP_SIZE_BITWISE as u16;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(block.pop_back().unwrap());
                        } 

                        let mut num = BigUint::from_bytes_le(&operand);
                        num = num >> S as usize;
                        operand = num.to_bytes_be();
                        
                        for byte in operand.iter(){
                            block.push_back(*byte);
                        }
                    }
                    36 => {
                        // random address from front, sum
                        let mut address:u64 = 0;
                        for i in (0..8).rev(){
                            let mut buf_num:u64 = block.pop_front().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(block.len()-8)) as u64;
                        
                        let mut first_operand:u32 = 0;
                        first_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= block[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= block[address as usize] as u32;

                        let result:u32 = first_operand+second_operand;
                        
                        block.push_front((result&0x000000ff) as u8);
                        block.push_front((result&0x0000ff00) as u8);
                        block.push_front((result&0x00ff0000) as u8);
                        block.push_front((result&0xff000000) as u8);

                    }
                    37 => {
                        //random address from back, sum
                        let mut address:u64 = 0;
                        for i in 0..8{
                            let mut buf_num:u64 = block.pop_back().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(block.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= block[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= block[address as usize] as u32;

                        let result:u32 = first_operand+second_operand;
                        
                        block.push_back((result&0xff000000) as u8);
                        block.push_back((result&0x00ff0000) as u8);
                        block.push_back((result&0x0000ff00) as u8);
                        block.push_back((result&0x000000ff) as u8);
                    }
                    38 => {
                        //random address from front, sub
                        let mut address:u64 = 0;
                        for i in (0..8).rev(){
                            let mut buf_num:u64 = block.pop_front().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(block.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= block[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= block[address as usize] as u32;

                        let result:u32 = unsafe{transmute(first_operand as i32 - second_operand as i32)};
                        
                        block.push_front((result&0x000000ff) as u8);
                        block.push_front((result&0x0000ff00) as u8);
                        block.push_front((result&0x00ff0000) as u8);
                        block.push_front((result&0xff000000) as u8);
                    }
                    39 => {
                        //random address from back, sub
                        let mut address:u64 = 0;
                        for i in 0..8{
                            let mut buf_num:u64 = block.pop_back().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(block.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= block[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (block[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (block[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= block[address as usize] as u32;

                        let result:u32 = unsafe{transmute(first_operand as i32 - second_operand as i32)};
                        
                        block.push_back((result&0xff000000) as u8);
                        block.push_back((result&0x00ff0000) as u8);
                        block.push_back((result&0x0000ff00) as u8);
                        block.push_back((result&0x000000ff) as u8);
                    }
                    40 => {
                        // aes128 encrypt front
                        let mut key_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes128::new(&key);
                        cipher.encrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    41 => {
                        // aes128 decrypt front
                        let mut key_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes128::new(&key);
                        cipher.decrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    42 => {
                        // aes192 encrypt front
                        let mut key_raw:[u8;24] = [0u8;24];
                        for i in 0..24{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes192::new(&key);
                        cipher.encrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    43 => {
                        // aes192 decrypt front
                        let mut key_raw:[u8;24] = [0u8;24];
                        for i in 0..24{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes192::new(&key);
                        cipher.decrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    44 => {
                        // aes256 encrypt front
                        let mut key_raw:[u8;32] = [0u8;32];
                        for i in 0..32{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes256::new(&key);
                        cipher.encrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    45 => {
                        // aes256 decrypt front
                        let mut key_raw:[u8;32] = [0u8;32];
                        for i in 0..32{
                            key_raw[i] = block.pop_front().unwrap();
                        }
                        let key = GenericArray::from(key_raw);
                        
                        let mut block_raw:[u8;16] = [0u8;16];
                        for i in 0..16{
                            key_raw[i] = block.pop_back().unwrap();
                        }

                        let mut block_aes = GenericArray::from(block_raw);
                        
                        let cipher = Aes256::new(&key);
                        cipher.decrypt_block(&mut block_aes);

                        for num in block_aes.iter(){
                            block.push_front(*num);
                        }
                    }
                    46 => {
                        // pop 4 bytes from back push to front

                        for _ in 0..4{
                            let num  = block.pop_back().unwrap();
                            block.push_front(num);
                        }
                    }
                    47 => {
                        // pop 4 bytes from back, push to final state
                        let mut value:u32 = 0;

                        value |= block.pop_back().unwrap() as u32;
                        value |= (block.pop_back().unwrap() as u32)<<8;
                        value |= (block.pop_back().unwrap() as u32)<<16;
                        value |= (block.pop_back().unwrap() as u32)<<24;

                        self.final_state.push(value);
                    }
                    48 => {
                        let mut value:u32 = 0;

                        value |= (block.pop_front().unwrap() as u32)<<24;
                        value |= (block.pop_front().unwrap() as u32)<<16;
                        value |= (block.pop_front().unwrap() as u32)<<8;
                        value |= block.pop_front().unwrap() as u32;

                        self.final_state.push(value);
                    }
                    _ =>{
                        // println!("Unknown instruction {} 
                        //             Last index: {}
                        //             Instruction pos: {}",
                        //             instruction_opcode,
                        //             instruction_index,
                        //             instruction_counter);
                        return Err("Unknown instruction");
                    }
                }
            }
            
            for byte in block.iter(){
                stack[block_start] = *byte;
                block_start += 1;
            }

            stack.drain(block_start..block_end);

            if block_end >= stack.len(){
                block_start = 0;
                println!("End reached");
            }
            
            //println!("{:?}",stack.len());

        }
        

        for i in 0..(stack.len()>>2){
            let mut value:u32 = 0;
            let index = i*4;
            value |= (stack[i] as u32)<<24;
            value |= (stack[i+1] as u32)<<16;
            value |= (stack[i+2] as u32)<<8;
            value |= stack[i+3] as u32;

            self.final_state.push(value);            
        }

        let rem = stack.len() % 4;
        if rem != 0{
            let mut result:u32 = 0;
            match rem{
                1 => {
                    result |= stack[stack.len()-1] as u32;
                }
                2 => {
                    result |= stack[stack.len()-1] as u32;
                    result |= (stack[stack.len()-2] as u32) << 8;
                }
                3 => {
                    result |= stack[stack.len()-1] as u32;
                    result |= (stack[stack.len()-2] as u32) << 8;
                    result |= (stack[stack.len()-3] as u32) << 16;
                }
                _ => {}
            }
            self.final_state.push(result);
        }  

        return Ok(());
    }

    pub fn hex_digest(&self)->Vec<u8>{
        let mut to_return:Vec<u8> = Vec::with_capacity(self.final_state.len()<<1);
        for number in self.final_state.iter(){
            to_return.push(lookup_table[((number&0xf0000000)>>28) as usize]);
            to_return.push(lookup_table[((number&0x0f000000)>>24) as usize]);
            to_return.push(lookup_table[((number&0x00f00000)>>20) as usize]);
            to_return.push(lookup_table[((number&0x000f0000)>>16) as usize]);
            to_return.push(lookup_table[((number&0x0000f000)>>12) as usize]);
            to_return.push(lookup_table[((number&0x00000f00)>>8) as usize]);
            to_return.push(lookup_table[((number&0x000000f0)>>4) as usize]);
            to_return.push(lookup_table[(number&0x0000000f) as usize]);
        }
        return to_return;
    }
    pub fn digest(&self)->Vec<u8>{
        let mut to_return:Vec<u8> = Vec::with_capacity(self.final_state.len()<<2);
        for number in self.final_state.iter(){
            to_return.push(((number&0xff000000)>>24) as u8);
            to_return.push(((number&0x00ff0000)>>16) as u8);
            to_return.push(((number&0x0000ff00)>>8) as u8);
            to_return.push((number&0x000000ff) as u8);
        }
        return to_return;
    }
}