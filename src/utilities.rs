use huelib::color::Color;
use huelib::resource::{light, Light, Modifier, ModifierType};
use huelib::response::Response;
use huelib::Bridge;
use rand::Rng;
use std::vec::Vec;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps,
    NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSMenu, NSMenuItem,
    NSRunningApplication, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, selector, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSProcessInfo, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Protocol, Sel, BOOL};

#[allow(dead_code)]
fn get_random_color() -> Color {
    let mut range = rand::thread_rng();
    let red = range.gen::<u8>();
    let green = range.gen::<u8>();
    let blue = range.gen::<u8>();

    Color::from_rgb(red, green, blue)
}

#[allow(dead_code)]
fn get_random_brightness() -> u8 {
    let segments = 10;
    let max_brightness = 255;

    let max_segment_number = segments - 1;
    let multiplier = max_brightness / max_segment_number;

    let mut range = rand::thread_rng();

    let brightness_level = range.gen_range(0, segments);

    let brightness: u8;
    if brightness_level == max_segment_number {
        brightness = max_brightness;
    } else {
        brightness = brightness_level * multiplier;
    }

    return brightness;
}

pub fn set_random_light(bridge: &Bridge, lights: &Vec<Light>) {
    let mut range = rand::thread_rng();
    let element = range.gen_range(0, lights.len());

    light_on(&bridge, &lights[element], get_random_brightness());
}

#[allow(dead_code)]
fn light_on(bridge: &Bridge, light: &Light, brightness: u8) {
    let mut modifier = light::StateModifier::new()
        .on(true)
        .brightness(ModifierType::Override, brightness)
        .transition_time(0);

    if light.capabilities.control.color_gamut.is_some() {
        modifier = modifier.color(get_random_color());
    }

    match bridge.set_light_state(&light.id, &modifier) {
        Ok(v) => v.iter().for_each(|response| match response {
            Response::Success(_modified) => {}
            Response::Error(_error) => {}
        }),
        Err(_e) => {
            // eprintln!("Failed to modify the light state: {}", e);
            return;
        }
    };
}

#[allow(dead_code)]
fn light_off(bridge: &Bridge, light_id: impl AsRef<str>) {
    let modifier = light::StateModifier::new().on(false).transition_time(0);

    match bridge.set_light_state(light_id, &modifier) {
        Ok(v) => v.iter().for_each(|response| match response {
            Response::Success(v) => println!("OK {}", v),
            Response::Error(e) => eprintln!("Failed to modify the light state: {}", e),
        }),
        Err(e) => {
            eprintln!("Failed to modify the light state: {}", e);
            return;
        }
    };
}

pub fn show_purchase_prompt() {
    unsafe {
        let alert: *const Object = msg_send!(class!(NSAlert), alloc);
        let alert: *const Object = msg_send!(alert, init);

        let alert_title = NSString::alloc(nil)
            .init_str(&"Hello!".to_string())
            .autorelease();
        let alert_body = NSString::alloc(nil).init_str(&"Thanks for trying out Sublime Text.\r\n\nThis is an unregistered evaluation version, and although the trial is untimed, a license must be purchased for continued use.\n\rWould you like to purchase a license now?").autorelease();
        let alert_ok_button_text = NSString::alloc(nil)
            .init_str(&"Ok".to_string())
            .autorelease();
        let alert_purchase_button_text = NSString::alloc(nil)
            .init_str(&"Purchase".to_string())
            .autorelease();

        let _alert_id: id = msg_send!(alert, setMessageText: alert_title);
        let _alert_id: id = msg_send!(alert, setInformativeText: alert_body);
        let _alert_id: id = msg_send!(alert, addButtonWithTitle: alert_purchase_button_text);
        let _alert_id: id = msg_send!(alert, addButtonWithTitle: alert_ok_button_text);
        let _alert_id: id = msg_send!(alert, runModal);
    }
}
