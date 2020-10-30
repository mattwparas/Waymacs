use huelib::resource::Light;
use huelib::Bridge;
use std::net::{IpAddr, Ipv4Addr};

use std::thread;

extern crate crossbeam;
use crossbeam::channel::{unbounded, Receiver, Sender};

use std::env;
use std::sync::Mutex;
use std::time::Duration;

use crate::utilities::set_random_light;

const WORKER_THREADS: usize = 1;
const SLEEP_TIME: u64 = 200;

// This isn't amazing but it definitely gets the job done
lazy_static! {
    static ref BRIDGE: Bridge = Bridge::new(
        "192.168.1.2".parse().unwrap(),
        env::var("HUE_TOKEN").unwrap()
    );
    // worker thread stack
    static ref THREAD_STACK: Mutex<Vec<thread::JoinHandle<()>>> = Mutex::new(Vec::with_capacity(WORKER_THREADS));
    // Sender and receiver channels for sending the jobs to the worker threads
    static ref CHANNEL: (Mutex<Option<Sender<()>>>, Receiver<()>) = {
        let (s, r) = unbounded();
        (Mutex::new(Some(s)), r)
    };

    // Give all of the possible lights
    static ref CONTROL_LIGHTS: Vec<Light> = {
        let username = env::var("HUE_TOKEN").unwrap();
        let light_names = ["Hue play 1", "Hue play 2"];

        // Create a bridge with IP address and username.
        let bridge = Bridge::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)), username);

        let all_lights = match bridge.get_all_lights() {
            Ok(lights) => lights,
            Err(_) => {
                // eprintln!("Failed to get all lights: {}", e);
                return Vec::new();
            }
        };

        let mut control_lights: Vec<Light> = Vec::new();

        for light in &all_lights {
            if !light.state.reachable {
                continue;
            }

            if light_names
                .iter()
                .any(|&light_name| light_name == light.name)
            {
                control_lights.push(light.clone());
            }
        }

        control_lights
    };

}

pub fn spawn_workers() {
    for _ in 0..WORKER_THREADS {
        THREAD_STACK.lock().unwrap().push(thread::spawn(move || {
            while let Ok(()) = CHANNEL.1.recv() {
                send_to_bridge();
                thread::sleep(Duration::from_millis(SLEEP_TIME));
            }
        }));
    }
}

// Call this to change the light
pub fn change_random_light() {
    let _ = thread::spawn(move || {
        if let Some(sender) = CHANNEL.0.lock().unwrap().as_ref() {
            let _ = sender.send(());
        }
    });
}

fn send_to_bridge() {
    THREAD_STACK.lock().unwrap().push(thread::spawn(move || {
        set_random_light(&BRIDGE, &CONTROL_LIGHTS)
    }));
}

pub fn _join_all() {
    // give it some time to catch up and gracefully shut down;
    thread::sleep(Duration::from_millis(100));

    loop {
        if let Some(sender) = CHANNEL.0.lock().unwrap().as_ref() {
            if sender.len() == 0 {
                break;
            }
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }

    let _ = {
        CHANNEL.0.lock().unwrap().take();
    };

    while let Some(child) = THREAD_STACK.lock().unwrap().pop() {
        let _ = child.join();
    }
}
