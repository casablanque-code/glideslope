//! Synchronous publish/subscribe event bus.
//!
//! Deliberately simple for now: subscribers are plain closures called
//! in-line, in subscription order, on the thread that publishes. That's
//! enough for a single-threaded fixed-tick sim loop. If subsystems later
//! need to run concurrently, this is the seam where that would change —
//! nothing outside this file should assume synchronous delivery.

use crate::core::event::Event;

type Subscriber = Box<dyn Fn(&Event) + Send>;

#[derive(Default)]
pub struct EventBus {
    subscribers: Vec<Subscriber>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Register a closure to be called with every event published from
    /// this point forward. There is no unsubscribe yet — subscribers live
    /// as long as the bus does, which is fine while subsystems are set up
    /// once at startup.
    pub fn subscribe(&mut self, subscriber: impl Fn(&Event) + Send + 'static) {
        self.subscribers.push(Box::new(subscriber));
    }

    /// Deliver `event` to every current subscriber, in subscription order.
    pub fn publish(&self, event: Event) {
        for subscriber in &self.subscribers {
            subscriber(&event);
        }
    }

    #[allow(dead_code)] // diagnostics helper, will earn a call site once the UI shows sub counts
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::time::SimTime;
    use std::sync::{Arc, Mutex};

    #[test]
    fn subscribers_receive_published_events() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));

        let received_clone = Arc::clone(&received);
        bus.subscribe(move |event| {
            received_clone.lock().unwrap().push(format!("{:?}", event));
        });

        bus.publish(Event::Tick {
            at: SimTime::from_ticks(1),
        });
        bus.publish(Event::SimulationStopped {
            at: SimTime::from_ticks(2),
        });

        let received = received.lock().unwrap();
        assert_eq!(received.len(), 2);
    }

    #[test]
    fn multiple_subscribers_all_receive_the_same_event() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0));

        for _ in 0..3 {
            let count = Arc::clone(&count);
            bus.subscribe(move |_event| {
                *count.lock().unwrap() += 1;
            });
        }

        bus.publish(Event::SimulationStarted);
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn publishing_with_no_subscribers_does_not_panic() {
        let bus = EventBus::new();
        bus.publish(Event::SimulationStarted);
    }
}
