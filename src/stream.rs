use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// the trade tasks that the stream processes
#[derive(Debug, Clone)]
pub struct TradeTask {
    /// trade price
    pub price: f64,
    /// trade units
    pub units: f64,
    /// optional, memo
    pub memo: String,
}

impl TradeTask {
    pub fn new(price: f64, units: f64, memo: String) -> Self {
        TradeTask { price, units, memo }
    }
}

pub(crate) fn init() -> (Sender<TradeTask>, Receiver<TradeTask>) {
    mpsc::channel::<TradeTask>()
}

pub fn demo_send(tx: &Sender<TradeTask>) {
    // Push data to the channel
    let task1 = TradeTask::new(1.0, 2.0, "first task".to_string());
    let task2 = TradeTask::new(3.1, 4.1, "second task".to_string());
    tx.send(task1).unwrap();
    tx.send(task2).unwrap();
}

pub(crate) fn listen(rx: Receiver<TradeTask>) -> JoinHandle<()> {
    let worker = thread::spawn(move || loop {
        let job = rx.recv();

        match job {
            Ok(job) => println!("Job: {:?}", job),
            Err(_) => break,
        }
    });

    worker
}
