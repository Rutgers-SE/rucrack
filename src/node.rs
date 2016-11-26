// use hyper;
// use hyper::client::{Client};
// use hyper::server::{Server};
extern crate iron;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

use keypool::KeyPool;

use std::thread;
use std::fmt;
use std::net::{ToSocketAddrs, SocketAddr};
use std::iter::Iterator;
use std::sync::mpsc::channel;
use std::sync::mpsc as mp;

use crack::crack;
use hyper::client as cl;
use hyper::Url;


use util::{load_linux_dictionary};
use rustc_serialize::json::{Json, ToJson};

#[derive(Clone, Debug)]
pub struct Worker {
    available_threads: i64,
    ip_address: Url,
    master: Option<Url>,
    slaves: Option<Vec<Url>>
}

impl fmt::Display for Worker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ip_address)
    }
}

impl Worker {
    pub fn builder(av: i64, ip: Url) -> Worker {
        Worker {
            available_threads: av,
            ip_address: ip,
            master: None,
            slaves: None
        }
    }

    pub fn set_master(&mut self, master: Worker) -> Worker {
        self.master = Some(master.ip_address);
        self.clone()
    }

    // pub fn set_slaves(&mut self, slaves: Vec<Worker>) -> Worker {
    //     self.slaves = Some(
    //         slaves.clone()
    //             .iter()
    //             .map(|ref slave| slave.ip_address)
    //             .collect::<Vec<Url>>()
    //         );
    //     self.clone()
    // }

    pub fn done(&mut self) -> Worker {
        self.clone()
    }

    pub fn crack_keypool(&self, cipher_text: Vec<u8>, kp: KeyPool) {
        // let dict = load_linux_dictionary().unwrap();
        let pkeys = crack(kp, cipher_text, self.available_threads).to_json();
    }

    pub fn send_potential_keys(&self, potential_keys: Vec<u8>) {
        let pks = potential_keys.to_json().to_string();
        let client = cl::Client::new();
        let res = client.get("http://google.com").send().unwrap();
    }

    pub fn listen(&mut self) -> thread::JoinHandle<()> {
        fn handler(_: &mut Request) -> IronResult<Response> {
            Ok(Response::with((iron::status::Ok, "Hello, World")))
        }
        let mut slf = self.clone();
        thread::spawn(move || {
            let mut chain = Chain::new(handler);
            Iron::new(chain).http(slf.ip_address).unwrap();
        })
    }
}
