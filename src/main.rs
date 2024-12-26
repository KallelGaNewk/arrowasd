use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};

use lazy_static::lazy_static;
use rdev::{grab, simulate, Event, EventType, Key};

lazy_static! {
    static ref SWITCHING: AtomicBool = AtomicBool::new(false);
    static ref PRESSED_KEYS: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
    static ref BIND: HashSet<Key> = HashSet::from([Key::KeyW, Key::AltGr]);

    #[cfg(feature = "disable_esc_on_quote")]
    static ref ESC_DISABLED_UNTIL: Arc<Mutex<Option<SystemTime>>> = Arc::new(Mutex::new(None));
}

fn main() {
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) -> Option<Event> {
    #[cfg(feature = "disable_esc_on_quote")]
    {
        let mut esc_disabled_until = ESC_DISABLED_UNTIL.lock().unwrap();

        if let Some(disabled_until) = *esc_disabled_until {
            if disabled_until > SystemTime::now() {
                if let EventType::KeyPress(Key::Escape) = event.event_type {
                    return None;
                }
            } else {
                *esc_disabled_until = None;
            }
        }
    }

    match event.event_type {
        EventType::KeyPress(key) => {
            let mut keys = PRESSED_KEYS.lock().unwrap();
            keys.insert(key);

            if keys.intersection(&BIND).count() == BIND.len() && BIND.contains(&key) {
                SWITCHING.store(!SWITCHING.load(Ordering::SeqCst), Ordering::SeqCst);
            }
            println!("{key:?}");
            #[cfg(feature = "disable_esc_on_quote")]
            {
                if key == Key::BackQuote {
                    *ESC_DISABLED_UNTIL.lock().unwrap() = Some(SystemTime::now() + Duration::from_millis(300));
                }
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
