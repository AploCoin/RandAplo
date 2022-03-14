use std::io::{stdin, stdout, Read, Write};
use std::collections::VecDeque;
use std::collections::HashMap;
use num_bigint::{BigInt,Sign,BigUint};
use std::mem::transmute;
use crate::RNG::MurMur2RNG;
use std::iter::FromIterator;

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

// pub fn prepare_lookup_table(array:&[u16;42]) -> HashMap<u16,u8>{
//     let mut to_return:HashMap<u16,u8> = HashMap::with_capacity(42);

//     for i in 0..42{
//         to_return.insert(array[i],i as u8);
//     }

//     return to_return;
// }

pub fn pad_data(data:&mut VecDeque<u8>,size:u64){
    let mut initial_size:u64 = data.len() as u64;
    let mut data_vector:Vec<u8> = Vec::with_capacity(data.len());
    for byte in data.iter(){
        data_vector.push(*byte);
    }
    let mut rng = MurMur2RNG::get_generator(&mut data_vector, initial_size);
    
    //data.reserve(size as usize-data.capacity());

    let mut bytes:[u8;4];

    while initial_size <= size-4{
        bytes = unsafe{transmute(rng.generate())};
        for byte in bytes.iter().rev(){
            data.push_back(*byte);
        }
        initial_size += 4;
    }

    match size-initial_size{
        1 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push_back(bytes[0]);
        }
        2 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push_back(bytes[0]);
            data.push_back(bytes[1]);
        }
        3 => {
            bytes = unsafe{transmute(rng.generate())};
            data.push_back(bytes[0]);
            data.push_back(bytes[1]);
            data.push_back(bytes[2]);
        }
        _ => {}
    }
    // let mut value_to_put:u8 = 0;
    // while initial_size <= size{
    //     data.push_back(value_to_put);
    //     value_to_put += 1;
    //     initial_size += 1;
    // }

}

static lookup_table:[u8;16] = [b'0',b'1',b'2',b'3',
                                 b'4',b'5',b'6',b'7',
                                 b'8',b'9',b'A',b'B',
                                 b'C',b'D',b'E',b'F'];

impl VM{
    pub fn execute_from_buffer(&mut self,
                        buffer:&[u8],
                        stack:&mut VecDeque<u8>,
                        output_max_size:usize,
                        debug:bool,
                        breakpoint:usize) -> Result<(),&'static str>{
        // will change stack variable

        
        let mut instruction_opcode:u8;
        let mut in_debugging:bool;
        let bitwise_max_size:u16 = 15;

        while stack.len() > output_max_size{
            let mut instruction_index:usize = 0;
            let mut instruction_counter:usize = 0;
            println!("Stack: {}",stack.len());
            while instruction_index != buffer.len()
                    && stack.len() > output_max_size{

                instruction_opcode = buffer[instruction_index];

                instruction_index += 1;
                instruction_counter += 1;


                in_debugging = false;
                if debug && instruction_counter == breakpoint{
                    println!("Breakpoint at {} instruction",breakpoint);
                    println!("Debug Data:");
                    println!("Instruction Code: {}",instruction_opcode);
                    println!("Instruction Index: {}",instruction_index);
                    println!("Initial Stack size: {}",stack.len());
                    println!("Instruction Data:");
                    in_debugging = true;
                }

                match instruction_opcode{
                    0 => {
                        let mut N:u16 = (buffer[instruction_index] as u16) << 8;
                        N += buffer[instruction_index+1] as u16;

                        instruction_index += 2;

                        for i in 0..N{
                            stack.push_back(buffer[instruction_index]);
                            instruction_index += 1;
                        }

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                        }
                    }
                    1 => {
                        let mut N:u16 = (buffer[instruction_index] as u16) << 8;
                        N += buffer[instruction_index+1] as u16;

                        instruction_index += 2;

                        if stack.capacity()-stack.len() < N as usize{
                            stack.reserve(N as usize - stack.capacity());
                        }

                        for i in 0..N as usize{
                            stack.push_front(buffer[instruction_index+i]);
                        }
                        instruction_index += N as usize;

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                        }

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
                            poped_bytes.push(stack.pop_front().unwrap());
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

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:?}",first_operand);
                            println!("Second operand: {:?}",second_operand);
                            print!("Result: ");
                            let mut carry:u8 = 0;
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                let result:u16 = *first as u16 
                                                    + *second as u16 
                                                    + carry as u16;
                                print!("{:X}",result&0xff);
                                carry = (result>>8) as u8;
                                stack.push_front((result&0x00ff) as u8);
                            }
                            print!("\n");
                        }else{
                            let mut carry:u8 = 0;
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                let result:u16 = *first as u16 
                                                    + *second as u16 
                                                    + carry as u16;
                                carry = (result>>8) as u8;
                                stack.push_front((result&0x00ff) as u8);
                            }
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
                            poped_bytes.push(stack.pop_back().unwrap());
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


                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:?}",first_operand);
                            println!("Second operand: {:?}",second_operand);
                            print!("Result: ");
                            let mut carry:u8 = 0;
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                let result:u16 = *first as u16 
                                                    + *second as u16 
                                                    + carry as u16;
                                print!("{:X}",result&0xff);
                                carry = (result>>8) as u8;
                                resulting_bytes.push((result&0x00ff) as u8);
                                //stack.push_back((result&0xff) as u8);
                            }
                            print!("\n");
                        }else{
                            let mut carry:u8 = 0;
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                let result:u16 = *first as u16 
                                                    + *second as u16 
                                                    + carry as u16;
                                carry = (result>>8) as u8;
                                resulting_bytes.push((result&0x00ff) as u8);
                                //stack.push_back((result&0xff) as u8);
                            }
                        }
                        for byte in resulting_bytes.iter().rev(){
                            stack.push_back(*byte);
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
                            poped_bytes.push(stack.pop_front().unwrap());
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

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second operand: {:X?}",second_operand);
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

                            println!("First Operand BigInt: {}",first_operand_big);
                            println!("Second Operand BigInt: {}",second_operand_big);

                            let result = first_operand_big - second_operand_big;

                            println!("Result BigInt: {}",result);

                            let (sign,mut resulting_bytes) = result.to_bytes_be();

                            println!("Sign: {:?}",sign);
                            
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
                            println!("Resulting bytes: {:X?}",resulting_bytes);
                            
                            if res_bytes_length > min_amount_of_bytes as usize{
                                for i in (res_bytes_length-min_amount_of_bytes as usize..res_bytes_length).rev(){
                                    stack.push_front(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter().rev(){
                                    stack.push_front(*byte);
                                }
                            }

                        }else{
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
                                    stack.push_front(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter().rev(){
                                    stack.push_front(*byte);
                                }
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
                            poped_bytes.push(stack.pop_back().unwrap());
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

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second operand: {:X?}",second_operand);
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

                            println!("First Operand BigInt: {}",first_operand_big);
                            println!("Second Operand BigInt: {}",second_operand_big);

                            let result = first_operand_big - second_operand_big;

                            println!("Result BigInt: {}",result);

                            let (sign,mut resulting_bytes) = result.to_bytes_be();

                            println!("Sign: {:?}",sign);
                            
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
                            println!("Resulting bytes: {:X?}",resulting_bytes);
                            
                            if res_bytes_length > min_amount_of_bytes as usize{
                                for i in (res_bytes_length-min_amount_of_bytes as usize..res_bytes_length){
                                    stack.push_back(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter(){
                                    stack.push_back(*byte);
                                }
                            }

                        }else{
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
                                    stack.push_back(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter(){
                                    stack.push_back(*byte);
                                }
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
                            first_operand.push(stack.pop_front().unwrap());
                        }
                        // println!("len: {}",stack.len());
                        // println!("N: {}",N);
                        for i in 0..N{
                            second_operand.push(stack.pop_front().unwrap());
                        }

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second operand: {:X?}",second_operand);

                            let mut first_operand_big = BigUint::from_bytes_be(&first_operand[..]);
                            let mut second_operand_big = BigUint::from_bytes_be(&second_operand[..]);

                            println!("First Operand BigInt: {}",first_operand_big);
                            println!("Second Operand BigInt: {}",second_operand_big);

                            let result = first_operand_big * second_operand_big;

                            println!("Result BigInt: {}",result);

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

                            println!("Resulting bytes: {:X?}",resulting_bytes);

                            if res_bytes_length > N as usize{
                                for i in (res_bytes_length-N as usize..res_bytes_length).rev(){
                                    stack.push_front(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter().rev(){
                                    stack.push_front(*byte);
                                }
                            }

                        }else{
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
                                    stack.push_front(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter().rev(){
                                    stack.push_front(*byte);
                                }
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
                            first_operand.push(stack.pop_back().unwrap());
                        }     

                        for i in 0..N{
                            second_operand.push(stack.pop_back().unwrap());
                        }

                        if in_debugging{
                            println!("N: {}",N);
                            println!("Instruction index: {}",instruction_index);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second operand: {:X?}",second_operand);

                            let mut first_operand_big = BigUint::from_bytes_le(&first_operand[..]);
                            let mut second_operand_big = BigUint::from_bytes_le(&second_operand[..]);

                            println!("First Operand BigInt: {}",first_operand_big);
                            println!("Second Operand BigInt: {}",second_operand_big);

                            let result = first_operand_big * second_operand_big;

                            println!("Result BigInt: {}",result);

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

                            println!("Resulting bytes: {:X?}",resulting_bytes);

                            if res_bytes_length > N as usize{
                                for i in res_bytes_length-N as usize..res_bytes_length{
                                    stack.push_back(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter(){
                                    stack.push_back(*byte);
                                }
                            }

                        }else{
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
                                    stack.push_back(resulting_bytes[i]);
                                }
                            }else{
                                for byte in resulting_bytes.iter(){
                                    stack.push_back(*byte);
                                }
                            }
                        }
                    }
                    8|10|12|14 => {
                        // float operations front
                        let mut static_array:[u8;4] = [0,0,0,0];
                        for i in 0..4{
                            static_array[i] = stack.pop_front().unwrap();
                        }
                        let first_operand:f32 = f32::from_be_bytes(static_array);

                        for i in 0..4{
                            static_array[i] = stack.pop_front().unwrap();
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
                        
                        if in_debugging{
                            println!("First Operand: {}",first_operand);
                            println!("Second Operand: {}",second_operand);
                            println!("Result: {:?}",result);
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                println!("Overflow encountered");
                                print!("Adding bytes to stack: ");
                                for i in (instruction_index..instruction_index+4).rev(){
                                    stack.push_front(buffer[i]);
                                    print!("{:X}",buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter(){
                                    stack.push_front(*byte);
                                }
                            }
                        }else{
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                for i in (instruction_index..instruction_index+4).rev(){
                                    stack.push_front(buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter(){
                                    stack.push_front(*byte);
                                }
                            }
                        }
                        instruction_index += 4;
                    }
                    9|11|13|15 => {
                        // float operations back
                        let mut static_array:[u8;4] = [0,0,0,0];
                        for i in 0..4{
                            static_array[i] = stack.pop_back().unwrap();
                        }
                        let first_operand:f32 = f32::from_le_bytes(static_array);

                        for i in 0..4{
                            static_array[i] = stack.pop_back().unwrap();
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
                        
                        if in_debugging{
                            println!("First Operand: {}",first_operand);
                            println!("Second Operand: {}",second_operand);
                            println!("Result: {:?}",result);
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                println!("Overflow encountered");
                                print!("Adding bytes to stack: ");
                                for i in instruction_index..instruction_index+4{
                                    stack.push_back(buffer[i]);
                                    print!("{:X}",buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter().rev(){
                                    stack.push_back(*byte);
                                }
                            }
                        }else{
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                for i in instruction_index..instruction_index+4{
                                    stack.push_back(buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter().rev(){
                                    stack.push_back(*byte);
                                }
                            }
                        }
                        instruction_index += 4;
                    }
                    16|18|20|22 => {
                        // double operations front
                        let mut static_array:[u8;8] = [0,0,0,0,0,0,0,0];
                        for i in 0..8{
                            static_array[i] = stack.pop_front().unwrap();
                        }
                        let first_operand:f64 = f64::from_be_bytes(static_array);

                        for i in 0..8{
                            static_array[i] = stack.pop_front().unwrap();
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
                        
                        if in_debugging{
                            println!("First Operand: {}",first_operand);
                            println!("Second Operand: {}",second_operand);
                            println!("Result: {:?}",result);
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                println!("Overflow encountered");
                                print!("Adding bytes to stack: ");
                                for i in (instruction_index..instruction_index+8).rev(){
                                    stack.push_front(buffer[i]);
                                    print!("{:X}",buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter(){
                                    stack.push_front(*byte);
                                }
                            }
                        }else{
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                for i in (instruction_index..instruction_index+8).rev(){
                                    stack.push_front(buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter(){
                                    stack.push_front(*byte);
                                }
                            }
                        }
                        instruction_index += 8;
                    }
                    17|19|21|23 => {
                        // double operations back
                        let mut static_array:[u8;8] = [0,0,0,0,0,0,0,0];
                        for i in 0..8{
                            static_array[i] = stack.pop_back().unwrap();
                        }
                        let first_operand:f64 = f64::from_le_bytes(static_array);

                        for i in 0..8{
                            static_array[i] = stack.pop_back().unwrap();
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
                        
                        if in_debugging{
                            println!("First Operand: {}",first_operand);
                            println!("Second Operand: {}",second_operand);
                            println!("Result: {:?}",result);
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                println!("Overflow encountered");
                                print!("Adding bytes to stack: ");
                                for i in instruction_index..instruction_index+8{
                                    stack.push_back(buffer[i]);
                                    print!("{:X}",buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter().rev(){
                                    stack.push_back(*byte);
                                }
                            }
                        }else{
                            if !result.is_finite() || result == 0.0{
                                //overflow
                                for i in instruction_index..instruction_index+8{
                                    stack.push_back(buffer[i]);
                                }  
                            }else{
                                static_array = unsafe{transmute(result)};
                                for byte in static_array.iter().rev(){
                                    stack.push_back(*byte);
                                }
                            }
                        }
                        instruction_index += 8;
                    }
                    24|26|28 => {
                        // bitwise operations front
                        let mut N:u16 = ((stack.pop_front().unwrap() as u16)<<8)+stack.pop_front().unwrap() as u16;
                        N = 1 + N%bitwise_max_size;

                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(stack.pop_front().unwrap());
                        }
                        
                        for i in 0..N{
                            second_operand.push(stack.pop_front().unwrap());
                        }
                        if in_debugging{
                            println!("N: {}",N);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second Operand: {:X?}",second_operand);
                            print!("Result: ");
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                match instruction_opcode{
                                    24 =>{stack.push_front(*first|*second);
                                            print!("{}",*first|*second);}
                                    26 =>{stack.push_front(*first&*second);
                                            print!("{}",*first&*second);}
                                    28 =>{stack.push_front(*first^*second);
                                            print!("{}",*first^*second);}
                                    _=>{}
                                }
                            }
                            print!("\n");
                        }
                        else{
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                match instruction_opcode{
                                    24 =>{stack.push_front(*first|*second)}
                                    26 =>{stack.push_front(*first&*second)}
                                    28 =>{stack.push_front(*first^*second)}
                                    _=>{}
                                }
                            }
                        }
                    }
                    25|27|29 => {
                        let mut N:u16 = stack.pop_back().unwrap() as u16+((stack.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%bitwise_max_size;

                        let mut first_operand:Vec<u8> = Vec::with_capacity(N as usize);
                        let mut second_operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            first_operand.push(stack.pop_back().unwrap());
                        }
                        
                        for i in 0..N{
                            second_operand.push(stack.pop_back().unwrap());
                        }
                        if in_debugging{
                            println!("N: {}",N);
                            println!("First Operand: {:X?}",first_operand);
                            println!("Second Operand: {:X?}",second_operand);
                            print!("Result: ");
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                match instruction_opcode{
                                    25 =>{stack.push_back(*first|*second);
                                            print!("{}",*first|*second);}
                                    27 =>{stack.push_back(*first&*second);
                                            print!("{}",*first&*second);}
                                    29 =>{stack.push_back(*first^*second);
                                            print!("{}",*first^*second);}
                                    _=>{}
                                }
                            }
                            print!("\n");
                        }
                        else{
                            for (first,second) in first_operand.iter().rev().zip(second_operand.iter().rev()){
                                match instruction_opcode{
                                    25 =>{stack.push_back(*first|*second)}
                                    27 =>{stack.push_back(*first&*second)}
                                    29 =>{stack.push_back(*first^*second)}
                                    _=>{}
                                }
                            }
                        }
                    }
                    30 => {
                        //pop front biwise not
                        let mut N:u16 = ((stack.pop_front().unwrap() as u16)<<8)+stack.pop_front().unwrap() as u16;
                        N = 1 + N%bitwise_max_size;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            operand.push(stack.pop_front().unwrap());
                        }
                        
                        if in_debugging{
                            println!("N: {}",N);
                            println!("Operand: {:X?}",operand);
                            print!("Result: ");
                            for byte in operand.iter().rev(){
                                stack.push_front(!*byte);
                                print!("{}",!*byte);
                            }
                            print!("\n");
                        }
                        else{
                            for byte in operand.iter().rev(){
                                stack.push_front(!*byte);
                            }
                        }
                    }
                    31 => {
                        //pop back bitwise not
                        let mut N:u16 = stack.pop_back().unwrap() as u16+((stack.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%bitwise_max_size;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);

                        for i in 0..N{
                            operand.push(stack.pop_back().unwrap());
                        }
                        
                        if in_debugging{
                            println!("N: {}",N);
                            println!("Operand: {:X?}",operand);
                            print!("Result: ");
                            for byte in operand.iter().rev(){
                                stack.push_back(!*byte);
                                print!("{}",!*byte);
                            }
                            print!("\n");
                        }
                        else{
                            for byte in operand.iter().rev(){
                                stack.push_back(!*byte);
                            }
                        }
                    }
                    32 => {
                        // pop front shift left
                        let mut N:u16 = ((stack.pop_front().unwrap() as u16)<<8)+stack.pop_front().unwrap() as u16;
                        N = 1 + N%bitwise_max_size;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(stack.pop_front().unwrap());
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
                        if in_debugging{
                            println!("N: {}",N);
                            println!("S: {}",S);
                            println!("result: {:X?}",operand);
                        }
                        for byte in operand.iter(){
                            stack.push_front(*byte);
                        }
                    }
                    33 => {
                        // pop back shift left
                        let mut N:u16 = stack.pop_back().unwrap() as u16+((stack.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%bitwise_max_size;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(stack.pop_back().unwrap());
                        }

                        let mut num = BigUint::from_bytes_le(&operand);
                        num = num<<S as usize;
                        operand = num.to_bytes_be();
                        if in_debugging{
                            println!("N: {}",N);
                            println!("S: {}",S);
                            println!("result: {:X?}",operand);
                        }
                        for byte in operand.iter(){
                            stack.push_back(*byte);
                        }
                    }
                    34 => {
                        // pop front shift right
                        let mut N:u16 = ((stack.pop_front().unwrap() as u16)<<8)+stack.pop_front().unwrap() as u16;
                        N = 1 + N%bitwise_max_size;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(stack.pop_front().unwrap());
                        }

                        let mut num = BigUint::from_bytes_be(&operand);
                        num = num>>S as usize;
                        operand = num.to_bytes_le();
                        if in_debugging{
                            println!("N: {}",N);
                            println!("S: {}",S);
                            println!("result: {:X?}",operand);
                        }
                        for byte in operand.iter(){
                            stack.push_front(*byte);
                        }

                    }
                    35 => {
                        // pop back shift right
                        let mut N:u16 = stack.pop_back().unwrap() as u16+((stack.pop_back().unwrap() as u16)<<8);
                        N = 1 + N%bitwise_max_size;

                        let mut S = buffer[instruction_index];
                        instruction_index += 1;

                        let mut operand:Vec<u8> = Vec::with_capacity(N as usize);
                        for i in 0..N{
                            operand.push(stack.pop_back().unwrap());
                        } 

                        let mut num = BigUint::from_bytes_le(&operand);
                        num = num >> S as usize;
                        operand = num.to_bytes_be();
                        if in_debugging{
                            println!("N: {}",N);
                            println!("S: {}",S);
                            println!("result: {:X?}",operand);
                        }
                        for byte in operand.iter(){
                            stack.push_back(*byte);
                        }
                    }
                    36 => {
                        // random address from front, sum
                        let mut address:u64 = 0;
                        for i in (0..8).rev(){
                            let mut buf_num:u64 = stack.pop_front().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(stack.len()-8)) as u64;
                        
                        let mut first_operand:u32 = 0;
                        first_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= stack[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= stack[address as usize] as u32;

                        let result:u32 = first_operand+second_operand;
                        if in_debugging{
                            println!("address: {}",address);
                            println!("first operand: {}",first_operand);
                            println!("second operand: {}",second_operand);
                            println!("result: {:X?}",result);
                        }
                        stack.push_front((result&0x000000ff) as u8);
                        stack.push_front((result&0x0000ff00) as u8);
                        stack.push_front((result&0x00ff0000) as u8);
                        stack.push_front((result&0xff000000) as u8);

                    }
                    37 => {
                        //random address from back, sum
                        let mut address:u64 = 0;
                        for i in 0..8{
                            let mut buf_num:u64 = stack.pop_back().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(stack.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= stack[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= stack[address as usize] as u32;

                        let result:u32 = first_operand+second_operand;
                        if in_debugging{
                            println!("address: {}",address);
                            println!("first operand: {}",first_operand);
                            println!("second operand: {}",second_operand);
                            println!("result: {:X?}",result);
                        }
                        stack.push_back((result&0xff000000) as u8);
                        stack.push_back((result&0x00ff0000) as u8);
                        stack.push_back((result&0x0000ff00) as u8);
                        stack.push_back((result&0x000000ff) as u8);
                    }
                    38 => {
                        //random address from front, sub
                        let mut address:u64 = 0;
                        for i in (0..8).rev(){
                            let mut buf_num:u64 = stack.pop_front().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(stack.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= stack[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= stack[address as usize] as u32;

                        let result:u32 = unsafe{transmute(first_operand as i32 - second_operand as i32)};
                        if in_debugging{
                            println!("address: {}",address);
                            println!("first operand: {}",first_operand);
                            println!("second operand: {}",second_operand);
                            println!("result: {:X?}",result);
                        }
                        stack.push_front((result&0x000000ff) as u8);
                        stack.push_front((result&0x0000ff00) as u8);
                        stack.push_front((result&0x00ff0000) as u8);
                        stack.push_front((result&0xff000000) as u8);
                    }
                    39 => {
                        //random address from back, sub
                        let mut address:u64 = 0;
                        for i in 0..8{
                            let mut buf_num:u64 = stack.pop_back().unwrap() as u64;
                            buf_num = buf_num << (8*i);
                            address |= buf_num;
                        }

                        address = (address as usize%(stack.len()-8)) as u64;

                        let mut first_operand:u32 = 0;
                        first_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        first_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        first_operand |= stack[address as usize] as u32;
                        address += 1;

                        let mut second_operand:u32 = 0;
                        second_operand |= (stack[address as usize] as u32)<<24;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<16;
                        address += 1;
                        second_operand |= (stack[address as usize] as u32)<<8;
                        address += 1;
                        second_operand |= stack[address as usize] as u32;

                        let result:u32 = unsafe{transmute(first_operand as i32 - second_operand as i32)};
                        if in_debugging{
                            println!("address: {}",address);
                            println!("first operand: {}",first_operand);
                            println!("second operand: {}",second_operand);
                            println!("result: {:X?}",result);
                        }
                        stack.push_back((result&0xff000000) as u8);
                        stack.push_back((result&0x00ff0000) as u8);
                        stack.push_back((result&0x0000ff00) as u8);
                        stack.push_back((result&0x000000ff) as u8);
                    }
                    40 => {
                        // pop 4 bytes from back, push to final state
                        let mut value:u32 = 0;

                        value |= stack.pop_back().unwrap() as u32;
                        value |= (stack.pop_back().unwrap() as u32)<<8;
                        value |= (stack.pop_back().unwrap() as u32)<<16;
                        value |= (stack.pop_back().unwrap() as u32)<<24;

                        self.final_state.push(value);
                    }
                    41 => {
                        let mut value:u32 = 0;

                        value |= (stack.pop_front().unwrap() as u32)<<24;
                        value |= (stack.pop_front().unwrap() as u32)<<16;
                        value |= (stack.pop_front().unwrap() as u32)<<8;
                        value |= stack.pop_front().unwrap() as u32;

                        self.final_state.push(value);
                    }
                    _ =>{
                        println!("Unknown instruction {} 
                                    Last index: {}
                                    Instruction pos: {}",
                                    instruction_opcode,
                                    instruction_index,
                                    instruction_counter);
                        return Err("Unknown instruction");
                    }
                }
                if in_debugging{
                    pause();
                }
            }
        }

        for i in 0..(stack.len()>>2){
            let mut value:u32 = 0;

            value |= (stack.pop_front().unwrap() as u32)<<24;
            value |= (stack.pop_front().unwrap() as u32)<<16;
            value |= (stack.pop_front().unwrap() as u32)<<8;
            value |= stack.pop_front().unwrap() as u32;

            self.final_state.push(value);            
        }

        if stack.len() != 0{
            let mut result:u32 = 0;
            match stack.len(){
                1 => {
                    result |= stack.pop_front().unwrap() as u32;
                }
                2 => {
                    result |= (stack.pop_front().unwrap() as u32) << 8;
                    result |= stack.pop_front().unwrap() as u32;                
                }
                3 => {
                    result |= (stack.pop_front().unwrap() as u32) << 16;
                    result |= (stack.pop_front().unwrap() as u32) << 8;
                    result |= stack.pop_front().unwrap() as u32;
                }
                _ =>{}
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