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

use util::{load_linux_dictionary};

#[derive(Clone, Debug)]
pub struct Worker {
    available_threads: i64,
    ip_address: SocketAddr,
    master: Option<SocketAddr>,
    slaves: Option<Vec<SocketAddr>>
}

impl fmt::Display for Worker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ip_address)
    }
}

impl Worker {
    pub fn builder(av: i64, ip: &'static str) -> Worker {
        let addr: SocketAddr = ip.to_string().parse()
            .expect("ip should be a valid socket addr");
        Worker {
            available_threads: av,
            ip_address: addr,
            master: None,
            slaves: None
        }
    }

    pub fn set_master(&mut self, master: Worker) -> Worker {
        self.master = Some(master.ip_address);
        self.clone()
    }

    pub fn set_slaves(&mut self, slaves: Vec<Worker>) -> Worker {
        self.slaves = Some(
            slaves.clone()
                .iter()
                .map(|ref slave| slave.ip_address)
                .collect::<Vec<SocketAddr>>()
            );
        self.clone()
    }

    pub fn done(&mut self) -> Worker {
        self.clone()
    }

    pub fn crack_keypool(cipher_text: &Vec<u8>, kp: KeyPool) {
        let dict = load_linux_dictionary().unwrap();
        let pkeys = crack(&cipher_text, &dict, 4);
    }

    pub fn send_keypool() {
        unimplemented!()
    }

    pub fn send_potential_keys() {
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
