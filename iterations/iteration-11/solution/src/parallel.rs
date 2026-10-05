use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

/// この計算機で同時に動かせるスレッドの数．わからなければ1を返す．
pub fn available_jobs() -> usize {
    thread::available_parallelism().map_or(1, NonZeroUsize::get)
}

/// `items`の各要素に`f`を当てはめた結果を，`items`と同じ順に返す．
/// `jobs`個(0なら1個)までのスレッドが，まだ処理していない要素を1つずつ取って処理する．
pub fn map_parallel<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    let workers = jobs.clamp(1, items.len().max(1));
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    let mut results: Vec<Option<R>> = items.iter().map(|_| None).collect();
    thread::scope(|scope| {
        for _ in 0..workers {
            let sender = sender.clone();
            let (next, f) = (&next, &f);
            scope.spawn(move || {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(i) else {
                        break;
                    };
                    sender.send((i, f(item))).unwrap();
                }
            });
        }
        // 自分の送信側を捨てる．すべてのスレッドが終わると送信側がなくなり，受信のループが終わる．
        drop(sender);
        for (i, result) in receiver {
            results[i] = Some(result);
        }
    });
    results
        .into_iter()
        .map(|result| result.expect("どの要素も1回ずつ処理される"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn results_keep_the_order_of_items() {
        let items: Vec<u64> = (0..100).collect();
        for jobs in [1, 2, 3, 8] {
            let squares = map_parallel(&items, jobs, |n| n * n);
            assert_eq!(squares, items.iter().map(|n| n * n).collect::<Vec<_>>());
        }
    }

    #[test]
    fn empty_items_give_empty_results() {
        let items: Vec<u64> = Vec::new();
        assert_eq!(map_parallel(&items, 4, |n| n + 1), Vec::<u64>::new());
    }

    #[test]
    fn zero_jobs_runs_on_one_thread() {
        assert_eq!(map_parallel(&[1, 2], 0, |n| n * 10), [10, 20]);
    }

    #[test]
    fn work_runs_on_several_threads() {
        let threads = Mutex::new(HashSet::new());
        let items: Vec<u64> = (0..64).collect();
        map_parallel(&items, 4, |_| {
            threads.lock().unwrap().insert(thread::current().id());
            thread::sleep(std::time::Duration::from_millis(1));
        });
        assert!(threads.lock().unwrap().len() > 1);
    }

    #[test]
    fn available_jobs_is_at_least_one() {
        assert!(available_jobs() >= 1);
    }
}
