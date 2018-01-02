#![allow(unused)]
extern crate scoped_threadpool;

use std::time::Instant;
use std::f64::consts;

const TOW_PI_SQARE      : f64 = consts::PI *  consts::PI * 2.0;

#[inline(always)]
fn calc(block: &mut [f64], size: usize, part: usize, start: usize) {
    
    let end = start * part;
    let begin = if start > 0 {0} else {1}; 
    for i in 1..part{
        let fpisin_i = f64::sin((i + end+start) as f64) * TOW_PI_SQARE;

        for j in 1..size{

            unsafe{
                *(*block).get_unchecked_mut(i * size + j) = (fpisin_i * f64::sin(j as f64) + 
                    ((0.25 * *(*block).get_unchecked((i-1) * size + j)
                    * *(*block).get_unchecked(i * size + j-1)
                    * *(*block).get_unchecked(i * size + j+1)
                    * *(*block).get_unchecked((i+1) * size + j)))).abs();
            }
        }
    }
}

#[inline(always)]
fn calc2(matrix: &[f64], block: &mut [f64], size: usize, part: usize, start: usize, m: usize) {
    let add = m*(size+1)*(size+1);
    let end = start * part;
    let begin = if start > 0 {0} else {1}; 
    for i in 1..part{
        let fpisin_i = f64::sin((i + end+start) as f64) * TOW_PI_SQARE; 
        for j in 1..size{

            unsafe{
                *(*block).get_unchecked_mut(i * size + j) = (fpisin_i * f64::sin(j as f64) + 
                    ((0.25 * *(*matrix).get_unchecked(add + (i+end-1) * size + j)
                    * *(*matrix).get_unchecked(add + (i+end) * size + j-1)
                    * *(*matrix).get_unchecked(add + (i+end) * size + j+1)
                    * *(*matrix).get_unchecked(add + (i+end+1) * size + j)))).abs();
            }
        }
    }
}

#[allow(unused)]
fn print_vec(vec: &mut [f64], size: usize){
    for i in 0 .. size {
        for j in 0 .. size {
            print!(" {:.*}" , 2, vec[i *size +j]);
        }
        print!("\n");
    }
}


fn main() {
    let term = 100;
    let mut t = term;
    let size = 1000;

    let processes = 2;
    let part = size/processes; 

    let mut time_parallel : f64 = 0.0;
    let mut time_serial : f64 = 0.0;

    {
        let mut vec = vec![0_f64; 2*(size+1)*(size+1)];
        {
            let mut m :usize = 0;
            let mut pool      = scoped_threadpool::Pool::new(processes as u32);
            
            
            
            let now = Instant::now();

            while t > 0{
                let (left, right) = vec.split_at_mut((size+1) * (size+1));
                if m == 0 {
                    m = 1;
                    let ref_left = &right;
                    let (left, right) = left.split_at_mut((size+1) * (size+1)/processes);
                    let mut hold = vec![left, right];
                    
                    pool.scoped( |scoped| {
                        for (id, block) in hold.iter_mut().enumerate(){
                            scoped.execute( move ||{
                                calc2(ref_left, block, size, part, id, m);
                            });
                        }
                    }); 
                }else{
                    m = 0;
                    let ref_left = &left;
                    let (left, right) = right.split_at_mut((size+1) * (size+1)/processes);
                    let mut hold = vec![left, right];
                    
                    pool.scoped( |scoped| {
                        for (id, block) in hold.iter_mut().enumerate(){
                            scoped.execute( move ||{
                                calc2(ref_left,block, size, part, id, m);
                            });
                        }
                    });
                }
                t-=1;
            }
            let elapsed = now.elapsed();
            time_parallel = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        }
        println!("ENDED parallel in {}", time_parallel);
        // print_vec(&mut vec, size);
    }
    {
        t = term;
        let mut vec = vec![0_f64; (size+1)*(size+1)];

        let now = Instant::now();
        while t > 0{
                calc(&mut vec, size, size, 0);
            t-=1;
        }

        let elapsed = now.elapsed();
        time_serial = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);

        println!("ENDED serial in {}", time_serial);
        //print_vec(&mut vec, size);
    }
    println!("\nRESULTS");

    let speedup = time_serial / time_parallel;
    let efficiency = speedup / processes as f64;

    println!("Speedup: {}", speedup);
    println!("Efficiency: {}", efficiency);
}
