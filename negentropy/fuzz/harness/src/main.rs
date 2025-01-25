// Copyright (c) 2023 Doug Hoyte
// Distributed under the MIT software license

// This is a testing harness for compatibility with the negentropy reference
// implementation's test suite: https://github.com/hoytech/negentropy/tree/master/test

use std::env;
use std::io::{self, BufRead};

use negentropy::{Id, Negentropy, NegentropyStorageVector};

fn main() {
    let frame_size_limit_env_var = env::var("FRAMESIZELIMIT");
    let frame_size_limit = if let Ok(frame_size_limit) = frame_size_limit_env_var {
        frame_size_limit.parse::<usize>().unwrap()
    } else {
        0
    };

    let mut storage = NegentropyStorageVector::new();

    for line in io::stdin().lock().lines() {
        let line_unwrapped = line.unwrap();
        let items: Vec<&str> = line_unwrapped.split(',').collect();

        if items[0] == "item" {
            let created = items[1].parse::<u64>().unwrap();
            let id = items[2];
            let bytes = hex::decode(id).unwrap();
            storage
                .insert(created, Id::from_slice(&bytes).unwrap())
                .unwrap();
        } else if items[0] == "seal" {
            storage.seal().unwrap();
            break;
        } else {
            panic!("unknwown cmd");
        }
    }

    let mut ne = Negentropy::borrowed(&storage, frame_size_limit as u64).unwrap();

    for line in io::stdin().lock().lines() {
        let line_unwrapped = line.unwrap();
        let items: Vec<&str> = line_unwrapped.split(',').collect();

        if items[0] == "initiate" {
            let q = ne.initiate().unwrap();
            if frame_size_limit > 0 && q.len() / 2 > frame_size_limit {
                panic!("frame_size_limit exceeded");
            }
            println!("msg,{}", hex::encode(q));
        } else if items[0] == "msg" {
            let mut q = String::new();

            if items.len() >= 2 {
                q = items[1].to_string();
            }

            if ne.is_initiator() {
                let mut have_ids = Vec::new();
                let mut need_ids = Vec::new();
                let bytes = hex::decode(q).unwrap();
                let resp = ne
                    .reconcile_with_ids(&bytes, &mut have_ids, &mut need_ids)
                    .unwrap();

                for id in have_ids.into_iter() {
                    println!("have,{}", hex::encode(id.as_bytes()));
                }
                for id in need_ids.into_iter() {
                    println!("need,{}", hex::encode(id.as_bytes()));
                }

                if let Some(resp) = resp {
                    q = hex::encode(resp);
                } else {
                    println!("done");
                    continue;
                }
            } else {
                let bytes = hex::decode(q).unwrap();
                let out = ne.reconcile(&bytes).unwrap();
                q = hex::encode(out);
            }

            if frame_size_limit > 0 && q.len() / 2 > frame_size_limit {
                panic!("frame_size_limit exceeded");
            }
            println!("msg,{}", q);
        } else {
            panic!("unknwown cmd");
        }
    }
}
