// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

fn main() {
    uniffi::generate_scaffolding("./src/negentropy.udl").expect("Building the UDL file failed");
}
