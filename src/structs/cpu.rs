use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{
            AtomicU32,
            Ordering::Acquire, AtomicBool
        }
    }
};
use tokio::{
    time::{sleep, Duration},
    sync::{
        mpsc,
        Mutex
    }
};

use crate::{
    task_process::Task,
    logger::{
        LoggerSignals,
        Logger
    }
};

pub struct CpuStruct{
    /// CPU is asynchronous to scheduler. so the ready queue should not perform concurrent read writes.
    pub process_queue: Arc<Mutex<VecDeque<Task>>>,
    pub current_task: Option<Task>,

    // these parts are just for QOL and conceptualizing the system
    pub logger_chan: mpsc::Sender<LoggerSignals>,
    pub clock: Arc<AtomicU32>,

    pub time_quantum: u16,
    pub terminable: Arc<AtomicBool>,

}

impl CpuStruct{
    pub fn new(ready_queue: Arc<Mutex<VecDeque<Task>>>, clock: Arc<AtomicU32>, time_quantum: u16, terminable: Arc<AtomicBool>) -> Self{
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let mut logger = Logger::new(rx);
            logger.run().await;
        });
        CpuStruct {
            process_queue: ready_queue,
            current_task: None,

            logger_chan: tx,
            clock,
            time_quantum,
            terminable
        }
    }

    pub async fn fetch_next_task(&mut self){
        let mut ready_queue = self.process_queue.lock().await;
        match ready_queue.pop_front(){
            Some(next_task) =>{
                self.current_task = Some(next_task);
            },
            None => {
                if self.terminable.load(Acquire){
                    (self.logger_chan.send(LoggerSignals::Sigterm).await).expect("could not send sigterm")
                }
            },
        };
    }

    pub async fn run_task(&mut self){
        if let Some(mut task) = self.current_task.take(){
            println!("executing task {}", task.process.pid);
            match task.execute(){
                Some(early_return) => {
                    println!("task finished! {}", task.process.pid);
                    (self.logger_chan.send(
                        LoggerSignals::Message(
                            task.process.pid,
                                task.ctx_switch_counter,
                                task.creation_time,
                                self.clock.load(Acquire)
                        )
                    ).await).expect("error sending message");
                    drop(task);
                    self.clock.fetch_add(early_return as u32, Acquire);
                },
                None => {
                    println!("task requeued! {}", task.process.pid);
                    self.clock.fetch_add(self.time_quantum as u32, Acquire);
                    self.process_queue.lock().await.push_back(
                        task
                    )
                }
            }
        }else{
            println!("no task in queue, incrementing clock");
            self.clock.fetch_add(self.time_quantum as u32, Acquire);
        }
    }

    pub async fn run(&mut self){
        while !self.terminable.load(Acquire) || (self.process_queue.lock().await.len() > 0){
            self.fetch_next_task().await;
            self.run_task().await;
            sleep(Duration::from_millis(1000)).await;
        }
    }
}