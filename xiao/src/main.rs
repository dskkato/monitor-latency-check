#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use panic_halt as _;
use xiao_m0 as bsp;

use bsp::hal::clock::GenericClockController;
use bsp::hal::gpio::v2::{Output, Pin, PushPull, PA02};
use bsp::hal::prelude::*;
use bsp::hal::usb::UsbBus;
use bsp::pac::{interrupt, CorePeripherals, Peripherals};
use bsp::{entry, Led0};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let pins = bsp::Pins::new(peripherals.PORT);

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    let d0 = pins.a0.into_push_pull_output();
    unsafe { D0 = Some(d0) };

    let led = pins.led0.into_push_pull_output();
    unsafe { LED = Some(led) };
    unsafe {
        D0.as_mut().map(|d0| d0.set_high().unwrap());
        LED.as_mut().unwrap().set_low().unwrap();
    }

    // Initialize USB.
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(bsp::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            pins.usb_dm,
            pins.usb_dp,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0xdead, 0xbeef))
                .manufacturer("Hackers University")
                .product("xiao_usb_echo")
                .serial_number("42")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    loop {}
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
static mut D0: Option<Pin<PA02, Output<PushPull>>> = None;
static mut LED: Option<Led0> = None;

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(serial) = USB_SERIAL.as_mut() {
                usb_dev.poll(&mut [serial]);
                let mut buf = [0u8; 32];

                match serial.read(&mut buf) {
                    Ok(count) if count == 1 => {
                        // if let Some(led) = LED.as_mut() {
                        //     led.set_high().unwrap();
                        // }
                        if buf[0] == b'0' {
                            D0.as_mut().map(|d0| d0.set_low().unwrap());
                            LED.as_mut().unwrap().set_high().unwrap();
                        } else if buf[0] == b'1' {
                            D0.as_mut().map(|d0| d0.set_high().unwrap());
                            LED.as_mut().unwrap().set_low().unwrap();
                        }
                    }
                    _ => {}
                };
            }
        }
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}
