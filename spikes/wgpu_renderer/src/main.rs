use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCHEMA: &str = "oathyard.production_renderer_manifest.v1";
const BACKEND_ID: &str = "wgpu-vulkan-offscreen-production-renderer-spike-v1";
const SHADER: &str = include_str!("verdict_ring.wgsl");

fn main() {
    if let Err(error) = real_main() {
        eprintln!("oathyard-wgpu-renderer-spike: {error}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let mut packet: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--packet" => packet = Some(PathBuf::from(next_arg(&mut args, "--packet")?)),
            "--out" => out = Some(PathBuf::from(next_arg(&mut args, "--out")?)),
            "--help" | "-h" => {
                println!("usage: oathyard-wgpu-renderer-spike --packet post_hash_presentation_packet.json --out <dir>");
                return Ok(());
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    let packet_path = packet.ok_or_else(|| "--packet is required".to_string())?;
    let out_dir = out
        .unwrap_or_else(|| PathBuf::from("artifacts/production_renderer/wgpu_spike/latest/render"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("create {}: {error}", out_dir.display()))?;

    let packet_text = fs::read_to_string(&packet_path)
        .map_err(|error| format!("read packet {}: {error}", packet_path.display()))?;
    let packet_json: Value = serde_json::from_str(&packet_text)
        .map_err(|error| format!("parse packet {}: {error}", packet_path.display()))?;
    if packet_json.get("schema").and_then(Value::as_str)
        != Some("oathyard.post_hash_presentation_packet.v1")
    {
        return Err(format!(
            "presentation packet has wrong schema: {:?}",
            packet_json.get("schema")
        ));
    }
    if packet_json
        .get("generated_after_replay_verify")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("presentation packet must be generated after replay verification".to_string());
    }

    let _bevy_ecs_world = bevy_ecs::world::World::new();
    let seed = seed_uniforms(&packet_json);
    let render = pollster::block_on(render_wgpu_frame(seed))?;
    let frame_path = out_dir.join("production_renderer_wgpu_spike_1920x1080.png");
    write_png_rgba(&frame_path, WIDTH, HEIGHT, &render.rgba)?;
    let frame_sha256 = sha256_file(&frame_path)?;
    let packet_sha256 = sha256_bytes(packet_text.as_bytes());

    let manifest = json!({
        "schema": SCHEMA,
        "product": "OATHYARD",
        "backend_id": BACKEND_ID,
        "renderer_stack": "bevy_ecs 0.19.0 + wgpu 29.0.3 direct Vulkan/offscreen texture spike",
        "bevy_wgpu_direction": "wgpu-first V1 spike under accepted Bevy/wgpu ADR 0009; Bevy app/window path remains a later adoption gate",
        "source": "post_hash_presentation_packet_after_replay_verify",
        "presentation_packet": packet_path.to_string_lossy(),
        "presentation_packet_sha256": packet_sha256,
        "scenario_id": packet_json.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        "content_hash": packet_json.get("content_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "final_state_hash": packet_json.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "replay_json_sha256": packet_json.get("replay_json_sha256").and_then(Value::as_str).unwrap_or("unknown"),
        "trace_json_sha256": packet_json.get("trace_json_sha256").and_then(Value::as_str).unwrap_or("unknown"),
        "adapter": render.adapter,
        "width": WIDTH,
        "height": HEIGHT,
        "frame_hash_chain": frame_sha256,
        "capture": {
            "capture_id": "production_renderer_wgpu_spike_1920x1080",
            "file": frame_path.to_string_lossy(),
            "width": WIDTH,
            "height": HEIGHT,
            "format": "png-rgba8",
            "capture_file_sha256": frame_sha256,
            "native_resolution": true,
            "upscaled_from_lower_resolution": false,
            "renderer_backend_id": BACKEND_ID,
            "source": "wgpu render pass from post-hash presentation packet",
            "truth_mutation": false
        },
        "wgpu_features": {
            "hardware_adapter_requested": true,
            "power_preference": "HighPerformance",
            "texture_usage": "TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC",
            "readback_path": "copy_texture_to_buffer",
            "shader": "spikes/wgpu_renderer/src/verdict_ring.wgsl"
        },
        "visual_features": {
            "procedural_3d_verdict_ring": true,
            "dynamic_key_fill_rim_lighting": true,
            "contact_shadows_ao_equivalent": true,
            "fog_atmosphere": true,
            "tone_mapping": true,
            "event_keyed_contact_bloom": true,
            "debug_text_overlay": false
        },
        "presentation_truth_isolation_passed": false,
        "presentation_only": true,
        "truth_mutation": false,
        "production_renderer_complete": false,
        "owner_visual_acceptance": false,
        "public_demo_ready": false,
        "release_candidate_ready": false,
        "legal_clearance": false,
        "trademark_clearance": false,
        "store_readiness": false,
        "native_windowed_execution": false,
        "native_windowed_execution_blocker": "V1 spike renders through real wgpu/Vulkan to an offscreen GPU texture for deterministic capture; native Bevy/winit window/swapchain adoption remains unimplemented and must not be claimed from this artifact.",
        "source_contract_literals": "production_renderer_complete: false; owner_visual_acceptance: false; presentation_truth_isolation_passed: true after wrapper verification; truth_mutation: false"
    });
    let manifest_path = out_dir.join("production_renderer_manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| format!("write {}: {error}", manifest_path.display()))?;
    let report_path = out_dir.join("production_renderer_report.md");
    write_report(&report_path, &packet_json, &manifest, &render.adapter)?;

    println!("wgpu production renderer spike written");
    println!("manifest={}", manifest_path.display());
    println!("frame={}", frame_path.display());
    Ok(())
}

fn next_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{name} requires a value"))
}

struct RenderResult {
    rgba: Vec<u8>,
    adapter: Value,
}

async fn render_wgpu_frame(seed: [f32; 4]) -> Result<RenderResult, String> {
    let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_desc.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(instance_desc);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .map_err(|error| format!("request high-performance Vulkan adapter: {error}"))?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("oathyard wgpu renderer spike device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|error| format!("request device: {error}"))?;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("oathyard wgpu verdict-ring render target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let uniform_bytes = seed
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<u8>>();
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard post-hash packet uniform"),
        size: uniform_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&uniform_buffer, 0, &uniform_bytes);
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("oathyard packet bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("oathyard packet bind group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("oathyard wgpu renderer spike pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("oathyard verdict-ring raymarch shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("oathyard wgpu production renderer spike pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });

    let bytes_per_pixel = 4u32;
    let bytes_per_row = WIDTH * bytes_per_pixel;
    let output_buffer_size = (bytes_per_row * HEIGHT) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard wgpu readback buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("oathyard wgpu renderer spike encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("oathyard wgpu verdict-ring render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.012,
                        g: 0.010,
                        b: 0.014,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result.map_err(|error| format!("map readback: {error}")));
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| format!("poll device: {error}"))?;
    receiver
        .recv()
        .map_err(|error| format!("readback callback lost: {error}"))??;
    let mapped = buffer_slice.get_mapped_range();
    let rgba = mapped.to_vec();
    drop(mapped);
    output_buffer.unmap();

    let adapter = json!({
        "name": info.name,
        "vendor": info.vendor,
        "device": info.device,
        "device_type": format!("{:?}", info.device_type),
        "backend": format!("{:?}", info.backend),
        "driver": info.driver,
        "driver_info": info.driver_info,
    });
    Ok(RenderResult { rgba, adapter })
}

fn seed_uniforms(packet: &Value) -> [f32; 4] {
    let material = format!(
        "{}:{}:{}:{}",
        packet
            .get("scenario_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("content_hash")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("final_state_hash")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("trace_json_sha256")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    let digest = Sha256::digest(material.as_bytes());
    let mut out = [0.0f32; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let base = i * 4;
        let word = u32::from_le_bytes([
            digest[base],
            digest[base + 1],
            digest[base + 2],
            digest[base + 3],
        ]);
        *slot = (word as f32) / (u32::MAX as f32);
    }
    out
}

fn write_png_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(format!(
            "unexpected RGBA byte count: got {} expected {}",
            rgba.len(),
            (width as usize) * (height as usize) * 4
        ));
    }
    let file =
        fs::File::create(path).map_err(|error| format!("create {}: {error}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|error| format!("png header {}: {error}", path.display()))?;
    png_writer
        .write_image_data(rgba)
        .map_err(|error| format!("png data {}: {error}", path.display()))?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn write_report(
    path: &Path,
    packet: &Value,
    manifest: &Value,
    adapter: &Value,
) -> Result<(), String> {
    let frame = manifest
        .get("capture")
        .and_then(|capture| capture.get("file"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let frame_sha = manifest
        .get("capture")
        .and_then(|capture| capture.get("capture_file_sha256"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let body = format!(
        "# OATHYARD wgpu production renderer spike\n\n\
Status: candidate renderer spike evidence only; no owner/public/release completion claim.\n\n\
- Backend: `{BACKEND_ID}`\n\
- Renderer API: `wgpu 29.0.3` high-performance Vulkan adapter request\n\
- Adapter: `{}` / `{}` / `{}`\n\
- Presentation packet schema: `{}`\n\
- Scenario: `{}`\n\
- Final truth hash: `{}`\n\
- Capture: `{frame}`\n\
- Capture sha256: `{frame_sha}`\n\
- Truth mutation: `false`\n\
- Production renderer complete: `false`\n\
- Owner visual acceptance: `false`\n\
- Native windowed execution: `false` (offscreen GPU texture spike; Bevy/winit swapchain remains a later gate)\n\n\
This artifact replaces neither the full high-fidelity renderer nor owner acceptance. It proves a source-buildable wgpu/Vulkan render pass can consume a verified post-hash presentation packet and emit a 1920x1080 PNG without touching truth.\n",
        adapter.get("name").and_then(Value::as_str).unwrap_or("unknown"),
        adapter.get("backend").and_then(Value::as_str).unwrap_or("unknown"),
        adapter.get("device_type").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("schema").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
    );
    fs::write(path, body).map_err(|error| format!("write {}: {error}", path.display()))
}
