#![allow(dead_code)]
#![allow(unused)]

extern crate crypto;
extern crate rand;
extern crate time;
extern crate rayon;
extern crate uuid;
mod encryption;
mod util;
mod overflow;
mod keypool;

// external packages
use time::PreciseTime;
use rand::Rng;
use rand::os::OsRng;
use rayon::prelude::*;
use uuid::{Uuid};

// project packages
use encryption::{encrypt, decrypt};
use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use overflow::{WrappedStep, WrappedInc};
use keypool::{KeyPool};

// standar packages
use std::collections::HashMap;
use std::str::from_utf8;
use std::env::args;
use std::sync::mpsc::{Sender, channel};
use std::marker::Sync;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;


#[derive(PartialOrd, PartialEq)]
struct Packet(Vec<u8>);
unsafe impl Sync for Packet {}
unsafe impl Send for Packet {}


fn dc(cipher_text: &Vec<u8>, key: &KeyPool) -> Vec<Vec<u8>> {
    let mut output = vec![];
    let mut key = key.clone();
    while !key.is_done() {
        match decrypt(&cipher_text[..], &key.to_vec()) {

            Ok(decrypted_data) => {
                match from_utf8(&decrypted_data) {
                    Ok(pt) => {
                        if is_english(pt.to_string()) {
                            println!("{:?}", key.to_vec());
                            output.push(key.to_vec());
                        }
                    }
                    Err(_) => (),
                }
            }
            Err(_) => (),
        }
        key = key.inc();
    }
    output
}

// returns potential keys
fn crack(cipher_text: &Vec<u8>, dictionary: &Vec<String>, thread_count: i64) -> Arc<Mutex<Vec<Vec<u8>>>> {

    let output: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));

    // load the iv file
    let iv_bytes = read_file_from_arg(args().nth(1))
        .expect("Need to provide the IV file");

    // setup the key pool
    let iv = iv_bytes.len() as u8;
    let im = 16 - iv_bytes.len() as u8;
    let mut keys = KeyPool::generate_keys(thread_count, im); // TODO: change the `1` to a variable


    let mut threads = vec![]; // thread pool

    for mut key in keys {
        key.static_ms_bytes = iv_bytes.clone();
        println!("{:?}", key.to_vec());
        let output = output.clone();
        let cipher_text = cipher_text.clone();
        threads.push(thread::spawn(move || {
            let mut output = output.lock().unwrap();
            output.append(&mut dc(&cipher_text, &key));
        }));
    }

    // for t in threads {
    //     t.join();
    // }

    output
}

fn main() {
    // Load the dictionary
    let sorted_dictionary = load_linux_dictionary().unwrap();

    // Parse commandline argument
    let file_name = args().nth(1)
        .expect("The first argument is a filename");
    let thread_count: i64 = args().nth(2)
        .expect("Second argument is the thread count")
        .parse()
        .expect("You should have given a valid integer.");

    // load file given from commandline argument
    let iv_bytes = read_file_from_arg(Some(file_name.clone()))
        .expect("The filel you provided is not valid");

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

    rng.fill_bytes(&mut kp.dynamic_bytes);

    let cipher_text = encrypt(&message.as_bytes(), &kp.to_vec())
        .expect("Could not encrypt for some reason");

    let start = PreciseTime::now();
    let potential_keys = crack(&cipher_text, &sorted_dictionary, thread_count);
    let end = PreciseTime::now();
    //
    println!("{} seconds for file {}", start.to(end), file_name);
    println!("Potential Keys {:?}", potential_keys);
    println!("Actual Key {:?}", kp.to_vec());

}
