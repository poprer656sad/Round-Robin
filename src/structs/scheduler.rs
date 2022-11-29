use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{
            AtomicU32,
            Ordering::{Acquire,Release}, AtomicBool
        }
    }
};
use crate::task_process::{
    Task,
    Process
};
use tokio::{
    time::{sleep, Duration},
    sync::Mutex
};

pub struct Scheduler{
    pub wait_queue: Vec<(u16, Process)>,
    pub ready_queue: Arc<Mutex<VecDeque<Task>>>,
    pub clock: Arc<AtomicU32>,
    pub time_quantum: u16,
    pub terminable: Arc<AtomicBool>
}

impl Scheduler{
    pub fn new(wait_queue: Vec<(u16, Process)>, ready_queue: Arc<Mutex<VecDeque<Task>>>, clock: Arc<AtomicU32>, time_quantum: u16, terminable: Arc<AtomicBool>) -> Self{
        Scheduler{
            wait_queue,
            ready_queue,
            clock,
            time_quantum,
            terminable
        }
    }
    pub async fn queue_task(&mut self, process: Process){
        let mut ready_queue = self.ready_queue.lock().await;
        println!("task queueing {}", process.pid);
        ready_queue.push_back(
            Task::new(
                process,
                self.clock.load(Acquire),
                self.time_quantum
            )
        );
    }

    pub async fn run(&mut self){
        while self.wait_queue.len() > 0{
            println!("still tasks left, {:?}. time is: {}", self.wait_queue, self.clock.load(Acquire));
            if (self.wait_queue.last().expect("could not get last element").0 as u32) <= self.clock.load(Acquire){
                let process = self.wait_queue.pop().expect("could not pop last element");
                self.queue_task(
                    process.1
                ).await;
            };
            sleep(Duration::from_millis(1000)).await;
        };
        self.terminable.store(true, Release);
    }
}