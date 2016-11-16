#![allow(dead_code)]
#![allow(unused)]


extern crate crypto;
extern crate rand;
extern crate time;
mod encryption;
mod util;

use encryption::{encrypt, decrypt};
use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};

use time::PreciseTime;

use rand::Rng;
use rand::os::OsRng;
use std::str::from_utf8;

use std::env::args;
use std::io::prelude::*;
use std::fs::File;
use std::iter::Iterator;
use std::path::Path;

trait WrappedInc {
    fn inc(self) -> Self;
    fn dec(self) -> Self;
}

trait WrappedStep {
    fn step(self, &Self) -> Self;
    fn back(self, &Self) -> Self;
}

impl WrappedStep for u8 {
    fn step(self, other: &Self) -> Self {
        let mut counter: Self = 0;
        let mut output = self.clone();
        while counter < other.clone() {
            output = output.inc();
            counter += 1;
        }
        output
    }
    fn back(self, other: &Self) -> Self {
        self
    }
}

impl WrappedInc for u8 {
    fn inc(self) -> Self {
        match self {
            255u8 => 0u8,
            _ => self + 1u8,
        }
    }

    fn dec(self) -> Self {
        match self {
            0u8 => 255u8,
            _ => self - 1,
        }
    }
}


#[derive(Debug, Clone)]
struct KeyPool {
    dynamic_ms_cap: u8,
    parition_count: u8,
    static_ms_bytes: Vec<u8>,
    dynamic_bytes: Vec<u8>,
    static_ls_bytes: Vec<u8>,
}

impl KeyPool {
    fn new(parition_count: i64, ms_bytes: u8, dy_bytes: u8, ls_bytes: u8) -> KeyPool {
        let cap = (256 / parition_count);
        println!("{} {}", cap, parition_count);
        KeyPool {
            dynamic_ms_cap: (cap as u8).dec(),
            parition_count: parition_count as u8,
            static_ms_bytes: u8_vector(ms_bytes),
            dynamic_bytes: u8_vector(dy_bytes),
            static_ls_bytes: u8_vector(ls_bytes),
        }
    }

    // returns a vector of keys containing the amount specified by the `parition_count`
    fn generate_keys(parition_count: i64, dynamic_byte_len: i64) -> Vec<KeyPool> {
        let mut output: Vec<KeyPool> = vec![];
        let step = 256 / parition_count;
        let mut cursor = 0;

        for key_id in 0..parition_count {
            let mut db = u8_vector(dynamic_byte_len as u8);
            println!("Cursor: {}", cursor);
            db[0] = cursor;
            output.push(KeyPool {
                dynamic_ms_cap: cursor - 1,
                parition_count: parition_count as u8,
                static_ms_bytes: vec![],
                dynamic_bytes: db,
                static_ls_bytes: vec![]
            });
            cursor = cursor.step(&(step as u8));
        }

        output
    }

    fn is_done(&self) -> bool {
        let mut done = true;
        for idx in 1..self.dynamic_bytes.len() {
            if self.dynamic_bytes[idx] != 255 || self.dynamic_bytes[0] != self.dynamic_ms_cap {
                done = false;
                break;
            }
        }
        done
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut output: Vec<u8> = vec![];
        for b in self.static_ms_bytes.clone() {
            output.push(b);
        }
        for b in self.dynamic_bytes.clone() {
            output.push(b);
        }
        for b in self.static_ls_bytes.clone() {
            output.push(b);
        }

        output
    }

    // only increment the dynamic bytes
    fn inc(&mut self) -> KeyPool {
        let mut idx = self.dynamic_bytes.len() - 1;
        loop {

            if self.dynamic_bytes.len() == 0 || idx > self.dynamic_bytes.len() - 1 {
                break;
            }

            let tmp = self.dynamic_bytes[idx];
            self.dynamic_bytes[idx] = self.dynamic_bytes[idx].inc();
            let progress = tmp > self.dynamic_bytes[idx];
            if progress {
                idx -= 1;
            } else {
                break;
            }
        }

        KeyPool {
            dynamic_ms_cap: self.dynamic_ms_cap.clone(),
            parition_count: self.parition_count.clone(),
            static_ms_bytes: self.static_ms_bytes.clone(),
            dynamic_bytes: self.dynamic_bytes.clone(),
            static_ls_bytes: self.static_ls_bytes.clone(),
        }

    }
}

// impl Iterator for KeyPool {
//     type Item = KeyPool;
//     // fn next(&mut self) -> Option<Self::Item> {
//     //     let next_num = &self.current_key + 1.to_bigint().unwrap();
//     //     Some(KeyPool { current_key: next_num})
//     // }
// }

// returns potential keys
fn crack(cipher_text: &Vec<u8>, dictionary: &Vec<String>) -> Vec<Vec<u8>> {
    // define the output
    let mut output: Vec<Vec<u8>> = vec![];

    // load the iv file
    let iv_bytes = match read_file_from_arg(args().nth(1)) {
        Some(file_bytes) => file_bytes,
        None => panic!("Need to provide the IV file"),
    };

    // setup the key pool
    let iv = iv_bytes.len() as u8;
    let im = 16 - iv_bytes.len() as u8;
    let mut kp = KeyPool::new(1, iv, im, 0);
    // ind the iv to the MSB
    kp.static_ms_bytes = iv_bytes;

    while !kp.is_done() {

        match decrypt(&cipher_text[..], &kp.to_vec()) {
            Ok(decrypted_data) => {
                match from_utf8(&decrypted_data) {
                    Ok(pt) => {
                        if is_english(pt.to_string()) {
                            output.push(kp.to_vec())
                        }
                    }
                    Err(_) => (),
                }
            }
            Err(_) => (),
        }

        kp = kp.inc();
    }
    output
}

fn main() {
    // Load the dictionary
    let sorted_dictionary = load_linux_dictionary().unwrap();

    // Parse commandline argument
    let file_name = args().nth(1).unwrap();

    // load file given from commandline argument
    let iv_bytes = match read_file_from_arg(Some(file_name.clone())) {
        Some(file_bytes) => file_bytes,
        None => panic!("Need to provide the IV file"),
    };

    // The original plain text
    let message = "This is a test of the message";

    // Random number generator
    let mut rng = OsRng::new() // new and unwrap are Rust idioms
        .ok().unwrap();

    // initialize base key
    let iv = iv_bytes.len() as u8;
    let im = 16 - iv_bytes.len() as u8;
    let mut kp = KeyPool::new(1, iv, im, 0); // for no
    kp.static_ms_bytes = iv_bytes.clone();
    rng.fill_bytes(&mut kp.dynamic_bytes);

    let cipher_text = match encrypt(&message.as_bytes(), &kp.to_vec()) {
        Ok(cp) => cp,
        Err(e) => {
            panic!("Could not encrypt for some reason: {:?}", e);
        }
    };

    let start = PreciseTime::now();
    let potential_keys = crack(&cipher_text, &sorted_dictionary);
    let end = PreciseTime::now();

    println!("{} seconds for file {}", start.to(end), file_name);
    println!("Potential Keys {:?}", potential_keys);
    println!("Actual Key {:?}", kp.to_vec());
}
