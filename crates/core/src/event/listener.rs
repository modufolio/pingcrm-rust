use super::Event;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait EventListener<E: Event>: Send + Sync {
    async fn handle(&self, event: &E) -> Result<(), Box<dyn Error>>;

    fn priority(&self) -> i32 {
        0
    }

    fn should_handle(&self, _event: &E) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    struct TestListener;

    #[async_trait]
    impl EventListener<TestEvent> for TestListener {
        async fn handle(&self, _event: &TestEvent) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_listener_defaults() {
        let listener = TestListener;
        let event = TestEvent { value: 42 };

        assert_eq!(listener.priority(), 0);
        assert!(listener.should_handle(&event));
        assert!(listener.handle(&event).await.is_ok());
    }
}
