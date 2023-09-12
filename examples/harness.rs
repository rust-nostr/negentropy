// This is a testing harness for compatibility with the negentropy reference
// implementation's test suite: https://github.com/hoytech/negentropy/tree/master/test

use negentropy::Negentropy;
use std::io;
use std::env;

fn main() {
    let id_size = 16;

    let frame_size_limit_env_var = env::var("FRAMESIZELIMIT");
    let frame_size_limit = if frame_size_limit_env_var.is_ok() { frame_size_limit_env_var.unwrap().parse::<usize>().unwrap() } else { 0 };

    let mut ne = Negentropy::new(id_size, Some(frame_size_limit as u64)).unwrap();

    for line in io::stdin().lines() {
        let line_unwrapped = line.unwrap();
        let items: Vec<&str> = line_unwrapped.split(",").collect();

        if items[0] == "item" {
            let created = items[1].parse::<u64>().unwrap();
            let id = items[2];
            ne.add_item(created, id).unwrap();
        } else if items[0] == "seal" {
            ne.seal().unwrap();
        } else if items[0] == "initiate" {
            let q = ne.initiate().unwrap();
            if frame_size_limit > 0 && q.len()/2 > frame_size_limit { panic!("frameSizeLimit exceeded"); }
            println!("msg,{}", q);
        } else if items[0] == "msg" {
            let mut q = String::new();

            if items.len() >= 2 {
                q = items[1].to_string();
            }

            if ne.is_initiator() {
                let mut have_ids = Vec::new();
                let mut need_ids = Vec::new();
                q = ne.reconcile_with_ids(&q, &mut have_ids, &mut need_ids).unwrap();

                for id in &have_ids { println!("have,{}", id); }
                for id in &need_ids { println!("need,{}", id); }

                if q.len() == 0 {
                    println!("done");
                    continue;
                }
            } else {
                q = ne.reconcile(&q).unwrap();
            }

            if frame_size_limit > 0 && q.len()/2 > frame_size_limit { panic!("frameSizeLimit exceeded"); }
            println!("msg,{}", q);
        } else {
            panic!("unknwown cmd");
        }
    }
}