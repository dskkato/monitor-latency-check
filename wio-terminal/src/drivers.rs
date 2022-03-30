use wio::hal::gpio::v2::{Disabled, Floating, Input, Output, Pin, PushPull, PA15, PC27};
use wio::prelude::*;
use wio_terminal as wio;

pub struct Button1 {
    pin: Pin<PC27, Input<Floating>>,
}

impl Button1 {
    pub fn new(pin: Pin<PC27, Disabled<Floating>>) -> Self {
        Self {
            pin: pin.into_floating_input(),
        }
    }
    pub fn is_pressed(&self) -> bool {
        self.pin.is_low().unwrap()
    }
    #[allow(dead_code)]
    pub fn is_released(&self) -> bool {
        self.pin.is_high().unwrap()
    }
}

pub struct Led {
    pin: Pin<PA15, Output<PushPull>>,
}

impl Led {
    pub fn new(pin: Pin<PA15, Disabled<Floating>>) -> Self {
        Self {
            pin: pin.into_push_pull_output(),
        }
    }
    pub fn turn_on(&mut self) {
        self.pin.set_high().unwrap()
    }
    pub fn turn_off(&mut self) {
        self.pin.set_low().unwrap()
    }
    #[allow(dead_code)]
    pub fn toggle(&mut self) {
        self.pin.toggle().unwrap()
    }
}
