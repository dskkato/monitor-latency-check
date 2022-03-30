#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use panic_halt as _;
use wio_terminal as wio;

use wio::hal::gpio::v2::{Output, Pin, PushPull, PA15, PB08};
use wio::hal::{clock::GenericClockController, usb::UsbBus};
use wio::pac::{interrupt, CorePeripherals, Peripherals};
use wio::prelude::*;
use wio::{entry, Usb};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let pins = wio::Pins::new(peripherals.PORT);

    // USB接続の場合にRPI_5V, RPI_3_3Vから給電出来るようにする
    let mut output_ctr_3v3 = pins.output_ctr_3v3.into_push_pull_output();
    output_ctr_3v3.set_low().unwrap();
    let mut output_ctr_5v = pins.output_ctr_5v.into_push_pull_output();
    output_ctr_5v.set_high().unwrap();
    let mut usb_host_enable = pins.usb_host_en.into_push_pull_output();
    usb_host_enable.set_low().unwrap();

    unsafe {
        core.NVIC.set_priority(interrupt::USB_OTHER, 1);
        core.NVIC.set_priority(interrupt::USB_TRCPT0, 1);
        core.NVIC.set_priority(interrupt::USB_TRCPT1, 1);
        NVIC::unmask(interrupt::USB_OTHER);
        NVIC::unmask(interrupt::USB_TRCPT0);
        NVIC::unmask(interrupt::USB_TRCPT1);
    }

    // Digital 0 output
    let d0 = pins.a0_d0.into_push_pull_output();
    unsafe { D0 = Some(d0) };

    let led = pins.user_led.into_push_pull_output();
    unsafe { LED = Some(led) };

    // Initialize USB.
    let usb = Usb {
        dm: pins.usb_dm,
        dp: pins.usb_dp,
    };
    // Initialize USB.
    let bus_allocator = unsafe {
        USB_ALLOCATOR =
            Some(usb.usb_allocator(peripherals.USB, &mut clocks, &mut peripherals.MCLK));
        USB_ALLOCATOR.as_ref().unwrap()
    };
    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    loop {}
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
static mut D0: Option<Pin<PB08, Output<PushPull>>> = None;
static mut LED: Option<Pin<PA15, Output<PushPull>>> = None;

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
                            D0.as_mut().unwrap().set_low().unwrap();
                            LED.as_mut().unwrap().set_low().unwrap();
                        } else if buf[0] == b'1' {
                            D0.as_mut().unwrap().set_high().unwrap();
                            LED.as_mut().unwrap().set_high().unwrap();
                        }
                    }
                    _ => {}
                };
            }
        }
    };
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}
