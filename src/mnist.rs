use std::fs::File;
use std::io;
use std::io::prelude::*;
use brain::Brain;

use rand::{thread_rng, Rng};

pub struct Mnist {
    pub number: usize,
    pub rows: usize,
    pub columns: usize,
    pub images: Vec<Vec<f64>>,
    pub results: Vec<Vec<f64>>
}

impl Mnist {
    pub fn new(data: &str, idx: &str) -> Result<Mnist, io::Error> {
        let mut data = File::open(data)?;
        let mut buffer = [0; 16];
        data.read_exact(&mut buffer)?;
        if buffer[0..4] != [0, 0, 0x8, 0x3] {
            // Invalid magic number
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let number = ((((buffer[4] as usize)<<24)+(buffer[5] as usize)<<16)+(buffer[6] as usize)<<8)+(buffer[7] as usize);
        let rows = ((((buffer[8] as usize)<<24)+(buffer[9] as usize)<<16)+(buffer[10] as usize)<<8)+(buffer[11] as usize);
        let columns = ((((buffer[12] as usize)<<24)+(buffer[13] as usize)<<16)+(buffer[14] as usize)<<8)+(buffer[15] as usize);

        let images = {
            let mut tmp_images = Vec::with_capacity(number);
            for k in 0..number {
                tmp_images.push(vec![0; rows*columns]);
                data.read_exact(&mut tmp_images[k])?;
            }
            tmp_images.iter().map(|ref v| v.iter().map(|&x| x as f64/256.).collect()).collect()
        };


        let mut idx = File::open(idx)?;
        let mut buffer = [0; 8];
        idx.read_exact(&mut buffer)?;
        if buffer[0..4] != [0, 0, 0x8, 0x1] {
            // Invalid magic number
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        if number != ((((buffer[4] as usize)<<24)+(buffer[5] as usize)<<16)+(buffer[6] as usize)<<8)+(buffer[7] as usize) {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        let results = {
            let mut tmp_results = vec![0; number];
            idx.read_exact(&mut tmp_results)?;
            tmp_results.iter().map(|&i| {
                let mut out = vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
                out[i as usize] = 1.;
                out
            }).collect()
        };

        Ok(Mnist {
            number,
            rows,
            columns,
            images,
            results
        })
    }

    pub fn measure_error(&self, network: &mut Brain) -> f64 {
        let mut rng = thread_rng();
        // average errors on 100 runs
        let mut sum = 0.;
        for _ in 0..1000 {
            let index: usize = rng.gen::<usize>()%self.number;
            network.forward(&self.images[index]).unwrap();
            let mut expected_val = 0;
            for i in 0..self.results[index].len() {
                if self.results[index][i] > 0.5 {
                    expected_val = i;
                }
            }
            let mut val = 0;
            let outputs = network.get_outputs();
            for i in 1..outputs.len() {
                if outputs[i] > outputs[val] {
                    val = i;
                }
            }
            if val == expected_val {
                sum += 1.;
            }
        }
        1. - sum / 1000.
    }
}
