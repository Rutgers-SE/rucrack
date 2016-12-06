use util::is_english;
use keypool::KeyPool;
use encryption::decrypt;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::str::from_utf8;

fn dc(cipher_text: Vec<u8>,
      key: KeyPool,
      sender: &Sender<Vec<u8>>,
      tdone: &Sender<bool>)
      -> KeyPool {
    let key = key.clone();
    match decrypt(&cipher_text[..], &key.to_vec()) {
        Ok(decrypted_data) => {
            match from_utf8(&decrypted_data) {
                Ok(pt) => {
                    println!("Msg found \"{}\", key = {}", pt, key);
                    if is_english(pt.to_string()) {
                        match sender.send(key.to_vec()) {
                            Ok(_) => {
                                tdone.send(true).unwrap();
                                println!("Sent key");
                            }
                            Err(e) => panic!("Sorry master.. I failed... -- {}", e),
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
    let (tcount, rcount) = channel();

    // let ref rcc = tc.as_ref();
    let klen = keys.len();

    for key in keys {
        let mut key = key.clone();
        let tx = tx.clone();
        let tcount = tcount.clone();
        let tdone = tdone.clone();
        let cipher_text = cipher_text.clone();

        threads.push(thread::spawn(move || {
            // let mut key = key.clone();
            loop {
                if !key.is_done() {
                    key = dc(cipher_text.clone(), key.clone(), &tx, &tdone);
                } else {
                    tcount.send(1).unwrap();
                    break;
                }
            }
        }));
    }

    let mut count = 0;
    let mut output = vec![]; // This will become the list of potential keys

    loop {
        match rdone.try_recv() {
            Ok(value) if value => {
                println!("Awesome");
                let v = rx.recv().unwrap();
                output.push(v);
                println!("Key sent!");
                break;
            }
            _ => {
                // if count == klen {
                //     println!("Here");
                //     break;
                // }
            }
        }

        match rcount.try_recv() {
            Ok(n) => {
                count = count + n;
            }
            _ => ()
        }
        if count == klen {
            println!("Here");
            break;
        }

    }

    for t in threads {
        t.join().unwrap();
    }

    println!("I got here!");

    output
}
