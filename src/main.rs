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
                             0xff,0xff,0xff,0xff,0xff,
                             0xff,0xff,0xff,0xff,0xff,
                             0xff,0xff,0xff,0xff,0xff];
    let now = Instant::now();
    println!("Generating algorithm");
    //let instructions_lookup = AlgorithmCreator::generate_lookup_table(&mut data.clone());
    //println!("{:?}",instructions_lookup.len());
    //println!("{:?}",instructions_lookup);
    let algorithm = AlgorithmCreator::create_algorithm(&mut data, 
                                                    2147483648,
                                                    500,
                                                    10000000
                                                ).unwrap();
    println!("Algorithm generated seconds:{}", now.elapsed().as_secs());
    println!("{:?}",algorithm.len());

    //let instructions_lookup_table = AlgorithmProcessor::prepare_lookup_table(&instructions_lookup);

    let bytes_size:[u8;8] = algorithm[0..8].try_into().unwrap();
    let block_size = u64::from_be_bytes(bytes_size);

    let mut vm = AlgorithmProcessor::get_VM();

    let mut stack:VecDeque<u8> = VecDeque::with_capacity((block_size/4) as usize);

    for i in 0..6{
        stack.push_back(0xFF);
    }
    stack[0] = 0xff;
    
    println!("Padding data...");
    AlgorithmProcessor::pad_data(&mut stack, block_size);

    //let mut stack_to_process = VecDeque::from_iter(stack);

    println!("Executing algo");
    let now = Instant::now();
    let res = vm.execute_from_buffer(&algorithm[8..], 
                                    &mut stack,
                                    1024,
                                    false,
                                    0);
    println!("Algorithm executed seconds:{}", now.elapsed().as_secs());
    println!("{:?}",res);

    let mut digest:Vec<u8> = vm.digest();
    
    let mut hasher = Sha256::new();
    hasher.update(digest);
    let result = hasher.finalize();
    println!("{:?}",result);
    // println!("{:?}",digest.len());
    // println!("{:X?}",digest);
    
}
