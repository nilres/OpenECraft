//! The 3D view drawn by the graphics card (OpenGL 3 or WebGL 2 through egui_glow): the
//! [`Scene3d`] is uploaded when it changes, each frame only passes the camera. A depth
//! buffer takes the place of the painting order, multisampling smooths the edges; both
//! live in a framebuffer of the view's own, which is then copied into the window.
//! Without OpenGL 3 / WebGL 2, or after an error, the view is projected on the processor
//! again ([`super::view3d::project`]).
//!
//! The wires keep the rules of the painted view: they show through the cable ducts (the
//! ducts are drawn after them where no wire is, by the stencil buffer), walls, ceilings
//! and doors seen from the outside are translucent, the selected wire or route is on top.

use super::scene3d::{Scene3d, SceneLine, SceneSide};
use super::OUTER_WALLS;
use eframe::egui_glow;
use eframe::glow::{self, HasContext};
use egui::{Color32, Pos2, Rect, Vec2};
use openecraft_core::parts::vec3::{dot, sub, V3};
use std::sync::{Arc, Mutex};

/// Nearest depth drawn (mm), as by the processor.
const NEAR: f32 = 10.0;
/// Samples per pixel (at most what the graphics card offers).
const SAMPLES: i32 = 4;
/// Width of a stripe on a two-colour wire, of the wire's width.
const STRIPE: f32 = 0.45;

/// Camera of the 3D view: the eye, its axes and the focal length (points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Camera {
    pub eye: V3,
    pub right: V3,
    pub up: V3,
    pub fwd: V3,
    pub f: f32,
}

impl Camera {
    /// A point on the screen in a view centred on `center`, and its depth; `None` for a
    /// point behind the eye or too close to it.
    pub fn project(&self, center: Pos2, q: V3) -> Option<(Pos2, f32)> {
        let v = sub(q, self.eye);
        let z = dot(v, self.fwd);
        if z < NEAR {
            return None;
        }
        Some((center + egui::vec2(dot(v, self.right) / z * self.f, -dot(v, self.up) / z * self.f), z))
    }

    /// The point at depth `z` seen at `at` points from the centre of the view (the
    /// reverse of [`Camera::project`]).
    pub fn unproject(&self, at: Vec2, z: f32) -> V3 {
        let k = z / self.f;
        std::array::from_fn(|i| self.eye[i] + self.fwd[i] * z + self.right[i] * at.x * k - self.up[i] * at.y * k)
    }

    /// Matrix (column major) to the clip space of a view of `size` points that shows the
    /// depths `near` .. `far`; it projects as [`Camera::project`].
    pub fn matrix(&self, size: Vec2, near: f32, far: f32) -> [f32; 16] {
        let (fx, fy) = (2.0 * self.f / size.x.max(1.0), 2.0 * self.f / size.y.max(1.0));
        let (a, b) = ((far + near) / (far - near), -2.0 * far * near / (far - near));
        let row = |s: f32, d: V3, o: f32| [s * d[0], s * d[1], s * d[2], o - s * dot(d, self.eye)];
        let rows = [row(fx, self.right, 0.0), row(fy, self.up, 0.0), row(a, self.fwd, b), row(1.0, self.fwd, 0.0)];
        std::array::from_fn(|i| rows[i % 4][i / 4])
    }
}

/// Depths to draw for a scene within `bounds` seen from `eye`: from the box (or the
/// nearest depth) to its farthest corner.
fn depth_range((lo, hi): (V3, V3), eye: V3) -> (f32, f32) {
    let corner = |c: usize| [0, 1, 2].map(|k| if c >> k & 1 == 0 { lo[k] } else { hi[k] });
    let far = (0..8).map(|c| dot(sub(corner(c), eye), sub(corner(c), eye)).sqrt()).fold(0.0, f32::max) * 1.01 + 1.0;
    let gap = [0, 1, 2].map(|k| (lo[k] - eye[k]).max(eye[k] - hi[k]).max(0.0));
    let near = (dot(gap, gap).sqrt() * 0.9).max(NEAR);
    (near, far.max(near + 1.0))
}

/// How the faces of a wall or door are drawn from where the eye is.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Look {
    Hidden,
    Opaque,
    Translucent,
}

fn look(side: &SceneSide, eye: V3, hide_front: bool, opacity: f32) -> Look {
    if side.glass {
        return Look::Translucent;
    }
    match side.layer(eye, hide_front) {
        None => Look::Hidden,
        Some(OUTER_WALLS) if opacity <= 0.0 => Look::Hidden,
        Some(OUTER_WALLS) if opacity < 1.0 => Look::Translucent,
        Some(_) => Look::Opaque,
    }
}

/// Vertices from `first` on, `count` of them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Span {
    first: i32,
    count: i32,
}

/// Where the parts of a scene are in the vertex buffers.
#[derive(Debug, Default, PartialEq)]
struct Spans {
    /// Faces of everything that is neither a duct nor a wall: as they are, and those
    /// lying on another face.
    base: Span,
    decal: Span,
    ducts: Span,
    /// Faces of each wall or door ([`Scene3d::sides`]).
    sides: Vec<Span>,
    /// Lines among the equipment, the stripes of two-colour ones, lines on top.
    wires: Span,
    stripes: Span,
    top: Span,
}

/// Bytes per vertex of a face: position, normal (f32 × 3 each), colour (u8 × 4).
const FACE_STRIDE: i32 = 28;
/// Bytes per vertex of a line: both ends (f32 × 3 each), colour (u8 × 4) and the
/// parameters (f32 × 4: end, side, least width, diameter).
const LINE_STRIDE: i32 = 44;

/// Vertex data of a scene for the graphics card.
struct Geometry {
    faces: Vec<u8>,
    lines: Vec<u8>,
    spans: Spans,
}

fn put(v: &mut Vec<u8>, xs: &[f32]) {
    for x in xs {
        v.extend_from_slice(&x.to_ne_bytes());
    }
}

fn geometry(scene: &Scene3d) -> Geometry {
    // Buckets: base, decal, ducts, then one per side.
    let mut buckets: Vec<Vec<u8>> = vec![Vec::new(); 3 + scene.sides.len()];
    // The round tubes of the wires, cables and tubes with the other faces.
    for face in scene.faces.iter().chain(&scene.tubes) {
        let bucket = match face.side {
            Some(s) => 3 + s,
            None if face.obj.is_some_and(|o| scene.ducts[o]) => 2,
            None if face.decal => 1,
            None => 0,
        };
        let b = &mut buckets[bucket];
        let corners: &[usize] = if face.n == 4 { &[0, 1, 2, 0, 2, 3] } else { &[0, 1, 2] };
        for &i in corners {
            put(b, &face.pts[i]);
            put(b, &face.normal);
            b.extend_from_slice(&face.color.to_array());
        }
    }
    let mut faces = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    for b in buckets {
        spans.push(Span { first: faces.len() as i32 / FACE_STRIDE, count: b.len() as i32 / FACE_STRIDE });
        faces.extend(b);
    }
    let mut lines = Vec::new();
    let add = |lines: &mut Vec<u8>, l: &SceneLine, color: Color32| {
        // Two triangles across the segment: (end, side) of each corner.
        for (t, side) in [(0.0, -1.0), (0.0, 1.0), (1.0, 1.0), (0.0, -1.0), (1.0, 1.0), (1.0, -1.0)] {
            put(lines, &l.a);
            put(lines, &l.b);
            lines.extend_from_slice(&color.to_array());
            put(lines, &[t, side, l.least, l.diameter]);
        }
    };
    let span = |lines: &mut Vec<u8>, fill: &mut dyn FnMut(&mut Vec<u8>)| {
        let first = lines.len() as i32 / LINE_STRIDE;
        fill(lines);
        Span { first, count: lines.len() as i32 / LINE_STRIDE - first }
    };
    // Lines drawn as tubes are left out; a selected one stays on top as a thin line.
    let flat = || scene.wires.iter().filter(|w| !w.line.tube);
    let wires = span(&mut lines, &mut |v: &mut Vec<u8>| flat().for_each(|w| add(v, &w.line, w.line.color)));
    let stripes = span(&mut lines, &mut |v: &mut Vec<u8>| flat().for_each(|w| w.line.stripe.into_iter().for_each(|s| add(v, &w.line, s))));
    let top = span(&mut lines, &mut |v: &mut Vec<u8>| {
        for l in &scene.lines {
            let thin = SceneLine { diameter: 0.0, least: l.least.min(2.0), ..l.clone() };
            add(v, if l.tube { &thin } else { l }, l.color);
        }
    });
    Geometry { faces, lines, spans: Spans { base: spans[0], decal: spans[1], ducts: spans[2], sides: spans[3..].to_vec(), wires, stripes, top } }
}

/// The renderer shared with the paint callbacks; `failed` once it could not draw.
#[derive(Default)]
struct Shared {
    renderer: Option<Renderer>,
    failed: bool,
}

/// The graphics card renderer of a 3D view.
#[derive(Clone, Default)]
pub(super) struct Gpu {
    shared: Arc<Mutex<Shared>>,
}

impl Gpu {
    /// The graphics card cannot draw the view (no OpenGL 3 / WebGL 2, or an error).
    pub fn failed(&self) -> bool {
        self.shared.lock().map(|s| s.failed).unwrap_or(true)
    }

    /// A shape that draws `scene` into `rect`.
    pub fn shape(&self, ctx: &egui::Context, rect: Rect, scene: Arc<Scene3d>, frame: Frame) -> egui::Shape {
        let shared = self.shared.clone();
        let ctx = ctx.clone();
        let callback = egui_glow::CallbackFn::new(move |info, painter| {
            let Ok(mut s) = shared.lock() else { return };
            if s.failed {
                return;
            }
            if s.renderer.is_none() {
                match Renderer::new(painter.gl()) {
                    Ok(r) => s.renderer = Some(r),
                    Err(e) => {
                        log::warn!("3D view on the processor: {e}");
                        s.failed = true;
                        ctx.request_repaint();
                        return;
                    }
                }
            }
            let painted = s.renderer.as_mut().map(|r| r.paint(painter.gl(), &info, painter.intermediate_fbo(), &scene, &frame));
            if let Some(Err(e)) = painted {
                log::warn!("3D view on the processor: {e}");
                s.failed = true;
                ctx.request_repaint();
            }
        });
        egui::Shape::Callback(egui::PaintCallback { rect, callback: Arc::new(callback) })
    }
}

/// How a frame shows the scene: the camera, the opacity of the walls seen from the
/// outside, whether the walls in front are left out, the background.
pub(super) struct Frame {
    pub camera: Camera,
    pub opacity: f32,
    pub hide_front: bool,
    pub background: Color32,
}

/// Framebuffer of the view: multisampled colour, depth and stencil, and the texture it
/// is resolved into.
struct Target {
    size: (i32, i32),
    fbo: glow::Framebuffer,
    color: glow::Renderbuffer,
    depth: glow::Renderbuffer,
    resolved: glow::Framebuffer,
    texture: glow::Texture,
}

/// A shader program and the names of its uniforms (their locations are looked up when
/// used: in the browser they cannot be kept across threads).
struct Program {
    program: glow::Program,
    uniforms: &'static [&'static str],
}

impl Program {
    /// Compiles and links; `attributes` get the locations 0, 1, …
    fn new(gl: &glow::Context, prefix: &str, vertex: &str, fragment: &str, attributes: &[&str], uniforms: &'static [&'static str]) -> Result<Program, String> {
        unsafe {
            let program = gl.create_program()?;
            let mut shaders = Vec::new();
            for (kind, src) in [(glow::VERTEX_SHADER, vertex), (glow::FRAGMENT_SHADER, fragment)] {
                let shader = gl.create_shader(kind)?;
                gl.shader_source(shader, &format!("{prefix}{src}"));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    return Err(gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }
            for (i, name) in attributes.iter().enumerate() {
                gl.bind_attrib_location(program, i as u32, name);
            }
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            Ok(Program { program, uniforms })
        }
    }

    fn uniform(&self, gl: &glow::Context, i: usize) -> Option<glow::UniformLocation> {
        // SAFETY: a plain OpenGL call on the context egui paints with.
        unsafe { gl.get_uniform_location(self.program, self.uniforms[i]) }
    }
}

const FACE_VERTEX: &str = "
uniform mat4 u_matrix;
uniform vec3 u_eye;
in vec3 a_pos;
in vec3 a_normal;
in vec4 a_color;
out vec4 v_color;
out float v_facing;
void main() {
    gl_Position = u_matrix * vec4(a_pos, 1.0);
    v_color = a_color;
    v_facing = dot(a_normal, u_eye - a_pos);
}
";

const FACE_FRAGMENT: &str = "
uniform float u_alpha;
in vec4 v_color;
in float v_facing;
out vec4 o_color;
void main() {
    if (v_facing <= 0.0) {
        discard;
    }
    o_color = vec4(v_color.rgb, 1.0) * u_alpha;
}
";

/// A line is a band across the segment, as wide on the screen as the painted one.
const LINE_VERTEX: &str = "
uniform mat4 u_matrix;
uniform vec2 u_view;
uniform float u_f;
uniform float u_ppp;
uniform float u_scale;
uniform float u_bias;
uniform float u_near;
in vec3 a_a;
in vec3 a_b;
in vec4 a_color;
in vec4 a_params;
out vec4 v_color;
void main() {
    vec4 ca = u_matrix * vec4(a_a, 1.0);
    vec4 cb = u_matrix * vec4(a_b, 1.0);
    v_color = a_color;
    if (min(ca.w, cb.w) < u_near) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        return;
    }
    vec2 d = (cb.xy / cb.w - ca.xy / ca.w) * u_view;
    float len = length(d);
    vec2 n = len > 1e-4 ? vec2(-d.y, d.x) / len : vec2(0.0, 1.0);
    float z = 0.5 * (ca.w + cb.w);
    float w = a_params.w > 0.0 ? clamp(a_params.w * u_f / z, a_params.z, 24.0) : a_params.z;
    vec4 c = a_params.x < 0.5 ? ca : cb;
    c.xy += n * (a_params.y * 0.5 * w * u_scale * u_ppp) * 2.0 / u_view * c.w;
    c.z -= u_bias * c.w;
    gl_Position = c;
}
";

const LINE_FRAGMENT: &str = "
in vec4 v_color;
out vec4 o_color;
void main() {
    o_color = v_color;
}
";

/// The resolved view, a rectangle over the whole viewport.
const COPY_VERTEX: &str = "
out vec2 v_uv;
void main() {
    vec2 p = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
    v_uv = p;
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
";

const COPY_FRAGMENT: &str = "
uniform sampler2D u_texture;
in vec2 v_uv;
out vec4 o_color;
void main() {
    o_color = vec4(texture(u_texture, v_uv).rgb, 1.0);
}
";

// Uniforms of the programs, by their index.
const F_MATRIX: usize = 0;
const F_EYE: usize = 1;
const F_ALPHA: usize = 2;
const L_MATRIX: usize = 0;
const L_VIEW: usize = 1;
const L_F: usize = 2;
const L_PPP: usize = 3;
const L_SCALE: usize = 4;
const L_BIAS: usize = 5;
const L_NEAR: usize = 6;

struct Renderer {
    faces: Program,
    lines: Program,
    copy: Program,
    face_vao: glow::VertexArray,
    face_vbo: glow::Buffer,
    line_vao: glow::VertexArray,
    line_vbo: glow::Buffer,
    copy_vao: glow::VertexArray,
    /// Scene uploaded and where its parts are.
    uploaded: u64,
    spans: Spans,
    target: Option<Target>,
    samples: i32,
}

impl Renderer {
    fn new(gl: &glow::Context) -> Result<Renderer, String> {
        let version = egui_glow::ShaderVersion::get(gl);
        if !version.is_new_shader_interface() {
            return Err(format!("needs OpenGL 3 or WebGL 2, has {version:?}"));
        }
        let prefix = format!("{}{}", version.version_declaration(), if version.is_embedded() { "precision highp float;\n" } else { "" });
        // SAFETY: plain OpenGL calls on the context egui paints with, on its thread.
        unsafe {
            let faces = Program::new(gl, &prefix, FACE_VERTEX, FACE_FRAGMENT, &["a_pos", "a_normal", "a_color"], &["u_matrix", "u_eye", "u_alpha"])?;
            let lines = Program::new(gl, &prefix, LINE_VERTEX, LINE_FRAGMENT, &["a_a", "a_b", "a_color", "a_params"], &["u_matrix", "u_view", "u_f", "u_ppp", "u_scale", "u_bias", "u_near"])?;
            let copy = Program::new(gl, &prefix, COPY_VERTEX, COPY_FRAGMENT, &[], &["u_texture"])?;
            // (attribute, components, type, offset) of each layout.
            let layout = |vao: glow::VertexArray, vbo: glow::Buffer, stride: i32, attributes: &[(i32, u32, i32)]| {
                gl.bind_vertex_array(Some(vao));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                for (i, &(size, ty, offset)) in attributes.iter().enumerate() {
                    gl.enable_vertex_attrib_array(i as u32);
                    gl.vertex_attrib_pointer_f32(i as u32, size, ty, ty == glow::UNSIGNED_BYTE, stride, offset);
                }
                gl.bind_vertex_array(None);
            };
            let (face_vao, face_vbo) = (gl.create_vertex_array()?, gl.create_buffer()?);
            layout(face_vao, face_vbo, FACE_STRIDE, &[(3, glow::FLOAT, 0), (3, glow::FLOAT, 12), (4, glow::UNSIGNED_BYTE, 24)]);
            let (line_vao, line_vbo) = (gl.create_vertex_array()?, gl.create_buffer()?);
            layout(line_vao, line_vbo, LINE_STRIDE, &[(3, glow::FLOAT, 0), (3, glow::FLOAT, 12), (4, glow::UNSIGNED_BYTE, 24), (4, glow::FLOAT, 28)]);
            let copy_vao = gl.create_vertex_array()?;
            let samples = gl.get_parameter_i32(glow::MAX_SAMPLES).clamp(0, SAMPLES);
            Ok(Renderer { faces, lines, copy, face_vao, face_vbo, line_vao, line_vbo, copy_vao, uploaded: 0, spans: Spans::default(), target: None, samples })
        }
    }

    /// The framebuffer for a view of `size` pixels.
    fn target(&mut self, gl: &glow::Context, size: (i32, i32)) -> Result<&Target, String> {
        unsafe {
            if self.target.as_ref().is_some_and(|t| t.size != size) {
                if let Some(t) = self.target.take() {
                    gl.delete_framebuffer(t.fbo);
                    gl.delete_framebuffer(t.resolved);
                    gl.delete_renderbuffer(t.color);
                    gl.delete_renderbuffer(t.depth);
                    gl.delete_texture(t.texture);
                }
            }
            if self.target.is_none() {
                let (w, h) = size;
                let fbo = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                let buffer = |format: u32, attachment: u32| -> Result<glow::Renderbuffer, String> {
                    let rb = gl.create_renderbuffer()?;
                    gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rb));
                    gl.renderbuffer_storage_multisample(glow::RENDERBUFFER, self.samples, format, w, h);
                    gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, attachment, glow::RENDERBUFFER, Some(rb));
                    Ok(rb)
                };
                let color = buffer(glow::RGBA8, glow::COLOR_ATTACHMENT0)?;
                let depth = buffer(glow::DEPTH24_STENCIL8, glow::DEPTH_STENCIL_ATTACHMENT)?;
                gl.bind_renderbuffer(glow::RENDERBUFFER, None);
                let complete = gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
                let texture = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA8 as i32, w, h, 0, glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(None));
                for (name, value) in [(glow::TEXTURE_MIN_FILTER, glow::NEAREST), (glow::TEXTURE_MAG_FILTER, glow::NEAREST), (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE), (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE)] {
                    gl.tex_parameter_i32(glow::TEXTURE_2D, name, value as i32);
                }
                let resolved = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(resolved));
                gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(texture), 0);
                let complete = complete && gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
                gl.bind_texture(glow::TEXTURE_2D, None);
                self.target = Some(Target { size, fbo, color, depth, resolved, texture });
                if !complete {
                    return Err("incomplete framebuffer".into());
                }
            }
            self.target.as_ref().ok_or_else(String::new)
        }
    }

    fn paint(&mut self, gl: &glow::Context, info: &egui::PaintCallbackInfo, window: Option<glow::Framebuffer>, scene: &Scene3d, frame: &Frame) -> Result<(), String> {
        let viewport = info.viewport_in_pixels();
        let size = (viewport.width_px.max(1), viewport.height_px.max(1));
        let cam = &frame.camera;
        let (near, far) = depth_range(scene.bounds, cam.eye);
        let matrix = cam.matrix(info.viewport.size(), near, far);
        // Walls and doors drawn opaque, and the translucent ones far to near.
        let looks: Vec<Look> = scene.sides.iter().map(|s| look(s, cam.eye, frame.hide_front, frame.opacity)).collect();
        let mut translucent: Vec<usize> = (0..looks.len()).filter(|&i| looks[i] == Look::Translucent).collect();
        let distance = |i: usize| dot(sub(scene.sides[i].at, cam.eye), sub(scene.sides[i].at, cam.eye));
        translucent.sort_by(|&a, &b| distance(b).total_cmp(&distance(a)));
        // SAFETY: plain OpenGL calls on the context egui paints with, on its thread; the
        // state egui needs is set again after the callback.
        unsafe {
            if self.uploaded != scene.id {
                let g = geometry(scene);
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.face_vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &g.faces, glow::STATIC_DRAW);
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.line_vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &g.lines, glow::STATIC_DRAW);
                gl.bind_buffer(glow::ARRAY_BUFFER, None);
                (self.uploaded, self.spans) = (scene.id, g.spans);
            }
            let target = self.target(gl, size)?;
            let (fbo, resolved, texture) = (target.fbo, target.resolved, target.texture);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.viewport(0, 0, size.0, size.1);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::CULL_FACE);
            let [r, g, b, _] = frame.background.to_array().map(|c| c as f32 / 255.0);
            gl.clear_color(r, g, b, 1.0);
            gl.depth_mask(true);
            gl.stencil_mask(0xff);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            // Colours are premultiplied, as egui's.
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            let draw = |span: Span| {
                if span.count > 0 {
                    gl.draw_arrays(glow::TRIANGLES, span.first, span.count);
                }
            };
            let decal = |on: bool| {
                if on {
                    gl.enable(glow::POLYGON_OFFSET_FILL);
                    gl.polygon_offset(-1.0, -2.0);
                } else {
                    gl.disable(glow::POLYGON_OFFSET_FILL);
                }
            };
            let faces = &self.faces;
            let use_faces = |alpha: f32| {
                gl.use_program(Some(faces.program));
                gl.uniform_matrix_4_f32_slice(faces.uniform(gl, F_MATRIX).as_ref(), false, &matrix);
                gl.uniform_3_f32(faces.uniform(gl, F_EYE).as_ref(), cam.eye[0], cam.eye[1], cam.eye[2]);
                gl.uniform_1_f32(faces.uniform(gl, F_ALPHA).as_ref(), alpha);
                gl.bind_vertex_array(Some(self.face_vao));
            };
            let lines = &self.lines;
            let use_lines = |scale: f32, bias: f32| {
                gl.use_program(Some(lines.program));
                gl.uniform_matrix_4_f32_slice(lines.uniform(gl, L_MATRIX).as_ref(), false, &matrix);
                gl.uniform_2_f32(lines.uniform(gl, L_VIEW).as_ref(), size.0 as f32, size.1 as f32);
                gl.uniform_1_f32(lines.uniform(gl, L_F).as_ref(), cam.f);
                gl.uniform_1_f32(lines.uniform(gl, L_PPP).as_ref(), info.pixels_per_point);
                gl.uniform_1_f32(lines.uniform(gl, L_SCALE).as_ref(), scale);
                gl.uniform_1_f32(lines.uniform(gl, L_BIAS).as_ref(), bias);
                gl.uniform_1_f32(lines.uniform(gl, L_NEAR).as_ref(), NEAR);
                gl.bind_vertex_array(Some(self.line_vao));
            };
            let spans = &self.spans;
            // Everything solid but the ducts.
            use_faces(1.0);
            draw(spans.base);
            decal(true);
            draw(spans.decal);
            decal(false);
            for (i, span) in spans.sides.iter().enumerate().filter(|(i, _)| looks[*i] == Look::Opaque) {
                decal(scene.sides[i].decal);
                draw(*span);
            }
            decal(false);
            // The wires, marking where they are seen; then the ducts everywhere else.
            gl.enable(glow::STENCIL_TEST);
            gl.stencil_func(glow::ALWAYS, 1, 0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
            use_lines(1.0, 0.0);
            draw(spans.wires);
            use_lines(STRIPE, 2e-6);
            draw(spans.stripes);
            gl.stencil_func(glow::NOTEQUAL, 1, 0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
            use_faces(1.0);
            draw(spans.ducts);
            gl.disable(glow::STENCIL_TEST);
            // Walls seen from the outside, far to near, over everything.
            if !translucent.is_empty() {
                gl.depth_mask(false);
                for i in translucent {
                    use_faces(if scene.sides[i].glass { super::scene3d::GLASS_ALPHA } else { frame.opacity });
                    decal(scene.sides[i].decal);
                    draw(spans.sides[i]);
                }
                decal(false);
                gl.depth_mask(true);
            }
            gl.disable(glow::DEPTH_TEST);
            use_lines(1.0, 0.0);
            draw(spans.top);
            // Into the window.
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fbo));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(resolved));
            gl.blit_framebuffer(0, 0, size.0, size.1, 0, 0, size.0, size.1, glow::COLOR_BUFFER_BIT, glow::NEAREST);
            gl.bind_framebuffer(glow::FRAMEBUFFER, window);
            gl.viewport(viewport.left_px, viewport.from_bottom_px, size.0, size.1);
            let clip = info.clip_rect_in_pixels();
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(clip.left_px, clip.from_bottom_px, clip.width_px, clip.height_px);
            gl.disable(glow::BLEND);
            gl.use_program(Some(self.copy.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.uniform_1_i32(self.copy.uniform(gl, 0).as_ref(), 0);
            gl.bind_vertex_array(Some(self.copy_vao));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_vertex_array(None);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
