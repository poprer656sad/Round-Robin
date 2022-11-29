pub struct Task{
    pub ctx_switch_counter: u8,
    pub creation_time: u32,
    pub process: Process,
    pub time_quantum: u16
}

impl Task{
    pub fn new(process: Process, creation_time: u32, time_quantum: u16) -> Task{
        Task{
            ctx_switch_counter: 0,
            creation_time,
            process,
            time_quantum
        }
    }

    pub fn execute(&mut self) -> Option<u16>{
        if self.process.burst <= self.time_quantum{
            self.process.burst = 0;
            return Some(self.time_quantum - self.process.burst)
        };

        self.process.burst -= self.time_quantum;
        None
    }
}

#[derive(Debug)]
pub struct Process{
    pub pid: u8, // pseudo pid tag. good for loggin
    pub burst: u16,
}

impl Process{
    pub fn new(process_num: u8, burst_time: u16) -> Self{
        Process { 
            pid: process_num,
            burst: burst_time
        }
    }
}