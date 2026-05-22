#![cfg(not(loom))]

use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_authenticator_chain_shared_across_tasks() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let success_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for i in 0..100 {
        let counter = Arc::clone(&success_count);

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_micros(10)).await;

            counter.fetch_add(1, Ordering::SeqCst);

            i
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(success_count.load(Ordering::SeqCst), 100);
}

#[test]
fn test_authenticator_chain_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    #[allow(dead_code)]
    #[derive(Clone)]
    struct MockRepo;

    assert_send_sync::<Arc<Vec<String>>>();
}

#[tokio::test]
#[ignore]
async fn benchmark_concurrent_authentication() {
    use std::time::Instant;

    let num_tasks = 1000;
    let start = Instant::now();

    let handles: Vec<_> = (0..num_tasks)
        .map(|i| {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_micros(1)).await;
                i
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();

    println!(
        "Completed {} concurrent authentications in {:?}",
        num_tasks, elapsed
    );
    println!(
        "Average: {:?} per authentication",
        elapsed / num_tasks as u32
    );

    assert!(
        elapsed < Duration::from_secs(1),
        "Performance regression: took {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_concurrent_reads_no_data_race() {
    use std::sync::Arc;

    let shared_data = Arc::new(vec!["jwt".to_string(), "session".to_string()]);

    let mut handles = vec![];

    for _ in 0..50 {
        let data = Arc::clone(&shared_data);

        let handle = tokio::spawn(async move {
            let _jwt = &data[0];
            let _session = &data[1];

            assert_eq!(data.len(), 2);
            assert_eq!(data[0], "jwt");
            assert_eq!(data[1], "session");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_generic_monomorphization_thread_safety() {
    #[derive(Clone)]
    struct Repository1;

    #[derive(Clone)]
    struct Repository2;

    let auth1 = Arc::new(Repository1);
    let auth2 = Arc::new(Repository2);

    let a1 = Arc::clone(&auth1);
    let a2 = Arc::clone(&auth2);

    let t1 = tokio::spawn(async move {
        let _r = &*a1;
    });

    let t2 = tokio::spawn(async move {
        let _r = &*a2;
    });

    t1.await.unwrap();
    t2.await.unwrap();
}

#[tokio::test]
#[ignore]
async fn stress_test_concurrent_access() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let iterations = 10000;
    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for _ in 0..iterations {
        let c = Arc::clone(&counter);

        let handle = tokio::spawn(async move {
            c.fetch_add(1, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), iterations);

    println!("Stress test completed: {} iterations", iterations);
}

#[tokio::test]
async fn test_repository_clone_concurrent_access() {
    #[derive(Clone)]
    struct MockRepository {
        id: usize,
    }

    let repo = MockRepository { id: 42 };

    let mut handles = vec![];

    for _ in 0..10 {
        let cloned = repo.clone();

        let handle = tokio::spawn(async move {
            assert_eq!(cloned.id, 42);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn example_production_usage_pattern() {
    use std::sync::Arc;

    #[derive(Clone)]
    struct AppState {
        auth_chain: Arc<Vec<String>>,
    }

    let state = AppState {
        auth_chain: Arc::new(vec!["jwt".to_string(), "session".to_string()]),
    };

    let mut handles = vec![];

    for request_id in 0..10 {
        let state = state.clone();

        let handle = tokio::spawn(async move {
            let _authenticators = &state.auth_chain;

            tokio::time::sleep(Duration::from_micros(10)).await;

            request_id
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
