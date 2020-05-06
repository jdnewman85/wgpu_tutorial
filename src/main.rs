use winit::event::*;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder};

use futures::executor;

use lyon::math::{point, Point};
use lyon::path::Path;
//use lyon::path::builder::*;
use lyon::tessellation::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}


const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.08682410,  0.49240386], tex_coords: [0.4131759000, 1.0 - 0.992403860] },
    Vertex { position: [-0.49513406,  0.06958647], tex_coords: [0.0048659444, 1.0 - 0.569586460] },
    Vertex { position: [-0.21918549, -0.44939706], tex_coords: [0.2808145300, 1.0 - 0.050602943] },
    Vertex { position: [ 0.35966998, -0.34732910], tex_coords: [0.8596700000, 1.0 - 0.152670890] },
    Vertex { position: [ 0.44147372,  0.23473590], tex_coords: [0.9414737000, 1.0 - 0.734735900] },
];
const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];


unsafe impl bytemuck::Pod for Vertex{}
unsafe impl bytemuck::Zeroable for Vertex{}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor{
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor{
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor{
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ]
        }
    }
}

fn main() {
    println!("Hello WGPU");

    // Eventloop and Window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let mut size = window.inner_size();

    // Surface
    let surface = wgpu::Surface::create(&window);

    // Adapter
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
    // Device and Queue
    let device_future = adapter.request_device(device_descriptor);
    let (device, mut queue) = executor::block_on(device_future);
    let mut sc_desc = wgpu::SwapChainDescriptor{
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    // Swapchain
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Load textures
    // *
    let diffuse_bytes = include_bytes!("../media/happy-tree.png");
    let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
    let diffuse_rgba = diffuse_image.as_rgba8().unwrap();
    use image::GenericImageView;
    let dimensions = diffuse_image.dimensions();

    let diffuse_size = wgpu::Extent3d{
        width: dimensions.0,
        height: dimensions.1,
        depth: 1,
    };
    let diffuse_texture = device.create_texture(
        &wgpu::TextureDescriptor{
            label: Some("diffuse_texture"),
            size: wgpu::Extent3d{
                width: dimensions.0,
                height: dimensions.1,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        }
    );
    let diffuse_buffer = device.create_buffer_with_data(
        &diffuse_rgba,
        wgpu::BufferUsage::COPY_SRC,
     );

    let mut encoder = device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor{
            label: Some("texture_buffer_copy_encoder"),
        }
    );
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView{
            buffer: &diffuse_buffer,
            offset: 0,
            bytes_per_row: 4 * dimensions.0,
            rows_per_image: dimensions.1,
        },
        wgpu::TextureCopyView{
            texture: &diffuse_texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        diffuse_size,
    );
    queue.submit(&[encoder.finish()]);

    let diffuse_texture_view = diffuse_texture.create_default_view();
    let diffuse_sampler = device.create_sampler(
        &wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        }
    );

    let texture_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor{
            label: Some("texture_bind_group_layout"),
            bindings: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture{
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler{
                        comparison: false,
                    },
                },
            ],
        }
    );

    //*
    let diffuse_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor{
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            bindings: &[
                wgpu::Binding{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::Binding{
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                }
            ],
        }
    );


    // Shaders
    let vs_src = include_str!("../shaders/triangle.vert");
    let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
    let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
    let vs_module = device.create_shader_module(&vs_data);

    let fs_src = include_str!("../shaders/triangle.frag");
    let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();
    let fs_data = wgpu::read_spirv(fs_spirv).unwrap();
    let fs_module = device.create_shader_module(&fs_data);

    // Pipeline
    let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor{bind_group_layouts: &[
            &texture_bind_group_layout,
        ]}
    );
    let render_pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor{
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor{
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor{
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor{
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[
                wgpu::ColorStateDescriptor{
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor{
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc(),
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        },
    );

    ///*
    let vertex_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(VERTICES),
        wgpu::BufferUsage::VERTEX,
    );

    let index_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(INDICES),
        wgpu::BufferUsage::INDEX,
    );
    let num_indices = INDICES.len() as u32;
    //*/
    //
    /*
    let geometry = build_path();
    let vertex_data = geometry.vertices;
    let index_data = geometry.indices;
    let vertex_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&vertex_data),
        wgpu::BufferUsage::VERTEX,
    );
    let num_vertices = vertex_data.len() as u32;
    dbg!(num_vertices);
    dbg!(vertex_data);

    let index_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&index_data),
        wgpu::BufferUsage::INDEX,
    );
    let num_indices = index_data.len() as u32;

    dbg!(num_indices);
    dbg!(index_data);
    */

    // MAIN EVENT LOOP
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
                        // Resize
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
                    // Renderpass
                    let mut render_pass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor{
                            color_attachments: &[
                                wgpu::RenderPassColorAttachmentDescriptor{
                                    attachment: &frame.view,
                                    resolve_target: None,
                                    load_op: wgpu::LoadOp::Clear,
                                    store_op: wgpu::StoreOp::Store,
                                    clear_color: wgpu::Color {
                                        r: 0.2,
                                        g: 0.2,
                                        b: 0.4,
                                        a: 1.0,
                                    },
                                }
                            ],
                            depth_stencil_attachment: None,
                        }
                    );

                    render_pass.set_bind_group(0, &diffuse_bind_group, &[]);
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_vertex_buffer(0, &vertex_buffer, 0, 0);
                    render_pass.set_index_buffer(&index_buffer, 0, 0);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                } //Stop borrowing encoder


                queue.submit(&[ encoder.finish(), ]);
            },
            Event::MainEventsCleared => { window.request_redraw(); },
            _ => {}
        }
    });
}

fn build_path() -> VertexBuffers<Vertex, u16> {
    let mut builder = Path::builder();
    builder.move_to(point( 0.0,  0.0));
    builder.line_to(point(-1.0,  0.0));
    builder.line_to(point(-1.0,  1.0));
    builder.line_to(point( 1.0,  1.0));
    builder.close();
    let path = builder.build();

    let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    {
        tessellator.tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| {
                Vertex {
                    position: pos.to_array(),
                    tex_coords: [0.0, 1.0],
                }
            }),
            ).unwrap();
    }

    return geometry;
}
