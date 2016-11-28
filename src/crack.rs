use util::is_english;
use keypool::KeyPool;
use encryption::decrypt;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::str::from_utf8;
// NOTE: mid kill attempt
// use celix::pthread_cancel;
// use std::os::unix::thread::JoinHandleExt;

fn dc(cipher_text: &Vec<u8>,
      key: &KeyPool,
      sender: &Sender<Vec<u8>>,
      tdone: &Sender<bool>)
      -> KeyPool {
    let mut key = key.clone();
    match decrypt(&cipher_text[..], &key.to_vec()) {
        Ok(decrypted_data) => {
            match from_utf8(&decrypted_data) {
                Ok(pt) => {
                    println!("Msg found \"{}\", key = {}", pt, key);
                    if is_english(pt.to_string()) {
                        match sender.send(key.to_vec()) {
                            Ok(_) => tdone.send(true).unwrap(),
                            Err(e) => panic!("Sorry master.. I fucked up... -- {}", e),
                        }
                    }

                }
                Err(_) => {
                    tdone.send(false).unwrap();
                }

            }
        }
        Err(_) => {
            tdone.send(false).unwrap();
        }
    }
    key.inc()
}

// returns potential keys
pub fn crack(kp: KeyPool, cipher_text: Vec<u8>, thread_count: i64) -> Vec<Vec<u8>> {
    let keys = kp.split_key(thread_count as u64);
    for k in keys.clone() {
        println!("{}", k);
    }
    // let mut tc = Arc::new(thread_count.clone());

    let mut threads = vec![]; // thread pool

    // com. channels
    let (tx, rx) = channel();
    let (tdone, rdone) = channel::<bool>();
    let (tkill, _rkill) = channel();

    // let ref rcc = tc.as_ref();

    for key in keys {
        let mut key = key.clone();
        let tx = tx.clone();
        let tdone = tdone.clone();
        let cipher_text = cipher_text.clone();

        threads.push(thread::spawn(move || {
            loop {
                key = dc(&cipher_text, &key, &tx, &tdone);
                if key.is_done() {
                    break;
                }
            }
        }));
    }

    let mut output = vec![]; // This will become the list of potential keys

    loop {
        match rdone.recv() {
            Ok(value) if value => {
                let v = rx.recv().unwrap();
                tkill.send(()).unwrap();
                output.push(v);
                println!("Key sent!");
                break;
            }
            _ => (),
        }
    }

    for t in threads {
        t.join().unwrap();
    }

    output
}
