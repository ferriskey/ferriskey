use std::future::Future;
use std::sync::Arc;

use tokio::sync::{Mutex, mpsc, oneshot};

pub const WRITER_COUNT: usize = 4;
pub const MAX_BATCH: usize = 128;
const QUEUE_CAPACITY: usize = 4096;
const WRITER_STOPPED: &str = "the access token writer stopped";

pub trait RowSink: Send + Sync + 'static {
    type Row: Send + Sync + 'static;

    fn insert_many(&self, rows: &[Self::Row]) -> impl Future<Output = Result<(), String>> + Send;

    fn insert_one(&self, row: &Self::Row) -> impl Future<Output = Result<(), String>> + Send;
}

struct Pending<R> {
    row: R,
    reply: oneshot::Sender<Result<(), String>>,
}

#[derive(Debug)]
pub struct CoalescingWriter<R> {
    queue: mpsc::Sender<Pending<R>>,
}

impl<R: Send + Sync + 'static> CoalescingWriter<R> {
    pub fn spawn<S>(sink: Arc<S>) -> Self
    where
        S: RowSink<Row = R>,
    {
        let (queue, receiver) = mpsc::channel(QUEUE_CAPACITY);
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..WRITER_COUNT {
            tokio::spawn(write_loop(Arc::clone(&sink), Arc::clone(&receiver)));
        }

        Self { queue }
    }

    pub async fn write(&self, row: R) -> Result<(), String> {
        let (reply, outcome) = oneshot::channel();

        self.queue
            .send(Pending { row, reply })
            .await
            .map_err(|_| WRITER_STOPPED.to_string())?;

        outcome.await.map_err(|_| WRITER_STOPPED.to_string())?
    }

    #[cfg(test)]
    fn queued(&self) -> usize {
        QUEUE_CAPACITY - self.queue.capacity()
    }
}

async fn write_loop<S: RowSink>(
    sink: Arc<S>,
    receiver: Arc<Mutex<mpsc::Receiver<Pending<S::Row>>>>,
) {
    let mut taken = Vec::with_capacity(MAX_BATCH);

    loop {
        let count = receiver.lock().await.recv_many(&mut taken, MAX_BATCH).await;
        if count == 0 {
            return;
        }

        flush(sink.as_ref(), &mut taken).await;
    }
}

async fn flush<S: RowSink>(sink: &S, taken: &mut Vec<Pending<S::Row>>) {
    let (rows, replies): (Vec<S::Row>, Vec<_>) = taken
        .drain(..)
        .map(|pending| (pending.row, pending.reply))
        .unzip();

    match sink.insert_many(&rows).await {
        Ok(()) => {
            for reply in replies {
                let _ = reply.send(Ok(()));
            }
        }
        Err(error) if rows.len() == 1 => {
            for reply in replies {
                let _ = reply.send(Err(error.clone()));
            }
        }
        Err(_) => {
            for (row, reply) in rows.iter().zip(replies) {
                let _ = reply.send(sink.insert_one(row).await);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};

    use tokio::sync::Semaphore;
    use tokio::task::{JoinSet, yield_now};

    use super::{CoalescingWriter, RowSink, WRITER_COUNT};

    const POISON: u32 = 0;

    struct RecordingSink {
        gate: Semaphore,
        started: AtomicUsize,
        batches: StdMutex<Vec<Vec<u32>>>,
    }

    impl RecordingSink {
        fn open() -> Arc<Self> {
            Arc::new(Self {
                gate: Semaphore::new(usize::MAX >> 4),
                started: AtomicUsize::new(0),
                batches: StdMutex::new(Vec::new()),
            })
        }

        fn closed() -> Arc<Self> {
            Arc::new(Self {
                gate: Semaphore::new(0),
                started: AtomicUsize::new(0),
                batches: StdMutex::new(Vec::new()),
            })
        }

        fn started(&self) -> usize {
            self.started.load(Ordering::SeqCst)
        }

        fn batch_sizes(&self) -> Vec<usize> {
            let mut sizes: Vec<usize> = self.batches.lock().unwrap().iter().map(Vec::len).collect();
            sizes.sort_unstable();
            sizes
        }
    }

    impl RowSink for RecordingSink {
        type Row = u32;

        async fn insert_many(&self, rows: &[u32]) -> Result<(), String> {
            self.started.fetch_add(1, Ordering::SeqCst);
            self.gate.acquire().await.unwrap().forget();
            self.batches.lock().unwrap().push(rows.to_vec());
            if rows.contains(&POISON) {
                return Err("batch rejected".to_string());
            }
            Ok(())
        }

        async fn insert_one(&self, row: &u32) -> Result<(), String> {
            if *row == POISON {
                return Err("row rejected".to_string());
            }
            Ok(())
        }
    }

    async fn until(mut condition: impl FnMut() -> bool) {
        for _ in 0..10_000 {
            if condition() {
                return;
            }
            yield_now().await;
        }
        panic!("condition not reached");
    }

    async fn occupy_every_writer(
        writer: &Arc<CoalescingWriter<u32>>,
        sink: &Arc<RecordingSink>,
    ) -> JoinSet<Result<(), String>> {
        let mut calls = JoinSet::new();
        for row in 1..=WRITER_COUNT as u32 {
            let writer = Arc::clone(writer);
            calls.spawn(async move { writer.write(row).await });
            until(|| sink.started() == row as usize).await;
        }
        calls
    }

    #[tokio::test]
    async fn a_lone_row_is_written_alone_and_at_once() {
        let sink = RecordingSink::open();
        let writer = CoalescingWriter::spawn(Arc::clone(&sink));

        writer.write(7).await.unwrap();

        assert_eq!(sink.batch_sizes(), vec![1]);
    }

    #[tokio::test]
    async fn rows_arriving_while_every_writer_is_busy_share_one_batch() {
        let sink = RecordingSink::closed();
        let writer = Arc::new(CoalescingWriter::spawn(Arc::clone(&sink)));
        let mut calls = occupy_every_writer(&writer, &sink).await;

        for row in 100..110 {
            let writer = Arc::clone(&writer);
            calls.spawn(async move { writer.write(row).await });
        }
        until(|| writer.queued() == 10).await;

        sink.gate.add_permits(100);
        while let Some(done) = calls.join_next().await {
            done.unwrap().unwrap();
        }

        let mut expected = vec![1; WRITER_COUNT];
        expected.push(10);
        expected.sort_unstable();
        assert_eq!(sink.batch_sizes(), expected);
    }

    #[tokio::test]
    async fn a_rejected_batch_is_retried_row_by_row_so_only_the_bad_row_fails() {
        let sink = RecordingSink::closed();
        let writer = Arc::new(CoalescingWriter::spawn(Arc::clone(&sink)));
        let mut blockers = occupy_every_writer(&writer, &sink).await;

        let mut shared = JoinSet::new();
        for row in [11, POISON, 12] {
            let writer = Arc::clone(&writer);
            shared.spawn(async move { (row, writer.write(row).await) });
        }
        until(|| writer.queued() == 3).await;

        sink.gate.add_permits(100);
        while let Some(done) = blockers.join_next().await {
            done.unwrap().unwrap();
        }

        let mut outcomes = Vec::new();
        while let Some(done) = shared.join_next().await {
            let (row, outcome) = done.unwrap();
            outcomes.push((row, outcome.is_ok()));
        }
        outcomes.sort_unstable();
        assert_eq!(outcomes, vec![(POISON, false), (11, true), (12, true)]);
    }

    #[tokio::test]
    async fn a_lone_rejected_row_reports_the_sink_error() {
        let sink = RecordingSink::open();
        let writer = CoalescingWriter::spawn(Arc::clone(&sink));

        let outcome = writer.write(POISON).await;

        assert_eq!(outcome, Err("batch rejected".to_string()));
    }
}
