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
use hyper::client as cl;

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
use std::str::{from_utf8, FromStr};
use std::env::args;
use std::sync::mpsc::{Sender, channel};
use std::marker::Sync;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::process::Command;
use std::io::stdin;
use std::io::stdout;
use std::io::prelude::*;
use std::net::{SocketAddrV4, TcpStream, UdpSocket, TcpListener, Ipv4Addr};
use std::net::ToSocketAddrs;

enum Op {
    SendKey,
    SetupSlave(String),
    Err
}

fn parse (value: &'static str) -> Op {
    if value == "send" {
        Op::SendKey
    } else {
        Op::Err
    }
}

fn master_repl() {
    print!("master-ip and port> ");
    stdout().flush();

    let mut raw_ip = String::new();
    stdin().read_line(&mut raw_ip).expect("Expected valid string");
    let tokens = raw_ip.split(":").collect::<Vec<&str>>();

    assert!(tokens.len() == 2);

    let ip = match Ipv4Addr::from_str(tokens[0].trim()) {
        Ok(value) => value,
        Err(e) => panic!("{}", e)
    };

    let port = u64::from_str(tokens[1]);

    let w = Worker::builder(4, ip.clone().to_string().as_str());


    let mut input = String::new();
    loop {
        print!("> ");
        stdout().flush();
        stdin().read_line(&mut input).expect("You did not enter a valid string");
        let splits = input.split(" ");
        let tokens = splits.collect::<Vec<&str>>();

        println!("{}", input);

        // match parse(tokens[0]) {
        //     Op::SendKey => {
        //         1;
        //     }
        //     _ => ()
        // }

    }
}

fn slave_repl() {
    loop {}
}

fn main() {
    let role = args().nth(1)
        .expect("Role");

    if "master".to_string() == role {
        master_repl();
    } else if "slave".to_string() == role {
        // slave_repl();
    }

    // let client = cl::Client::new();
    // let mut res: cl::Response = client.get("https://reddit.com").send().unwrap();
    // let mut output = String::new();
    // res.read_to_string(&mut output);
    // println!("{}", output);

    // Load the dictionary
    // let sorted_dictionary = load_linux_dictionary().unwrap();
    // if args().len() != 4 {
    //     panic!("You need to supply three arguments");
    // }
    //
    // // Parse commandline argument
    // let thread_count: i64 = args().nth(2)
    //     .expect("Second argument is the thread count")
    //     .parse()
    //     .expect("You should have given a valid integer.");
    // let role = args().nth(3)
    //     .expect("You need to define the role of the client");

    // // Random number generator
    // let mut rng = OsRng::new() // new and unwrap are Rust idioms
    //     .ok().unwrap();

}
