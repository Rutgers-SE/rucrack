extern crate iron;
use std::fmt;
use hyper::Url;

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
}
