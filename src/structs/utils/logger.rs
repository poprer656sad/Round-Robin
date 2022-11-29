pub use tokio::sync::mpsc;

#[derive(Debug)]
pub enum LoggerSignals{
    Message(u8, u8, u32, u32),
    Sigterm
}

pub struct Logger{
    pub rec_chan: mpsc::Receiver<LoggerSignals>
}

impl Logger{
    pub fn new(rx: mpsc::Receiver<LoggerSignals>) -> Self{
        Logger { 
            rec_chan: rx
        }
    }

    pub async fn run(&mut self){
        while let Some(val) = self.rec_chan.recv().await{
            match val{
                LoggerSignals::Message(pid, switches, start, end) => {
                    println!("{} ended with {} switches\t start: {} \t end:{}", pid, switches, start, end);
                },
                LoggerSignals::Sigterm =>{
                    self.rec_chan.close();
                    break
                }
            }
        }
    }
    
}