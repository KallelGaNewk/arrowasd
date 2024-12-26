use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use lazy_static::lazy_static;
use rdev::{grab, simulate, Event, EventType, Key};

lazy_static! {
    static ref SWITCHING: AtomicBool = AtomicBool::new(false);
    static ref PRESSED_KEYS: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
    static ref BIND: HashSet<Key> = HashSet::from([Key::KeyW, Key::AltGr]);
}

fn main() {
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let mut keys = PRESSED_KEYS.lock().unwrap();
            keys.insert(key);

            if keys.intersection(&BIND).count() == BIND.len() && BIND.contains(&key) {
                SWITCHING.store(!SWITCHING.load(Ordering::SeqCst), Ordering::SeqCst);
            }
        }
        EventType::KeyRelease(key) => {
            let mut keys = PRESSED_KEYS.lock().unwrap();
            keys.remove(&key);
        }
        _ => {}
    }

    if SWITCHING.load(Ordering::SeqCst) {
        match event.event_type {
            EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                let direction = match key {
                    Key::KeyW => Some(Key::UpArrow),
                    Key::KeyA => Some(Key::LeftArrow),
                    Key::KeyS => Some(Key::DownArrow),
                    Key::KeyD => Some(Key::RightArrow),
                    _ => None,
                };

                if let Some(direction_key) = direction {
                    let simulated_event = if matches!(event.event_type, EventType::KeyPress(_)) {
                        EventType::KeyPress(direction_key)
                    } else {
                        EventType::KeyRelease(direction_key)
                    };
                    let _ = simulate(&simulated_event);
                    None
                } else {
                    Some(event)
                }
            }
            _ => Some(event),
        }
    } else {
        Some(event)
    }
}
