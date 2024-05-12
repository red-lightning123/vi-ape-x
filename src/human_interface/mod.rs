mod shader;

use crate::choose_matching_fbconfigs;
use crate::GlxContext;
use crate::Window;
use crate::X11Display;
use crate::{ImageOwned, ImageOwned2, ImageRef};
use glow::HasContext;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt as _;

fn build_shader_program(
    gl: &glow::Context,
) -> Result<shader::Program, shader::ProgramCreationError> {
    let vertex_source = include_str!("vertex.glsl");
    let fragment_source = include_str!("fragment.glsl");
    let source = shader::ProgramSource::new()
        .vertex(vertex_source.to_string())
        .fragment(fragment_source.to_string());
    shader::Program::new(gl, &source)
}

pub struct HumanInterface<'a> {
    display: X11Display<'a>,
    conn: x11rb::xcb_ffi::XCBConnection,
    win: u32,
    glx_context: GlxContext,
    gl: glow::Context,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    shader_program: shader::Program,
    frame: ImageOwned2,
    n_step: u32,
}

impl HumanInterface<'_> {
    pub fn new<'a>(ref_win: &Window) -> HumanInterface<'a> {
        let mut display = X11Display::open().expect("Can't open a connection to the X server");
        let conn = display
            .to_xcb_connection_mut()
            .expect("Can't convert display to xcb connection");
        let screen_num = display.default_screen();
        let screen = &conn.setup().roots[screen_num as usize];
        let root_win = screen.root;
        let win = conn.generate_id().unwrap();
        conn.create_window(
            ref_win.depth(),
            win,
            root_win,
            0,
            0,
            ref_win.width(),
            ref_win.height(),
            0,
            ref_win.class(),
            screen.root_visual,
            &xproto::CreateWindowAux::default(),
        )
        .unwrap();
        let win_title = b"Feed";
        conn.change_property(
            xproto::PropMode::REPLACE,
            win,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            8,
            win_title.len() as u32,
            win_title,
        )
        .unwrap();
        conn.change_window_attributes(
            win,
            &xproto::ChangeWindowAttributesAux::default()
                .event_mask(xproto::EventMask::KEY_PRESS | xproto::EventMask::KEY_RELEASE),
        )
        .unwrap();
        conn.map_window(win).unwrap();
        let fb_configs = choose_matching_fbconfigs(display.as_mut(), screen_num);
        let mut glx_context = display.create_glx_context(unsafe { *fb_configs });
        unsafe { x11::xlib::XFree(fb_configs.cast::<core::ffi::c_void>()) };
        display.make_glx_context_current(u64::from(win), &mut glx_context);
        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name_str| {
                x11::glx::glXGetProcAddressARB(name_str.as_ptr().cast::<u8>()).unwrap()
                    as *const core::ffi::c_void
            })
        };

        unsafe { gl.active_texture(glow::TEXTURE0) };
        let texture = unsafe { gl.create_texture().unwrap() };
        unsafe { gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };
        unsafe {
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            )
        };
        unsafe {
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            )
        };

        let vertices: [f32; 12] = [
            -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
        ];

        let vertices_u8_slice = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of::<f32>() * vertices.len(),
            )
        };

        let vbo = unsafe { gl.create_buffer().unwrap() };
        unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)) };
        unsafe {
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8_slice, glow::STATIC_DRAW)
        };

        let vao = unsafe { gl.create_vertex_array().unwrap() };
        unsafe { gl.bind_vertex_array(Some(vao)) };
        unsafe { gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0) };
        unsafe { gl.enable_vertex_attrib_array(0) };

        let shader_program = match build_shader_program(&gl) {
            Ok(shader_program) => shader_program,
            Err(err) => match err {
                shader::ProgramCreationError::GlError(e) => {
                    panic!("{}", e)
                }
                shader::ProgramCreationError::CompilationFailed(shader_type, log) => {
                    panic!(
                        "{} shader compilation failed with log:\n{}",
                        shader_type.name(),
                        log
                    )
                }
                shader::ProgramCreationError::LinkingFailed(log) => {
                    panic!("shader program linking failed with log:\n{log}")
                }
            },
        };
        shader_program.bind(&gl);
        unsafe {
            gl.uniform_1_i32(
                gl.get_uniform_location(shader_program.handle(), "tex")
                    .as_ref(),
                0,
            )
        };

        unsafe { gl.clear_color(0.2, 0.1, 0.2, 1.0) };
        HumanInterface {
            display,
            conn,
            win,
            glx_context,
            gl,
            vbo,
            vao,
            shader_program,
            frame: ImageOwned2::zeroed(0, 0),
            n_step: 0,
        }
    }
    pub fn terminate(mut self) {
        unsafe { self.gl.delete_buffer(self.vbo) };
        unsafe { self.gl.delete_vertex_array(self.vao) };
        self.shader_program.close(&self.gl);
        self.display.destroy_glx_context(self.glx_context);
        self.conn.destroy_window(self.win).unwrap();
        self.display.close();
    }
    pub fn poll_event(&mut self) -> Option<x11rb::protocol::Event> {
        self.conn.poll_for_event().unwrap()
    }
    pub fn swap_buffers(&mut self) {
        self.display.swap_buffers(u64::from(self.win));
    }
    fn scaled_dims(image_ref: &ImageOwned2) -> (f32, f32) {
        let x_scale_factor = 2.0 / (image_ref.width() as f32);
        let y_scale_factor = 2.0 / (image_ref.height() as f32);

        if x_scale_factor < y_scale_factor {
            (2.0, x_scale_factor * (image_ref.height() as f32))
        } else {
            (y_scale_factor * (image_ref.width() as f32), 2.0)
        }
    }
    pub fn clear_window(&self) {
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
    }
    pub fn draw(&self) {
        let (width, height) = Self::scaled_dims(&self.frame);
        unsafe {
            self.gl.uniform_2_f32(
                self.gl
                    .get_uniform_location(self.shader_program.handle(), "center")
                    .as_ref(),
                0.0,
                0.0,
            )
        };
        unsafe {
            self.gl.uniform_2_f32(
                self.gl
                    .get_uniform_location(self.shader_program.handle(), "dims")
                    .as_ref(),
                width,
                height,
            )
        };
        unsafe {
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RG as i32,
                self.frame.width() as i32,
                self.frame.height() as i32,
                0,
                glow::RG,
                glow::UNSIGNED_BYTE,
                Some(self.frame.as_ref().data()),
            )
        };
        unsafe { self.gl.draw_arrays(glow::TRIANGLES, 0, 6) };
    }
    pub fn set_frame(&mut self, frame: ImageOwned2) {
        self.frame = frame;
    }
    pub fn set_n_step(&mut self, n_step: u32) {
        self.n_step = n_step;
    }
}
