use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use env_logger::TimestampPrecision;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    monitor::MonitorHandle,
    window::{Fullscreen, Window, WindowBuilder},
};

#[derive(Parser)]
struct Args {
    pub serial_port: String,
    pub baud: u32,
    pub bg_color: f64,
    pub stim_color: f64,
    pub monitor: usize,
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self, color: wgpu::Color) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// Enumerate monitors and prompt user to choose one
fn prompt_for_monitor(event_loop: &EventLoop<()>, idx: usize) -> MonitorHandle {
    for (num, monitor) in event_loop.available_monitors().enumerate() {
        log::debug!("Monitor #{}: {:?}", num, monitor.name());
    }

    event_loop
        .available_monitors()
        .nth(idx)
        .expect("Please enter a valid ID")
}

// 実験用のシリアルコマンドを送るためのラッパー
#[cfg(unix)]
struct Gpio(serialport::TTYPort);

#[cfg(windows)]
struct Gpio(serialport::COMPort);

impl Gpio {
    pub fn new(port_name: String, baud_rate: u32) -> Result<Self> {
        let port_builder =
            serialport::new(port_name, baud_rate).timeout(Duration::from_millis(100));
        #[cfg(unix)]
        let port = serialport::TTYPort::open(&port_builder)?;
        #[cfg(windows)]
        let port = serialport::COMPort::open(&port_builder)?;
        Ok(Self(port))
    }
    pub fn set_high(&mut self) -> Result<()> {
        match self.0.write(&[0x01]) {
            Ok(n) if n == 1 => {
                self.0.flush()?;
            }
            _ => panic!("Error writing to serial port"),
        }
        Ok(())
    }
    pub fn set_low(&mut self) -> Result<(), serialport::Error> {
        match self.0.write(&[0x00]) {
            Ok(n) if n == 1 => {
                self.0.flush()?;
            }
            _ => panic!("Error writing to serial port"),
        }
        Ok(())
    }
}

fn main() {
    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();
    let args = Args::parse();

    // シリアル通信の設定
    let port_name = args.serial_port;
    let baud_rate = args.baud;

    let mut gpio = Gpio::new(port_name, baud_rate).expect("Failed to open serial port");
    gpio.set_low().expect("Failed to set gpio low"); // 初期化

    // 描画ウィンドウの準備
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_visible(false)
        // .with_inner_size(PhysicalSize::new(1920u32, 1080u32))
        .build(&event_loop)
        .unwrap();

    // フルスクリーンで使う
    let monitor_idx = args.monitor;
    let monitor = prompt_for_monitor(&event_loop, monitor_idx);
    let refresh_rate = monitor.video_modes().next().unwrap().refresh_rate();
    window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
    window.set_cursor_visible(false);

    // ウィンドウが開いたら描画開始
    let mut state = pollster::block_on(State::new(&window));
    window.set_visible(true);

    // 塗りつぶす色の準備
    let bg_color = wgpu::Color {
        r: args.bg_color,
        g: args.bg_color,
        b: args.bg_color,
        a: 1.0,
    };
    let stim_color = wgpu::Color {
        r: args.stim_color,
        g: args.stim_color,
        b: args.stim_color,
        a: 1.0,
    };

    let mut last_frame_inst = Instant::now();

    // countが0の時だけ刺激を描画する
    let mut count = 0u16;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (color, set_high) = if count == 0 {
                    (stim_color, true)
                } else {
                    (bg_color, false)
                };
                count += 1;
                count %= refresh_rate;

                match state.render(color) {
                    Ok(_) => {
                        if set_high {
                            gpio.set_high().expect("Error writing to serial port");
                        } else {
                            gpio.set_low().expect("Error writing to serial port");
                        }
                    }
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {} //state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => log::error!("{:?}", e),
                };
                log::trace!("Render finished");
                if last_frame_inst.elapsed().as_millis() > 20 {
                    log::info!("Frame was skipped {:?}", last_frame_inst.elapsed());
                }
                last_frame_inst = Instant::now();
            }
            Event::MainEventsCleared => {
                log::trace!("MainEventsCleared");
                window.request_redraw();
            }
            _ => {}
        }
    });
}
