#![allow(dead_code)]
#![allow(unused)]


extern crate crypto;
extern crate rand;
extern crate num;
// extern crate time;

// use time::now;

use std::{thread, time};

use crypto::{symmetriccipher, buffer, aes, blockmodes};
use crypto::buffer::{ReadBuffer, WriteBuffer, BufferResult};

use rand::Rng;
use rand::os::OsRng;
use std::str::from_utf8;

use std::env::args;
use std::io::prelude::*;
use std::fs::File;
use std::iter::Iterator;
use std::path::Path;
use num::bigint::{BigInt, ToBigInt, Sign};

use num::PrimInt;

fn u8_vector(amount: u8) -> Vec<u8> {
    let mut v: Vec<u8> = vec![];

    for i in 0..amount {
        v.push(0);
    }

    v
}

// Encrypt a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {

    // Create an encryptor instance of the best performing
    // type available for the platform.
    // let mut encryptor =
    //     aes::cbc_encryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);

    // For the project we will use ecb
    let mut encryptor = aes::ecb_encryptor(aes::KeySize::KeySize128, key, blockmodes::PkcsPadding);

    // Each encryption operation encrypts some data from
    // an input buffer into an output buffer. Those buffers
    // must be instances of RefReaderBuffer and RefWriteBuffer
    // (respectively) which keep track of how much data has been
    // read from or written to them.
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 1024];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    // Each encryption operation will "make progress". "Making progress"
    // is a bit loosely defined, but basically, at the end of each operation
    // either BufferUnderflow or BufferOverflow will be returned (unless
    // there was an error). If the return value is BufferUnderflow, it means
    // that the operation ended while wanting more input data. If the return
    // value is BufferOverflow, it means that the operation ended because it
    // needed more space to output data. As long as the next call to the encryption
    // operation provides the space that was requested (either more input data
    // or more output space), the operation is guaranteed to get closer to
    // completing the full operation - ie: "make progress".
    //
    // Here, we pass the data to encrypt to the enryptor along with a fixed-size
    // output buffer. The 'true' flag indicates that the end of the data that
    // is to be encrypted is included in the input buffer (which is true, since
    // the input data includes all the data to encrypt). After each call, we copy
    // any output data to our result Vec. If we get a BufferOverflow, we keep
    // going in the loop since it means that there is more work to do. We can
    // complete as soon as we get a BufferUnderflow since the encryptor is telling
    // us that it stopped processing data due to not having any more data in the
    // input buffer.
    loop {
        let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));

        // "write_buffer.take_read_buffer().take_remaining()" means:
        // from the writable buffer, create a new readable buffer which
        // contains all data that has been written, and then access all
        // of that data as a slice.
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

// Decrypts a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
//
// This function is very similar to encrypt(), so, please reference
// comments in that function. In non-example code, if desired, it is possible to
// share much of the implementation using closures to hide the operation
// being performed. However, such code would make this example less clear.
fn decrypt(encrypted_data: &[u8],
           key: &[u8])
           -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    // let mut decryptor =
    //     aes::cbc_decryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);
    let mut decryptor = aes::ecb_decryptor(aes::KeySize::KeySize128, key, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 1024];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

fn read_file_from_arg(arg: Option<String>) -> Option<Vec<u8>> {
    match arg {
        Some(name) => {
            match File::open(name) {
                Ok(mut file) => {
                    let mut output: Vec<u8> = vec![];
                    match file.read_to_end(&mut output) {
                        Ok(_) => Some(output),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
        None => None,
    }
}

fn is_english() -> bool {
    true
}

#[derive(Debug)]
struct KeyPool {
    static_ms_bytes: Vec<u8>,
    dynamic_bytes: Vec<u8>,
    static_ls_bytes: Vec<u8>,
}

trait WrapInc {
    fn inc(self) -> Self;
}

impl WrapInc for u8 {
    fn inc(self) -> Self {
        match self {
            255u8 => 0u8,
            _ => self + 1u8,
        }
    }
}

impl KeyPool {
    fn new(ms_bytes: u8, dy_bytes: u8, ls_bytes: u8) -> KeyPool {
        KeyPool {
            static_ms_bytes: u8_vector(ms_bytes),
            dynamic_bytes: u8_vector(dy_bytes),
            static_ls_bytes: u8_vector(ls_bytes),
        }
    }

    fn is_done(&self) -> bool {
        let mut done = true;
        for byte in self.dynamic_bytes.clone() {
            if byte != 255 {
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
                idx = idx - 1;
            } else {
                break;
            }
        }

        KeyPool {
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
fn crack(cipher_text: &Vec<u8>) -> Vec<Vec<u8>> {
    let output: Vec<Vec<u8>> = vec![];

    output
}

fn main() {
    let iv_bytes = match read_file_from_arg(args().nth(1)) {
        Some(file_bytes) => file_bytes,
        None => panic!("Need to provide the IV file"),
    };
    // let message = "This is a test of the message";
    //
    //
    let mut key0: [u8; 16] = [0; 16];
    //
    let mut rng = OsRng::new().ok().unwrap();
    // rng.fill_bytes(&mut key0);
    //
    // let encrypted_data = encrypt(message.as_bytes(), &key0).ok().unwrap();
    // let decrypted_data = decrypt(&encrypted_data[..], &key0).ok().unwrap();
    //
    //
    // match from_utf8(&decrypted_data) {
    //     Ok(s) => println!("{}", s),
    //     Err(_) => println!("Decrypted value is not correct"),
    // }
    // assert!(message.as_bytes() == &decrypted_data[..]);

    // let mut kp = KeyPool::new();
    // let power = 14;


    // for cur in 0..(2.pow(power)) {
    //     if cur == 2.pow(power)-1 {
    // println!("{:?}", &kp.to_bytes_le());
    // }
    // kp = kp.next().unwrap();
    // }

    // let mut looped: bool;
    // let mut over = 255 as u8;
    // let tmp = over;
    // over = over.inc().inc();
    let iv = iv_bytes.len() as u8;
    let im = 16 - iv_bytes.len() as u8;
    let mut kp = KeyPool::new(iv, im, 0);

    kp.static_ms_bytes = iv_bytes;
    // rng.fill_bytes(&mut kp.dynamic_bytes);
    let mut count = 0;

    while !kp.is_done() {
        if count % 1024 == 0 {
            println!("{:?}", kp.to_vec());
        }
        kp = kp.inc();
        // thread::sleep(time::Duration::from_millis(100));
        count += 1;
    }

    println!("{:?}", kp.to_vec());

}
