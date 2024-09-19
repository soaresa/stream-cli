use crate::trade_service::TradeTask;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub fn start_polling(mut tx: Sender<TradeTask>) -> JoinHandle<()> {
    let worker = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000));
        let task1 = TradeTask::new(1.0, 2.0, "first task".to_string());

        tx.borrow_mut().send(task1).unwrap();
    });

    // dbg!(&worker);

    // Push more data to the channel
    // tx.send("Yet another job").unwrap();

    // worker.join().unwrap();
    worker
}
