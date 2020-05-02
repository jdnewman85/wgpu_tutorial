use winit::event::*;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder};

use futures::executor;

fn main() {
    println!("Hello WGPU");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let mut size = window.inner_size();

    let surface = wgpu::Surface::create(&window);

    let adapter_options = &wgpu::RequestAdapterOptions{
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
    };
    let adapter_future = wgpu::Adapter::request( 
        adapter_options,
        wgpu::BackendBit::PRIMARY,
    );
    let adapter = executor::block_on(adapter_future).unwrap();
    let device_descriptor = &wgpu::DeviceDescriptor{
            extensions: wgpu::Extensions{
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        };
    let device_future = adapter.request_device(device_descriptor);
    let (device, queue) = executor::block_on(device_future);
    let mut sc_desc = wgpu::SwapChainDescriptor{
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);


    event_loop.run(move |event, _src_window, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent{ref event, window_id}
                if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput{input, .. } => {
                            match input {
                                KeyboardInput{state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. } => {
                                    *control_flow = ControlFlow::Exit
                                },
                                _ => {}
                            }
                        },
                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            size = **new_inner_size;
                            sc_desc.width = size.width;
                            sc_desc.height = size.height;
                            swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        }
                        WindowEvent::Resized(new_size) => {
                            size = *new_size;
                            sc_desc.width = size.width;
                            sc_desc.height =size.height;
                            swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        },
                        _ => {}
                    }
                },
            Event::RedrawRequested(_) => {
                let frame = swap_chain.get_next_texture().unwrap();
                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor{
                        label: Some("Render Encoder"),
                    }
                );
                {
                let _render_pass = encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor{
                        color_attachments: &[
                            wgpu::RenderPassColorAttachmentDescriptor{
                                attachment: &frame.view,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Clear,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.7,
                                    a: 1.0,
                                },
                            }
                        ],
                        depth_stencil_attachment: None,
                    }
                );
                }

                queue.submit(&[
                             encoder.finish(),
                ]);
            },
            Event::MainEventsCleared => { window.request_redraw(); },
            _ => {}
        }
    });
}
