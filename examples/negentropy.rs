// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use negentropy::Negentropy;

fn main() {
    // Client
    let mut client = Negentropy::new(16, None).unwrap();
    client.add_item(0, "aaaaaaaaaaaaaaaa").unwrap();
    client.add_item(1, "bbbbbbbbbbbbbbbb").unwrap();
    client.seal().unwrap();
    let init_output = client.initiate().unwrap();
    println!("Initiator Output: {}", init_output);

    // Relay
    let mut relay = Negentropy::new(16, None).unwrap();
    relay.add_item(0, "aaaaaaaaaaaaaaaa").unwrap();
    relay.add_item(2, "cccccccccccccccc").unwrap();
    relay.add_item(3, "1111111111111111").unwrap();
    relay.add_item(5, "2222222222222222").unwrap();
    relay.add_item(10, "3333333333333333").unwrap();
    relay.seal().unwrap();
    let reconcile_output = relay.reconcile(&init_output).unwrap();
    println!("Reconcile Output: {}", reconcile_output);

    // Client
    let mut have_ids = Vec::new();
    let mut need_ids = Vec::new();
    let reconcile_output_with_ids = client
        .reconcile_with_ids(&reconcile_output, &mut have_ids, &mut need_ids)
        .unwrap();
    println!("Reconcile Output with IDs: {}", reconcile_output_with_ids);
    println!("Have IDs: {:?}", have_ids);
    println!("Need IDs: {:?}", need_ids);
}
