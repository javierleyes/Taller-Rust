use std::fmt;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// This struct represents a thread pool creation error
#[derive(Debug, Clone)]
pub struct PoolCreationError;

/// This struct represents a thread pool
impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to create thread pool. Invalid size")
    }
}

/// This struct represents a thread pool
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Returns a result with a new ThreadPool
    /// # Arguments
    ///
    /// * `size` - It indicates the max amount of working threads that the ThreadPool can handle
    ///
    /// # Examples
    ///
    /// ```
    /// use threadpool::ThreadPool;
    ///
    /// let pool = threadpool::ThreadPool::new(4);
    /// ```
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            Err(PoolCreationError)
        } else {
            let mut workers = Vec::with_capacity(size);
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }
            Ok(ThreadPool { workers, sender })
        }
    }

    /// Delegates the run of a function to the ThreadPool
    /// # Arguments
    ///
    /// * `f` - A function to be run using the ThreadPool
    ///
    /// # Examples
    ///
    /// ```
    /// use threadpool::ThreadPool;
    ///
    /// let pool = threadpool::ThreadPool::new(4);
    ///
    /// pool.spawn(move || {
    ///     // do something
    /// })
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Join and shutdown worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// This struct represents a thread worker
struct Worker {
    /// A numeric value to identify this worker
    id: usize,
    /// The JoinHandle that this worker will manage
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Returns a Worker
    /// # Arguments
    ///
    /// * `id` - A numeric identifier
    /// * `receiver` - An Arc that wraps a receiver containing a message to process by the worker
    ///
    /// # Examples
    ///
    /// ```
    /// let (sender, receiver) = mpsc::channel();
    /// let receiver = Arc::new(Mutex::new(receiver));
    /// let worker = Worker::new(id, Arc::clone(&receiver))
    /// ```
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Change this unwrap to an expect with an error message if needed
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    // println!("Worker {} got a job; executing.", id);

                    job();
                    // println!("Worker {} finished executing.", id);
                }
                Message::Terminate => {
                    // println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// An alias to represent a job that has to be processed by a Worker in a ThreadPool
type Job = Box<dyn FnOnce() + Send + 'static>;

/// An enum to represent a signal that tells what a Worker should do in a ThreadPool
enum Message {
    /// A signal that indicates a worker that new job must be run
    NewJob(Job),
    /// A signal that indicates a worker to terminate
    Terminate,
}
