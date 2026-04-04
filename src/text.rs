use std::collections::HashMap;

use ab_glyph::{point, Font, FontArc, GlyphId, PxScale, ScaleFont};
use wgpu::util::DeviceExt;

const ATLAS_WIDTH: u32 = 1024;
const ATLAS_HEIGHT: u32 = 1024;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    clip_position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

impl TextVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct CachedGlyph {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    width_px: f32,
    height_px: f32,
    left_bearing: f32,
    top_bearing: f32,
    advance: f32,
}

pub struct TextRenderer {
    font: FontArc,
    atlas_texture: wgpu::Texture,
    atlas_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    glyph_cache: HashMap<(u16, u8), CachedGlyph>,
    atlas_next_x: u32,
    atlas_next_y: u32,
    atlas_row_height: u32,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        let font_bytes = std::fs::read("/System/Library/Fonts/Supplemental/Arial.ttf")?;
        let font = FontArc::try_from_vec(font_bytes)?;

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text_glyph_atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let zero = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &zero,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_WIDTH),
                rows_per_image: Some(ATLAS_HEIGHT),
            },
            wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("text_bind_group_layout"),
            });

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
            label: Some("text_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&text_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_capacity = 6 * 256;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text_vertex_buffer"),
            contents: bytemuck::cast_slice(&vec![
                TextVertex {
                    clip_position: [0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    color: [0.0, 0.0, 0.0, 0.0],
                };
                vertex_capacity
            ]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            font,
            atlas_texture,
            atlas_bind_group,
            pipeline,
            vertex_buffer,
            vertex_capacity,
            glyph_cache: HashMap::new(),
            atlas_next_x: 0,
            atlas_next_y: 0,
            atlas_row_height: 0,
        })
    }

    fn ensure_glyph_cached(
        &mut self,
        queue: &wgpu::Queue,
        ch: char,
        font_size: u8,
    ) -> Option<&CachedGlyph> {
        let scale = PxScale::from(font_size as f32);
        let scaled = self.font.as_scaled(scale);
        let glyph_id = scaled.glyph_id(ch);
        let cache_key = (glyph_id.0, font_size);

        if self.glyph_cache.contains_key(&cache_key) {
            return self.glyph_cache.get(&cache_key);
        }

        let glyph = glyph_id.with_scale_and_position(scale, point(0.0, 0.0));
        let advance = scaled.h_advance(glyph_id);

        let outlined = self.font.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();

        let width = bounds.width().ceil() as u32;
        let height = bounds.height().ceil() as u32;

        if width == 0 || height == 0 {
            let entry = CachedGlyph {
                uv_min: [0.0, 0.0],
                uv_max: [0.0, 0.0],
                width_px: 0.0,
                height_px: 0.0,
                left_bearing: bounds.min.x,
                top_bearing: bounds.min.y,
                advance,
            };
            self.glyph_cache.insert(cache_key, entry);
            return self.glyph_cache.get(&cache_key);
        }

        if self.atlas_next_x + width + 1 >= ATLAS_WIDTH {
            self.atlas_next_x = 0;
            self.atlas_next_y += self.atlas_row_height + 1;
            self.atlas_row_height = 0;
        }

        if self.atlas_next_y + height + 1 >= ATLAS_HEIGHT {
            return None;
        }

        let x = self.atlas_next_x;
        let y = self.atlas_next_y;
        self.atlas_next_x += width + 1;
        self.atlas_row_height = self.atlas_row_height.max(height);

        let mut pixels = vec![0u8; (width * height) as usize];
        outlined.draw(|gx, gy, coverage| {
            let idx = (gy * width + gx) as usize;
            pixels[idx] = (coverage * 255.0) as u8;
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let entry = CachedGlyph {
            uv_min: [
                x as f32 / ATLAS_WIDTH as f32,
                y as f32 / ATLAS_HEIGHT as f32,
            ],
            uv_max: [
                (x + width) as f32 / ATLAS_WIDTH as f32,
                (y + height) as f32 / ATLAS_HEIGHT as f32,
            ],
            width_px: width as f32,
            height_px: height as f32,
            left_bearing: bounds.min.x,
            top_bearing: bounds.min.y,
            advance,
        };

        self.glyph_cache.insert(cache_key, entry);
        self.glyph_cache.get(&cache_key)
    }

    fn measure_text_width(&self, text: &str, font_size: u8) -> f32 {
        let scale = PxScale::from(font_size as f32);
        let scaled = self.font.as_scaled(scale);
        let mut width = 0.0;
        let mut prev: Option<GlyphId> = None;

        for ch in text.chars() {
            let glyph_id = scaled.glyph_id(ch);
            if let Some(prev_id) = prev {
                width += scaled.kern(prev_id, glyph_id);
            }
            width += scaled.h_advance(glyph_id);
            prev = Some(glyph_id);
        }

        width
    }

    fn ensure_vertex_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.vertex_capacity {
            return;
        }

        let mut new_capacity = self.vertex_capacity.max(1);
        while new_capacity < needed {
            new_capacity *= 2;
        }

        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text_vertex_buffer"),
            contents: bytemuck::cast_slice(&vec![
                TextVertex {
                    clip_position: [0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    color: [0.0, 0.0, 0.0, 0.0],
                };
                new_capacity
            ]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.vertex_capacity = new_capacity;
    }

    fn screen_to_clip(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
        [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0]
    }

    pub fn render_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        screen_width: u32,
        screen_height: u32,
        text_items: &[(String, f32, f32, u8)],
    ) {
        if text_items.is_empty() {
            return;
        }

        let mut vertices = Vec::new();
        let screen_w = screen_width as f32;
        let screen_h = screen_height as f32;

        for (text, offset_x, offset_y, font_size) in text_items {
            let text_width = self.measure_text_width(text, *font_size);
            let mut pen_x = screen_w - *offset_x - text_width;

            let scale = PxScale::from(*font_size as f32);
            let font = self.font.clone();
            let scaled = font.as_scaled(scale);
            let baseline_y = *offset_y + scaled.ascent();
            let mut prev: Option<GlyphId> = None;

            for ch in text.chars() {
                if ch == '\n' {
                    continue;
                }

                let glyph_id = scaled.glyph_id(ch);
                if let Some(prev_id) = prev {
                    pen_x += scaled.kern(prev_id, glyph_id);
                }

                let cached = match self.ensure_glyph_cached(queue, ch, *font_size) {
                    Some(g) => g,
                    None => {
                        pen_x += scaled.h_advance(glyph_id);
                        prev = Some(glyph_id);
                        continue;
                    }
                };

                let x0 = pen_x + cached.left_bearing;
                let y0 = baseline_y + cached.top_bearing;
                let x1 = x0 + cached.width_px;
                let y1 = y0 + cached.height_px;

                if cached.width_px > 0.0 && cached.height_px > 0.0 {
                    let p0 = Self::screen_to_clip(x0, y0, screen_w, screen_h);
                    let p1 = Self::screen_to_clip(x1, y0, screen_w, screen_h);
                    let p2 = Self::screen_to_clip(x0, y1, screen_w, screen_h);
                    let p3 = Self::screen_to_clip(x1, y1, screen_w, screen_h);

                    let color = [0.0, 0.0, 0.0, 1.0];
                    let uv0 = [cached.uv_min[0], cached.uv_min[1]];
                    let uv1 = [cached.uv_max[0], cached.uv_min[1]];
                    let uv2 = [cached.uv_min[0], cached.uv_max[1]];
                    let uv3 = [cached.uv_max[0], cached.uv_max[1]];

                    vertices.extend_from_slice(&[
                        TextVertex {
                            clip_position: p0,
                            tex_coords: uv0,
                            color,
                        },
                        TextVertex {
                            clip_position: p2,
                            tex_coords: uv2,
                            color,
                        },
                        TextVertex {
                            clip_position: p1,
                            tex_coords: uv1,
                            color,
                        },
                        TextVertex {
                            clip_position: p2,
                            tex_coords: uv2,
                            color,
                        },
                        TextVertex {
                            clip_position: p3,
                            tex_coords: uv3,
                            color,
                        },
                        TextVertex {
                            clip_position: p1,
                            tex_coords: uv1,
                            color,
                        },
                    ]);
                }

                pen_x += cached.advance;
                prev = Some(glyph_id);
            }
        }

        if vertices.is_empty() {
            return;
        }

        self.ensure_vertex_capacity(device, vertices.len());
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.atlas_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}
