#![allow(dead_code)]
#![allow(unused)]
#![deny(warnings)]

extern crate crypto;
extern crate rand;
extern crate time;
extern crate hyper;
extern crate iron;
extern crate params;
extern crate bodyparser;
extern crate rustc_serialize;
extern crate crossbeam;
// extern crate unicode;

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
// use hyper::server::{Server, Request, Response};
use hyper::client as cl;
use hyper::{Error, Url};
use hyper::client::IntoUrl;
use iron::prelude::*;
use iron as i;
use params::Params;
use bodyparser::{Raw, Json as J};
// use iron::Plugin;

// project packages
use encryption::{encrypt, decrypt};
use util::{u8_vector, read_file_from_arg, is_english, load_linux_dictionary, prompt, read_r4c_file};
use overflow::{WrappedStep, WrappedInc};
use keypool::KeyPool;
use node::Worker;
use rustc_serialize::json::{self, Json, ToJson};
use crack::crack;

// standar packages
use std::str::{from_utf8, FromStr};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::io::{stdout, stdin};
use std::net::{SocketAddrV4, TcpStream, UdpSocket, TcpListener, Ipv4Addr, SocketAddr,
               ToSocketAddrs};
use std::collections::HashSet;
use std::env::args;
use std::marker::Sync;
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::io::prelude::*;

#[derive(Debug)]
enum Op {
    AddSlave(Url),
    ListSlaves,
    IvFile(String),
    CipherFile(String),
    Start,
}

fn sanitize(input: String) -> Vec<String> {
    let splits = input.split(" ");
    splits.map(|t| t.to_string().trim().to_string())
        .collect::<Vec<String>>()
}

// NOTE: there must be a better way
fn parse(value: Vec<String>) -> Result<Op, String> {
    if value[0] == "slave".to_string() {
        let ref raw = value[1];
        match Url::from_str(raw.as_str()) {
            Ok(value) => Ok(Op::AddSlave(value)),
            Err(e) => Err(e.to_string()),
        }
    } else if value[0] == "list".to_string() {
        Ok(Op::ListSlaves)
    } else if value[0] == "start".to_string() {
        Ok(Op::Start)
    } else if value[0] == "iv-file".to_string() {
        let ref raw = value[1];
        Ok(Op::IvFile(raw.clone()))
    } else if value[0] == "cipher-file".to_string() {
        let ref raw = value[1];
        Ok(Op::CipherFile(raw.clone()))
    } else {
        Err("command not supported".to_string())
    }
}

fn master_repl() {
    // prompt("master-ip and port> ");
    //
    // let mut raw_ip = String::new();
    // stdin().read_line(&mut raw_ip).expect("Expected valid string");
    let raw = args().nth(2).unwrap();
    let mut base = "http://127.0.0.1:".to_string();
    base.push_str(raw.as_str());

    let ip = match Url::from_str(&base.as_str()) {
        Ok(value) => value,
        Err(e) => panic!("{}", e),
    };

    let w = Worker::builder(4, ip.clone()); // TODO: 4 is the thread count
    let mut socket_set = HashSet::new();
    let mut iv_bytes: Option<Vec<u8>> = None;
    let mut cipher_text: Option<Vec<u8>> = None;

    println!("{:?}", w);

    loop {
        // let mut iv_bytes = iv_bytes.clone();
        let mut input = String::new();
        prompt("master> ");
        stdin().read_line(&mut input).expect("You did not enter a valid string");

        let ivb = iv_bytes.clone();
        let sslen = socket_set.len();
        let ss = socket_set.clone();
        let ct = cipher_text.clone();

        let tokens: Vec<String> = sanitize(input.clone());
        match parse(tokens) {
            Ok(v) => {
                match v {
                    Op::AddSlave(ip) => {
                        println!("Adding IP: {:?}", ip);
                        socket_set.insert(ip);
                    }
                    Op::ListSlaves => {
                        println!("CipherFile {:?}", ct);
                        println!("IV file {:?}", ivb);
                        println!("{:?}", socket_set);
                    }
                    Op::IvFile(f_name) => {
                        iv_bytes = read_r4c_file(f_name);
                    }
                    Op::CipherFile(f_name) => {
                        cipher_text = read_r4c_file(f_name);
                    }
                    Op::Start => {
                        use hyper::client::Client;
                        let bytes = ivb.unwrap_or(vec![]);

                        // Setting up the key
                        let mut kp = KeyPool::new(bytes.len() as u8, (16 - bytes.len()) as u8, 0);
                        kp.static_ms_bytes = bytes;
                        // NOTE: 16 will be replaced with the ip's
                        let keys = kp.split_key(ss.len() as i64);

                        crossbeam::scope(|scope| {
                            scope.defer(|| println!("All computers done"));
                            for idx in 0..ss.len() {
                                let keys = keys.clone();
                                let ss = ss.clone();
                                let cipher_text = cipher_text.clone();

                                scope.spawn(move || {
                                    let client = Client::new();
                                    let ref ul: Url = *(ss.iter().nth(idx).unwrap());
                                    let mut url = ul.clone();

                                    let cipher = json::encode(&cipher_text).unwrap();
                                    let cipher_string = cipher.to_string();

                                    let mut msg = "key=".to_string();
                                    let job: Json = keys[idx].to_json();
                                    msg.push_str(job.to_string().as_str());
                                    msg.push_str("&cipher=");
                                    msg.push_str(cipher_string.as_str());

                                    url.set_query(Some(msg.as_str()));


                                    // let final = msg.as_str();
                                    // println!("{:?}", msg);

                                    let resp = client.post(url)
                                        .send()
                                        .unwrap();

                                    // println!("{:?}", resp);
                                });
                            }
                        });
                    }
                    // _ => unimplemented!()
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

fn slave_repl() {
    fn handle(req: &mut iron::Request) -> i::IronResult<iron::Response> {
        // println!("{:?}", req.get::<Params>());
        // println!("Params:{:?}", req.get::<Params>());
        match req.get::<Params>() {
            Ok(map) => {
                match map.find(&["key"]) {
                    Some(&params::Value::String(ref key)) => {
                        match map.find(&["cipher"]) {
                            Some(&params::Value::String(ref cipher)) => {
                                let cipher_text: Vec<u8> = json::decode(&cipher).unwrap();
                                let kp: KeyPool = json::decode(&key).unwrap();
                                // println!("{}, {:?}", kp, cipher_text);

                                println!("Starting Crack");
                                // NOTE: we assuming 4 threads per machine
                                let start = PreciseTime::now();
                                let keys = crack(kp, cipher_text, 4);
                                let end = PreciseTime::now();
                                println!("Finished Crack");
                                println!("number of potential keys {:?}", keys.len());
                                println!("potential keys {:?}", keys);
                                println!("Time taken {}\n\n\n", start.to(end));

                                let keys_json = json::encode(&keys).unwrap();
                                return Ok(iron::Response::with((iron::status::Ok,
                                                                keys_json.to_string())));
                            }
                            _ => println!("Invalid request"),
                        }
                    }
                    _ => println!("Invalid request"),
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }


        Ok(iron::Response::with((iron::status::Ok, "Hello, From Iron")))
    }
    let mut base: String = "127.0.0.1:".to_owned();
    match args().nth(2) {
        Some(port) => {
            base.push_str(port.as_str());
        }
        None => {
            let port: &str = "4567";
            base.push_str(port);
        }
    }
    println!("Listening on port {}", base);

    let mut chain = i::Chain::new(handle);
    match i::Iron::new(chain).http(base.as_str()) {
        Ok(_) => println!("Success"),
        Err(_) => println!("Noooo"),
    }

    // loop {}
}

fn main() {
    let role = args()
        .nth(1)
        .expect("You need to pass the role as the first argument");

    if "master".to_string() == role {
        master_repl();
    } else if "slave".to_string() == role {
        slave_repl();
    }
}
