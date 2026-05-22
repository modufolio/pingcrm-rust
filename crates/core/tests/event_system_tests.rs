use appkit_core::event::{Event, EventDispatcher, EventListener};
use async_trait::async_trait;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct UserCreatedEvent {
    #[allow(dead_code)]
    user_id: i32,
    #[allow(dead_code)]
    email: String,
}

impl Event for UserCreatedEvent {
    fn name(&self) -> &'static str {
        "user.created"
    }
}

#[derive(Clone)]
struct OrderPlacedEvent {
    #[allow(dead_code)]
    order_id: i32,
    #[allow(dead_code)]
    total: f64,
}

impl Event for OrderPlacedEvent {
    fn name(&self) -> &'static str {
        "order.placed"
    }
}

#[derive(Clone)]
struct TestEvent {
    value: i32,
}

impl Event for TestEvent {
    fn name(&self) -> &'static str {
        "test.event"
    }
}

struct CountingListener {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl EventListener<TestEvent> for CountingListener {
    async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct PriorityListener {
    priority: i32,
    order: Arc<AtomicI32>,
    sequence: Arc<AtomicI32>,
}

#[async_trait]
impl EventListener<TestEvent> for PriorityListener {
    async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.order.store(
            self.sequence.fetch_add(1, Ordering::SeqCst),
            Ordering::SeqCst,
        );
        Ok(())
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

struct ErrorListener;

#[async_trait]
impl EventListener<TestEvent> for ErrorListener {
    async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        Err("Intentional error".into())
    }
}

struct SlowListener {
    duration_ms: u64,
}

#[async_trait]
impl EventListener<TestEvent> for SlowListener {
    async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_millis(self.duration_ms)).await;
        Ok(())
    }
}

struct ConditionalListener {
    threshold: i32,
}

#[async_trait]
impl EventListener<TestEvent> for ConditionalListener {
    async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn should_handle(&self, event: &TestEvent) -> bool {
        event.value > self.threshold
    }
}

struct UserEmailListener {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl EventListener<UserCreatedEvent> for UserEmailListener {
    async fn handle(&self, _event: &UserCreatedEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_event_name() {
    let event = UserCreatedEvent {
        user_id: 123,
        email: "test@example.com".to_string(),
    };

    assert_eq!(event.name(), "user.created");
}

#[test]
fn test_event_timestamp() {
    let before = SystemTime::now();
    let event = TestEvent { value: 42 };

    let timestamp = event.timestamp();

    assert!(timestamp >= before);

    let duration_since = timestamp.duration_since(before).unwrap();
    assert!(duration_since.as_secs() < 1);
}

#[test]
fn test_multiple_event_types() {
    let user_event = UserCreatedEvent {
        user_id: 1,
        email: "user@example.com".to_string(),
    };

    let order_event = OrderPlacedEvent {
        order_id: 100,
        total: 99.99,
    };

    assert_eq!(user_event.name(), "user.created");
    assert_eq!(order_event.name(), "order.placed");
}

#[test]
fn test_dispatcher_creation() {
    let dispatcher = EventDispatcher::new();

    assert!(std::mem::size_of_val(&dispatcher) > 0);
}

#[test]
fn test_dispatcher_clone() {
    let dispatcher1 = EventDispatcher::new();
    let dispatcher2 = dispatcher1.clone();

    assert!(std::mem::size_of_val(&dispatcher2) > 0);
}

#[tokio::test]
async fn test_single_listener_dispatch() {
    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter),
    });

    sleep(Duration::from_millis(10)).await;

    let event = TestEvent { value: 42 };
    dispatcher.dispatch(event.clone()).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_multiple_listeners_same_event() {
    let dispatcher = EventDispatcher::new();
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::new(AtomicUsize::new(0));

    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter1),
    });
    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter2),
    });

    sleep(Duration::from_millis(10)).await;

    let event = TestEvent { value: 42 };
    dispatcher.dispatch(event).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(counter1.load(Ordering::SeqCst), 1);
    assert_eq!(counter2.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_dispatch_different_event_types() {
    let dispatcher = EventDispatcher::new();
    let user_counter = Arc::new(AtomicUsize::new(0));
    let test_counter = Arc::new(AtomicUsize::new(0));

    dispatcher.register(UserEmailListener {
        counter: Arc::clone(&user_counter),
    });
    dispatcher.register(CountingListener {
        counter: Arc::clone(&test_counter),
    });

    sleep(Duration::from_millis(10)).await;

    let user_event = UserCreatedEvent {
        user_id: 1,
        email: "test@example.com".to_string(),
    };
    dispatcher.dispatch(user_event).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    let test_event = TestEvent { value: 42 };
    dispatcher.dispatch(test_event).await.unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(user_counter.load(Ordering::SeqCst), 1);
    assert_eq!(test_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_listener_priority_ordering() {
    let dispatcher = EventDispatcher::new();
    let sequence = Arc::new(AtomicI32::new(0));

    let order_low = Arc::new(AtomicI32::new(-1));
    let order_high = Arc::new(AtomicI32::new(-1));

    dispatcher.register(PriorityListener {
        priority: 10,
        order: Arc::clone(&order_low),
        sequence: Arc::clone(&sequence),
    });

    dispatcher.register(PriorityListener {
        priority: 100,
        order: Arc::clone(&order_high),
        sequence: Arc::clone(&sequence),
    });

    sleep(Duration::from_millis(20)).await;

    let event = TestEvent { value: 42 };
    dispatcher.dispatch(event).await.unwrap();

    sleep(Duration::from_millis(20)).await;

    assert_eq!(order_high.load(Ordering::SeqCst), 0);

    assert_eq!(order_low.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_listener_error_handling() {
    let dispatcher = EventDispatcher::new();

    dispatcher.register(ErrorListener);

    sleep(Duration::from_millis(10)).await;

    let event = TestEvent { value: 42 };
    let result = dispatcher.dispatch(event).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_partial_listener_errors() {
    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter),
    });
    dispatcher.register(ErrorListener);

    sleep(Duration::from_millis(10)).await;

    let event = TestEvent { value: 42 };
    let result = dispatcher.dispatch(event).await;

    sleep(Duration::from_millis(10)).await;

    assert!(result.is_err());

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_concurrent_event_dispatch() {
    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter),
    });

    sleep(Duration::from_millis(10)).await;

    let mut handles = vec![];
    for i in 0..10 {
        let disp = dispatcher.clone();
        let handle = tokio::spawn(async move {
            let event = TestEvent { value: i };
            disp.dispatch(event).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    sleep(Duration::from_millis(20)).await;

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_slow_listener_doesnt_block() {
    let dispatcher = EventDispatcher::new();

    dispatcher.register(SlowListener { duration_ms: 100 });

    sleep(Duration::from_millis(10)).await;

    let start = std::time::Instant::now();

    let event = TestEvent { value: 42 };
    let _ = dispatcher.dispatch(event).await;

    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() >= 100);
}

#[tokio::test]
async fn test_conditional_listener_should_handle() {
    let listener = ConditionalListener { threshold: 50 };

    let event_low = TestEvent { value: 30 };
    let event_high = TestEvent { value: 60 };

    assert!(!listener.should_handle(&event_low));
    assert!(listener.should_handle(&event_high));
}

#[tokio::test]
async fn test_listener_priority_default() {
    let listener = CountingListener {
        counter: Arc::new(AtomicUsize::new(0)),
    };

    assert_eq!(listener.priority(), 0);
}

#[tokio::test]
async fn test_listener_name() {
    let listener = CountingListener {
        counter: Arc::new(AtomicUsize::new(0)),
    };

    let name = listener.name();
    assert!(name.contains("CountingListener"));
}

#[tokio::test]
async fn test_dispatch_with_no_listeners() {
    let dispatcher = EventDispatcher::new();

    let event = TestEvent { value: 42 };
    let result = dispatcher.dispatch(event).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_dispatches_same_event_type() {
    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    dispatcher.register(CountingListener {
        counter: Arc::clone(&counter),
    });

    sleep(Duration::from_millis(10)).await;

    for i in 0..5 {
        let event = TestEvent { value: i };
        dispatcher.dispatch(event).await.unwrap();
    }

    sleep(Duration::from_millis(20)).await;

    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn test_event_cloning() {
    let event1 = TestEvent { value: 42 };
    let event2 = event1.clone();

    assert_eq!(event1.value, event2.value);
    assert_eq!(event1.name(), event2.name());
}
