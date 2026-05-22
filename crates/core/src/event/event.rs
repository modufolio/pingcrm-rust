use std::time::SystemTime;

pub trait Event: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn timestamp(&self) -> SystemTime {
        SystemTime::now()
    }

    fn metadata(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Clone)]
    struct TestEvent {
        data: String,
    }

    impl Event for TestEvent {
        fn name(&self) -> &'static str {
            "test.event"
        }
    }

    #[test]
    fn test_event_name() {
        let event = TestEvent {
            data: "test".to_string(),
        };
        assert_eq!(event.name(), "test.event");
    }

    #[test]
    fn test_event_timestamp() {
        let event = TestEvent {
            data: "test".to_string(),
        };
        let now = SystemTime::now();
        let ts = event.timestamp();

        assert!(ts >= now);
    }
}
