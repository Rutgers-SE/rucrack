#![allow(dead_code)]
#![allow(unused)]

extern crate crypto;
extern crate rand;
extern crate time;
mod encryption;
mod util;
mod overflow;
mod keypool;

use encryption::{encrypt, decrypt};
use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use overflow::{WrappedStep, WrappedInc};
use keypool::{KeyPool};

use time::PreciseTime;

use rand::Rng;
use rand::os::OsRng;
use std::str::from_utf8;
use std::env::args;
use std::fmt;


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
    let thread_count: u8 = args().nth(2).unwrap()
        .parse()
        .expect("You should have given a number");

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
    let mut kp = KeyPool::new(thread_count as i64, iv, im, 0); // for no
    // let mut keys = KeyPool::generate_keys(64, im);

    kp.static_ms_bytes = iv_bytes.clone();

    // for mut key in keys {
    //     key.static_ms_bytes = iv_bytes.clone();
        // println!("{}", key);
    // }

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
    //
    println!("{} seconds for file {}", start.to(end), file_name);
    println!("Potential Keys {:?}", potential_keys);
    println!("Actual Key {:?}", kp.to_vec());
}
