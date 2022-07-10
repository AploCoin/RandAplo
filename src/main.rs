mod RNG;
mod Tools;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
mod AlgorithmCreator;
mod AlgorithmProcessor;
use std::iter::FromIterator;
use std::convert::TryInto;
use sha2::{Sha256, Digest};

pub fn cast_slice_to_array(data: &[u8]) -> [u8;8]{
    return data.try_into().unwrap();
}


fn main() {    
    let mut data:[u8;20] = [0xff,0xff,0xff,0xff,0xff,
                             0xff,0xfa,0xff,0xff,0xff,
                             0xff,0xff,0xff,0xff,0xff,
                             0xff,0xff,0xff,0xff,0x00];
    let now = Instant::now();
    println!("Generating algorithm");

    let BLOCK_SIZE:usize = 209715200;

    let algorithm = AlgorithmCreator::create_algorithm(&mut data, 
                                                    BLOCK_SIZE,
                                                    1024,
                                                    4000000,
                                                    0,
                                                    15
                                                ).unwrap();
    println!("Algorithm generated seconds:{}", now.elapsed().as_secs());

    let mut vm = AlgorithmProcessor::get_VM();

    let mut stack:Vec<u8> = Vec::with_capacity(2147483648);

    for i in 0..6{
        stack.push(0xFA);
    }
    
    println!("Padding data...");
    AlgorithmProcessor::pad_data(&mut stack, 2147483648);

    println!("Executing algo");
    let now = Instant::now();
    let res = vm.execute_from_buffer(&algorithm, 
                                    &mut stack,
                                    BLOCK_SIZE,
                                    1024,
                                    1073741824);
    println!("Algorithm executed seconds:{}", now.elapsed().as_secs());

    let mut digest:Vec<u8> = vm.digest();
    
    let mut hasher = Sha256::new();
    hasher.update(digest);
    let result = hasher.finalize();
    println!("{:?}",result);
    // println!("{:?}",digest.len());
    // println!("{:X?}",digest);
    
}
