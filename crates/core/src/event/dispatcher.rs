use super::{Event, EventListener};
use async_trait::async_trait;
use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct EventError {
    pub message: String,
    pub errors: Vec<String>,
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} error(s)", self.message, self.errors.len())
    }
}

impl Error for EventError {}

#[async_trait]
trait ListenerHandler: Send + Sync {
    async fn call(&self, event: &(dyn std::any::Any + Send + Sync)) -> Result<(), Box<dyn Error>>;
    fn priority(&self) -> i32;
}

struct TypedListenerHandler<E: Event, L: EventListener<E>> {
    listener: L,
    _phantom: PhantomData<E>,
}

#[async_trait]
impl<E: Event + Clone, L: EventListener<E>> ListenerHandler for TypedListenerHandler<E, L> {
    async fn call(&self, event: &(dyn std::any::Any + Send + Sync)) -> Result<(), Box<dyn Error>> {
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.listener.handle(typed_event).await
        } else {
            Err("Type mismatch".into())
        }
    }

    fn priority(&self) -> i32 {
        self.listener.priority()
    }
}

#[derive(Clone)]
pub struct EventDispatcher {
    listeners: Arc<RwLock<HashMap<TypeId, Vec<Arc<dyn ListenerHandler>>>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register<E: Event + Clone, L: EventListener<E> + 'static>(&self, listener: L) {
        let type_id = TypeId::of::<E>();

        let handler: Arc<dyn ListenerHandler> = Arc::new(TypedListenerHandler {
            listener,
            _phantom: PhantomData,
        });

        let listeners = Arc::clone(&self.listeners);
        tokio::spawn(async move {
            let mut listeners_map = listeners.write().await;
            let list = listeners_map.entry(type_id).or_insert_with(Vec::new);
            list.push(handler);

            list.sort_by(|a, b| b.priority().cmp(&a.priority()));
        });
    }

    pub async fn dispatch<E: Event + Clone>(&self, event: E) -> Result<(), EventError> {
        let type_id = TypeId::of::<E>();

        let listeners = self.listeners.read().await;
        let listener_list = match listeners.get(&type_id) {
            Some(list) => list.clone(),
            None => return Ok(()),
        };
        drop(listeners);

        let mut errors = Vec::new();
        let event_any: &(dyn std::any::Any + Send + Sync) = &event;

        for handler in listener_list {
            match handler.call(event_any).await {
                Ok(()) => {}
                Err(e) => errors.push(e.to_string()),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(EventError {
                message: format!("Event '{}' dispatch failed", event.name()),
                errors,
            })
        }
    }

    pub async fn listener_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read().await;
        listeners.get(&type_id).map(|l| l.len()).unwrap_or(0)
    }

    pub async fn clear(&self) {
        let mut listeners = self.listeners.write().await;
        listeners.clear();
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[allow(dead_code)]
    #[derive(Clone)]
    struct TestEvent {
        value: i32,
    }

    impl Event for TestEvent {
        fn name(&self) -> &'static str {
            "test.event"
        }
    }

    struct TestListener {
        called: Arc<RwLock<bool>>,
    }

    #[async_trait]
    impl EventListener<TestEvent> for TestListener {
        async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn Error>> {
            let mut called = self.called.write().await;
            *called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dispatch_to_listener() {
        let dispatcher = EventDispatcher::new();
        let called = Arc::new(RwLock::new(false));

        let listener = TestListener {
            called: Arc::clone(&called),
        };
        dispatcher.register(listener);

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let event = TestEvent { value: 42 };
        dispatcher.dispatch(event).await.unwrap();

        assert!(*called.read().await);
    }

    #[tokio::test]
    async fn test_multiple_listeners() {
        let dispatcher = EventDispatcher::new();

        let called1 = Arc::new(RwLock::new(false));
        let called2 = Arc::new(RwLock::new(false));

        dispatcher.register(TestListener {
            called: Arc::clone(&called1),
        });
        dispatcher.register(TestListener {
            called: Arc::clone(&called2),
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let event = TestEvent { value: 42 };
        dispatcher.dispatch(event).await.unwrap();

        assert!(*called1.read().await);
        assert!(*called2.read().await);
    }
}
