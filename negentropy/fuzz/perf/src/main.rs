// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use negentropy::{Id, Negentropy, NegentropyStorageVector};

fn main() {
    let items = relay_set();

    // Client
    let mut storage_client = NegentropyStorageVector::new();
    storage_client
        .insert(
            0,
            Id::from_slice(&[
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa,
            ])
            .unwrap(),
        )
        .unwrap();
    storage_client
        .insert(
            1,
            Id::from_slice(&[
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb,
            ])
            .unwrap(),
        )
        .unwrap();
    storage_client.seal().unwrap();
    let mut client = Negentropy::borrowed(&storage_client, 0).unwrap();
    let now = Instant::now();
    let init_output = client.initiate().unwrap();
    println!("Client init took {} ms", now.elapsed().as_millis());

    // Relay
    let mut storage_relay = NegentropyStorageVector::new();
    println!("Relay items: {}", items.len());
    for (index, item) in items.into_iter().enumerate() {
        storage_relay
            .insert(index as u64, Id::from_slice(&item).unwrap())
            .unwrap();
    }
    storage_relay.seal().unwrap();
    let mut relay = Negentropy::borrowed(&storage_relay, 0).unwrap();
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

fn relay_set() -> Vec<Vec<u8>> {
    let characters = b"abc";
    let length = 32;
    let max = 1_000_000;
    generate_combinations(characters, length, max)
}

fn generate_combinations(characters: &[u8], length: usize, max: usize) -> Vec<Vec<u8>> {
    let mut combinations = Vec::with_capacity(max);
    let mut current = Vec::with_capacity(length);
    generate_combinations_recursive(&mut combinations, &mut current, characters, length, 0, max);
    combinations
}

fn generate_combinations_recursive(
    combinations: &mut Vec<Vec<u8>>,
    current: &mut Vec<u8>,
    characters: &[u8],
    length: usize,
    _index: usize,
    max: usize,
) {
    if length == 0 {
        combinations.push(current.clone());
        return;
    }

    for byte in characters.iter() {
        if combinations.len() < max {
            current.push(*byte);
            generate_combinations_recursive(
                combinations,
                current,
                characters,
                length - 1,
                _index + 1,
                max,
            );
            current.pop();
        } else {
            return;
        }
    }
}
