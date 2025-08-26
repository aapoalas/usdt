//! Integration test verifying JSON output, including when serialization fails.

// Copyright 2024 Oxide Computer Company
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{thread, time::Duration};

use serde::{Serialize, Serializer};
use usdt::UniqueId;

// Expected error message from serialization failure
const SERIALIZATION_ERROR: &str = "nonono";

#[derive(Debug, Serialize)]
pub struct ProbeArg {
    value: u8,
    buffer: Vec<i64>,
}

impl Default for ProbeArg {
    fn default() -> Self {
        ProbeArg {
            value: 1,
            buffer: vec![1, 2, 3],
        }
    }
}

// A type that intentionally fails serialization
#[derive(Debug, Default)]
pub struct NotJsonSerializable {
    _x: u8,
}

impl Serialize for NotJsonSerializable {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(SERIALIZATION_ERROR))
    }
}

#[usdt::provider]
mod test_json {
    use crate::{NotJsonSerializable, ProbeArg as GoodArg};
    fn good(_: &GoodArg) {}
    fn bad(_: &NotJsonSerializable) {}
}

#[usdt::provider]
mod with_ids {
    use usdt::UniqueId;
    fn start_work(_: &UniqueId) {}
    fn waypoint_from_thread(_: &UniqueId, message: &str) {}
    fn work_finished(_: &UniqueId, result: u64) {}
}

fn main() {
    usdt::register_probes().unwrap();
    println!("PID: {}", std::process::id());

    usdt::register_probes().unwrap();
    let id = UniqueId::new();
    println!("{}", id.as_u64());
    let id = UniqueId::new();
    println!("{}", id.as_u64());
    let id = UniqueId::new();
    println!("{}", id.as_u64());
    let id = UniqueId::new();
    println!("{}", id.as_u64());
    let id = UniqueId::new();
    // with_ids::start_work!(|| &id);
    loop {
        let id2 = id.clone();
        let thr = thread::spawn(move || {
            for _ in 0..10 {
                with_ids::waypoint_from_thread!(|| (&id2, "we're in a thread"));
                thread::sleep(Duration::from_millis(10));
            }
            id2.as_u64()
        });
        let result = thr.join().unwrap();
        println!("{result}");
        thread::sleep(Duration::from_millis(1000));
    }
    // with_ids::work_finished!(|| (&id, result));
    // assert_eq!(result, id.as_u64());

    // loop {
    //     let arg = ProbeArg::default();
    //     test_json::good!(|| {
    //         println!("Firing first probe");
    //         &arg
    //     });

    //     let data = NotJsonSerializable::default();
    //     test_json::bad!(|| {
    //         println!("Firing second probe");
    //         &data
    //     });
    // }
}
