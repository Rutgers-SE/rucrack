#![allow(dead_code)]
#![allow(unused)]
#![deny(warnings)]

extern crate crypto;
extern crate rand;
extern crate time;
extern crate hyper;
extern crate iron;
extern crate rustc_serialize;

mod encryption;
mod util;
mod overflow;
mod keypool;
mod node;
mod crack;

// external packages
use time::PreciseTime;
use rand::Rng;
use rand::os::OsRng;
use hyper::server::{Server, Request, Response};

// project packages
use encryption::{encrypt, decrypt};
use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary};
use overflow::{WrappedStep, WrappedInc};
use keypool::{KeyPool};
use node::{Worker};
use rustc_serialize::json::{self, Json, ToJson};
use crack::crack;

// standar packages
use std::collections::HashMap;
use std::str::from_utf8;
use std::env::args;
use std::sync::mpsc::{Sender, channel};
use std::marker::Sync;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::process::Command;
use std::io::stdin;

fn master_repl() {
    let file_name: String;
    let reader = stdin();
    loop {
        let mut input = String::new();
        reader.read_line(&mut input).expect("Error reading string");
    }
}

fn slave_repl() {
    loop {}
}

fn main() {
    // Load the dictionary
    let sorted_dictionary = load_linux_dictionary().unwrap();
    if args().len() != 4 {
        panic!("You need to supply three arguments");
    }

    // Parse commandline argument
    let file_name = args().nth(1)
        .expect("The first argument is a filename");
    let thread_count: i64 = args().nth(2)
        .expect("Second argument is the thread count")
        .parse()
        .expect("You should have given a valid integer.");
    let role = args().nth(3)
        .expect("You need to define the role of the client");

    // if role == "master".to_string() {
    //     master_repl();
    // } else if role == "slave".to_string() {
    //     slave_repl();
    // }

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
    // let mut kp = KeyPool::new(1, iv, im, 0); // for no
    let ref mut kp = KeyPool::generate_keys(2, im)[0];
    // let ref mut kp = keyss[0];
    // filling bytes
    kp.static_ms_bytes = iv_bytes.clone();

    let keys = kp.split_key(8);

    // let mut keys = KeyPool::generate_keys(64, im);
    // println!("{:?}", kp);
    for k in keys {
        println!("{:?}", k);
    }

    rng.fill_bytes(&mut kp.dynamic_bytes);

    let cipher_text = encrypt(&message.as_bytes(), &kp.to_vec())
        .expect("Could not encrypt for some reason");

    // println!("{}", message);
    // println!("{:?}", cipher_text);

    let start = PreciseTime::now();
    let potential_keys = crack(&cipher_text, &sorted_dictionary, thread_count);
    let end = PreciseTime::now();

    // println!("{} seconds for file {}", start.to(end), file_name);
    // println!("Potential Keys {:?}", potential_keys);
    // println!("Actual Key {:?}", kp.to_vec());
}
