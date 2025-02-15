use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub mod auth;
pub use auth::{register_user, login_user, verify_token};

type Job = Box<dyn FnOnce() + Send + 'static>;
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    log::info!("Worker {id} is processing a job...");
                    job();
                }
                Err(_) => {
                    log::info!("Worker {id} is shutting down...");
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            if sender.send(Box::new(f)).is_err() {
                log::error!("ThreadPool: Worker queue disconnected, job not sent.");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            log::info!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    log::error!("Worker {} encountered an error: {:?}", worker.id, e);
                }
            }
        }
    }
}