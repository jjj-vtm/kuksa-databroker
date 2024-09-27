/********************************************************************************
* Copyright (c) 2022 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License 2.0 which is available at
* http://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/

use kuksa::{proto, KuksaClient};
use tokio::{runtime::Handle, task::futures};

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> () {
    let parallel = 25;
    let path = "Vehicle.Speed";
    let mut clients: Vec<KuksaClient> = vec![];

    for i in 0..parallel {
        let mut client: KuksaClient =
            KuksaClient::new(kuksa_common::to_uri("127.0.0.1:55556").unwrap());
        clients.push(client);
    }

    let mut datapoints: HashMap<String, proto::v1::Datapoint> = HashMap::new();

    let dp = proto::v1::Datapoint {
        timestamp: None,
        value: Some(proto::v1::datapoint::Value::Float(1.0)),
    };
    datapoints.insert(path.to_owned(), dp);

    let mut datapoints_list: Vec<HashMap<String, proto::v1::Datapoint>> = vec![];

    for i in 0..parallel {
        datapoints_list.push(datapoints.clone());
    }

    let mut counters = vec![];

    for i in 0..parallel {
        counters.push(0);
    }

    let now = Instant::now();
    let msg_cnt = 100_000;
    
    let mut handles = vec![];


    for tr in 0 ..parallel  {
        let mut client = clients.pop().unwrap();
        let datapoint = datapoints_list.pop().unwrap();
        let mut cnt = counters.pop().unwrap();

        let handle = tokio::spawn(async move {
            loop {
                let res = client.set_current_values(datapoint.clone()).await;
                if let Err(e) = res {
                    println!("{:?}", e)
                }
                cnt  += 1;
                if cnt > msg_cnt {
                    println!("Task done");

                    break;
                }
            }
        });
        handles.push(handle);
    }

    for task in handles {
        let _ = task.await;
    }

    let elapsed = now.elapsed().as_micros();
    println!(
        "Time to handle {} message: : {:?} seconds. Msg per second: {:?}",
        parallel * msg_cnt,
        (elapsed as f64 / 1_000_000.0),
        (parallel * msg_cnt) as f64 / (elapsed / 1_000_000) as f64
    );
}
