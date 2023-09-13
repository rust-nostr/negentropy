// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use negentropy::Negentropy;

const ID_SIZE: u8 = 10;
const FRAME_SIZE_LIMIT: Option<u64> = None;

fn main() {
    let items = relay_set();

    // Client
    let mut client = Negentropy::new(ID_SIZE, FRAME_SIZE_LIMIT).unwrap();
    client.add_item(0, "aaaaaaaaaaaaaaaaaaaa").unwrap();
    client.add_item(1, "bbbbbbbbbbbbbbbbbbbb").unwrap();
    client.seal().unwrap();
    let now = Instant::now();
    let init_output = client.initiate().unwrap();
    println!("Client init took {} ms", now.elapsed().as_millis());

    // Relay
    let mut relay = Negentropy::new(ID_SIZE, FRAME_SIZE_LIMIT).unwrap();
    println!("Relay items: {}", items.len());
    for (index, item) in items.into_iter().enumerate() {
        relay.add_item(index as u64, item).unwrap();
    }
    relay.seal().unwrap();
    let now = Instant::now();
    let reconcile_output = relay.reconcile(&init_output).unwrap();
    println!("Relay reconcile took {} ms", now.elapsed().as_millis());

    // Client
    let now = Instant::now();
    let mut have_ids = Vec::new();
    let mut need_ids = Vec::new();
    client
        .reconcile_with_ids(&reconcile_output, &mut have_ids, &mut need_ids)
        .unwrap();
    println!("Client reconcile took {} ms", now.elapsed().as_millis());
}

fn relay_set() -> Vec<String> {
    let characters = "abc";
    let length = 20;
    let max = 1_000_000;
    generate_combinations(characters, length, max)
}

fn generate_combinations(characters: &str, length: usize, max: usize) -> Vec<String> {
    let mut combinations = Vec::new();
    let mut current = String::new();
    generate_combinations_recursive(&mut combinations, &mut current, characters, length, 0, max);
    combinations
}

fn generate_combinations_recursive(
    combinations: &mut Vec<String>,
    current: &mut String,
    characters: &str,
    length: usize,
    index: usize,
    max: usize,
) {
    if length == 0 {
        combinations.push(current.clone());
        return;
    }

    for char in characters.chars() {
        if combinations.len() < max {
            current.push(char);
            generate_combinations_recursive(
                combinations,
                current,
                characters,
                length - 1,
                index + 1,
                max,
            );
            current.pop();
        } else {
            return;
        }
    }
}
