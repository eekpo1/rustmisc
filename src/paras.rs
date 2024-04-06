use std::fmt::{Debug, Display, Formatter, write};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::{JoinHandle, sleep, Thread};
use std::time::Duration;
use log::{debug, info, warn};

type Res = Box<dyn FnMut() + Send + 'static>;

#[derive(Debug)]
struct InvalidPoolSizeError;

impl Display for InvalidPoolSizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid thread pool size.")
    }
}

struct WorkerThread {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl WorkerThread {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Res>>>) -> WorkerThread {
        let thread = thread::spawn(move || loop {
            info!("Worker: {id} performing function");
            let mut task = receiver.lock().unwrap().recv();

            match task {
                Ok(mut task) => {
                    println!("Worker {id} executing job");
                    let ans = task();
                }
                Err(e) => {
                    debug!("worker {id} has no work to do. Shutting down");
                    // sleep(Duration::from_secs(5));
                }
            }
        });
        WorkerThread { id, thread: Some(thread) }
    }
}

struct ThreadPool {
    threads: Vec<WorkerThread>,
    sender: Option<mpsc::Sender<Res>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for t in &mut self.threads {
            if let Some(thread) = t.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    pub fn new(pool_size: usize) -> Result<ThreadPool, InvalidPoolSizeError> {
        let (sender, receiver) = mpsc::channel(); // Multiple sender single receiver
        let receiver = Arc::new(Mutex::new(receiver));
        match pool_size < 1 || pool_size > 256 {
            true => Err(InvalidPoolSizeError),
            false => {
                let mut threads = Vec::new();
                for i in 0..pool_size {
                    threads.push(WorkerThread::new(i, Arc::clone(&receiver)));
                }
                Ok(ThreadPool { threads, sender: Some(sender) })
            }
        }
    }

    pub fn run<T>(&self, fun: T)
        where
            T: FnMut() + Send + 'static {
        let mut task = Box::new(fun);
        self.sender.as_ref().unwrap().send(task).unwrap();
    }
}

pub struct RustPars {
    thread_pool: ThreadPool,
    pool_len: usize,
}

impl RustPars {
    pub fn new(size: usize) -> RustPars {
        info!("Started thread pool");
        RustPars { thread_pool: ThreadPool::new(size).unwrap(), pool_len: size }
    }

    pub fn map_parallel<T, U, C>(self, mut data: Vec<T>, mut fun: C) -> RustPars
        where T: Send + 'static + Sync + Debug,
              U: PartialEq + Copy + Debug + Send + 'static,
              C: FnMut(&T) -> Option<U> + Send + Copy + Sync + 'static
    {
        let data_size = data.len() as f32;
        let  partition_length = (data_size / (self.pool_len) as f32).ceil() as usize;
        println!("{}", partition_length);
        let mut chunks = vec![];
        for i in 1..self.pool_len {
            chunks.push(data.split_off(partition_length * (self.pool_len - i)));
            // println!("{:?}", chunks);
        }
        chunks.push(data);
        // let chunks = data.chunks((data_size / (self.pool_len) as f32).ceil() as usize).map(|x| Box::new(x)).collect::<Vec<Vec<T>>>();
        //
        for mut chunk in chunks {
            self.thread_pool.run(move || {
                let res: Vec<Option<U>> = chunk
                    .iter()
                    .map(|item| fun(item))
                    .filter(|x| !Option::eq(x, &None))
                    .collect();
                println!("{:?}", res.iter().map(|item| *item.as_ref().clone().unwrap()).collect::<Vec<U>>());
            })
        }
        self
    }

    pub fn drop(mut self) {
        drop(self);
    }

    // fn collect<T, U: PartialEq + Copy + Debug, C: FnMut(&T) -> Option<U>>(&self, data: Vec<T>, mut fun: C) -> Option<Vec<U>> {
    //     let res: Vec<Option<U>> = data
    //         .iter()
    //         .map(|item| fun(item))
    //         .filter(|x| Option::eq(x, &None) )
    //         .collect();
    //     // if (res.len() > 0) {
    //     //     return None
    //     // }
    //
    //     // return Some(res.iter().map(|item| *item.as_ref().clone().unwrap() ).collect())
    //     println!("{:?}",res.iter().map(|item| *item.as_ref().clone().unwrap() ).collect::<Vec<U>>());
    // }


    // pub fn map_parallel<T, C, U>(&mut self, mut data: Vec<T>, fun: C)
    //     where C: Fn(&mut T) + Send, T: std::fmt::Debug  {
    //     info!("attempting to map data: {:?}", data);
    //     let data_size = data.len() as f32;
    //     // let mut chunks = data.chunks((data_size / (self.pool_len) as f32).ceil() as usize);usize
    //     let ans = data.chunks((data_size / (self.pool_len) as f32).ceil() as usize)
    //         .into_iter()
    //         .map(|mut chunk| self.thread_pool.run( || {
    //             Self::collect(&mut chunk, fun);
    //         }));
    //     println!("{:?}", ans)
    // }
    //
    // fn collect<T, U, C: FnMut(&mut T)>(data: &mut [T], fun: C) -> Vec<U> {
    //     let d = data.into_iter().map(fun).collect::<Vec<U>>();
    //     d
    // }
}
