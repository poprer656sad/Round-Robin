// Clock timestamps all events for processes, such as creation time, completion time, etc.
// Process Creator creates processes at arrival time
// CPU runs processes for a time slice (time quantum)
// Queue FIFO ready queue used by both the process creator and CPU
// Process Arrival Time arrival time of new processes into the ready queue
// Process Service Time amount of time required by the processes to complete execution
// Time Quantum time each process can spend in the CPU, before it is removed
// Context Switch number of times a process is switched

///////////////////////////////////////////////////////////
// Your program should also print out the following performance evaluation criteria:
// CPU Utilization
// Throughput
// Average Waiting Time
// Average Turnaround Time

mod structs;
use structs::*;

use csv::Reader;
use std::{
    env,
    sync::{
        atomic::{
            AtomicU32, AtomicBool
        },
        Arc,
    },
    collections::VecDeque,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let time_quantum: u16 = args[2].parse().unwrap();

    println!("time quantum {}", time_quantum);

    let mut file = Reader::from_path(&args[1]).expect("could not read csv file");

    println!("read file");

    let mut wait_queue: Vec<(u16, Process)> = 
    file.records().map(|record| {
        let vals = record.unwrap();
        return (
            vals[1].parse::<u16>().expect("could not deserialize csv record"),
            Process::new(
                vals[0].parse::<u8>().expect("could not deserialize csv record"),
                vals[2].parse::<u16>().expect("could not deserialize csv record")
            )
        )
    }).collect();

    println!("init wait queue");

    // reversed sorting. because Vec's only interface pop.
    // this is a language specific issue, nothing to do with lifo/fifo queues and what not
    wait_queue.sort_by(|(a, _), (b, _)| b.cmp(a));

    println!("sorted");

    let ready_queue = Arc::new(Mutex::new(VecDeque::<Task>::new()));
    let clock = Arc::new(AtomicU32::new(0));

    let termflag = Arc::new(AtomicBool::new(false));

    let mut scheduler = Scheduler::new(
        wait_queue,
        Arc::clone(&ready_queue),
        Arc::clone(&clock),
        time_quantum,
        Arc::clone(&termflag)
    );

    println!("init scheduler");

    let mut cpu = CpuStruct::new(
        Arc::clone(&ready_queue),
        Arc::clone(&clock),
        time_quantum,
        Arc::clone(&termflag)
    );

    println!("init cpu");

    let scheduler_thread = tokio::spawn(async move {
        scheduler.run().await
    });

    println!("spawned scheduler");

    let cpu_thread = tokio::spawn(async move {
        cpu.run().await
    });

    println!("spawned cpu");

    tokio::try_join!(scheduler_thread, cpu_thread).expect("could not join threads");
}
