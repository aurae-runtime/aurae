// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
use std::convert::TryFrom;
use std::env;

use api::Cli;
use vmm::Vmm;

fn main() {
    match Cli::launch(
        env::args()
            .collect::<Vec<String>>()
            .iter()
            .map(|s| s.as_str())
            .collect(),
    ) {
        Ok(vmm_config) => {
            let mut vmm =
                Vmm::try_from(vmm_config).expect("Failed to create VMM from configurations");
            // For now we are just unwrapping here, in the future we might use a nicer way of
            // handling errors such as pretty printing them.
            vmm.run().unwrap();
        }
        Err(e) => {
            eprintln!("Failed to parse command line options. {}", e);
        }
    }
}
