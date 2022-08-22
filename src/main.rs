use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

use multiinput::*;

#[derive(Default)]
struct Pressed {
    throttle: AtomicBool,
    brake: AtomicBool,
    left: AtomicBool,
    right: AtomicBool,
}

fn main() {
    let client: vigem_client::Client = vigem_client::Client::connect().unwrap();
    let mut target = vigem_client::Xbox360Wired::new(client, vigem_client::TargetId::XBOX360_WIRED);

    target.plugin().unwrap();

    target.wait_ready().unwrap();

    let mut gamepad = vigem_client::XGamepad {
        ..Default::default()
    };

    let pressed = Arc::new(Pressed::default());

    {
        let pressed = Arc::clone(&pressed);

        thread::spawn(move || {
            let mut manager = RawInputManager::new().unwrap();
            manager.register_devices(DeviceType::Keyboards);
            let tobool = |state| match state {
                State::Pressed => true,
                State::Released => false,
            };
            loop {
                if let Some(RawEvent::KeyboardEvent(_, id, state)) = manager.get_event() {
                    let state = tobool(state);
                    let input = match id {
                        KeyId::A => Some(&pressed.left),
                        KeyId::D => Some(&pressed.right),
                        KeyId::W => Some(&pressed.throttle),
                        KeyId::S => Some(&pressed.brake),
                        _ => None,
                    };
                    if let Some(key) = input {
                        key.store(state, Ordering::SeqCst);
                    }
                }
            }
        });
    }

    let mut start = time::Instant::now();
    loop {
        let elapsed = start.elapsed();
        start = time::Instant::now();

        if pressed.left.load(Ordering::SeqCst) {
            let steer = gamepad.thumb_lx as f64 - 100000.0 * elapsed.as_secs_f64();
            gamepad.thumb_lx = if steer > i16::MIN as f64 {
                steer as i16
            } else {
                i16::MIN
            }
        } else if pressed.right.load(Ordering::SeqCst) {
            let steer = gamepad.thumb_lx as f64 + 100000.0 * elapsed.as_secs_f64();
            gamepad.thumb_lx = if steer < i16::MAX as f64 {
                steer as i16
            } else {
                i16::MAX
            }
        } else {
            gamepad.thumb_lx = 0;
        }

        gamepad.right_trigger = if pressed.throttle.load(Ordering::SeqCst) {
            u8::MAX
        } else {
            0
        };

        gamepad.left_trigger = if pressed.brake.load(Ordering::SeqCst) {
            u8::MAX
        } else {
            0
        };

        let _ = target.update(&gamepad);
        thread::sleep(time::Duration::from_secs_f64(1.0 / 150.0));
    }
}
