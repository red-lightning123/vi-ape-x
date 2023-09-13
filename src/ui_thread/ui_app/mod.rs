use crate::{ MasterMessage, MasterThreadMessage, UiThreadMessage };
use crossbeam_channel::{ Sender, Receiver };
use glow::HasContext;
use eframe::egui;
use std::sync::{ Mutex, Arc };
use crate::{ ImageOwned2, ImageOwned, ImageRef };
mod shader;

fn build_shader_program(gl : &glow::Context) -> Result<shader::Program, shader::ProgramCreationError> {
    let vertex_source = include_str!("vertex.glsl");
    let fragment_source = include_str!("fragment.glsl");
    let source = shader::ProgramSource::new().vertex(vertex_source.to_string()).fragment(fragment_source.to_string());
    shader::Program::new(gl, &source)
}

struct UiState {
    vbo : glow::Buffer,
    vao : glow::VertexArray,
    shader_program : shader::Program,
    texture : glow::Texture,
    frame : ImageOwned2,
    n_step : u32
}

impl UiState {
    fn new(cc : &eframe::CreationContext, dims : (u32, u32)) -> UiState {
        let gl = cc
            .gl
            .as_ref()
            .expect("gl isn't present");

        let texture = unsafe { gl.create_texture().expect("gl couldn't create texture") };

        unsafe { gl.active_texture(glow::TEXTURE0) };
        unsafe { gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };
        unsafe { gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32) };
        unsafe { gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32) };

        let vertices : [f32; 12] = [
            -1.0, -1.0,
            -1.0, 1.0,
            1.0, -1.0,
            1.0, 1.0,
            -1.0, 1.0,
            1.0, -1.0
        ];
        let vertices_u8 : &[u8] = unsafe { std::slice::from_raw_parts(
            vertices.as_ptr().cast::<u8>(),
            std::mem::size_of::<f32>() * vertices.len()
        ) };

        let vbo = unsafe { gl.create_buffer().expect("gl couldn't create buffer") };
        unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo)) };
        unsafe { gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW) };

        let vao = unsafe { gl.create_vertex_array().expect("gl couldn't create vertex array") };
        unsafe { gl.bind_vertex_array(Some(vao)) };
        unsafe { gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0) };
        unsafe { gl.enable_vertex_attrib_array(0) };

        let shader_program = match build_shader_program(gl) {
            Ok(shader_program) => shader_program,
            Err(err) => match err {
                shader::ProgramCreationError::GlError(e) => {
                    panic!("{}", e);
                }
                shader::ProgramCreationError::CompilationFailed(shader_type, log) => {
                    panic!("{} shader compilation failed with log:\n{}", shader_type.name(), log)
                }
                shader::ProgramCreationError::LinkingFailed(log) => {
                    panic!("shader program linking failed with log:\n{log}")
                }
            }
        };
        shader_program.bind(gl);
        unsafe { gl.uniform_1_i32(gl.get_uniform_location(shader_program.handle(), "tex").as_ref(), 0) };

        unsafe { gl.clear_color(0.2, 0.1, 0.2, 1.0) };
        UiState {
            vbo,
            vao,
            shader_program,
            texture,
            frame: ImageOwned2::zeroed(dims.0, dims.1),
            n_step: 0
        }
    }
    fn close(&self, gl : &glow::Context) {
        unsafe { gl.delete_buffer(self.vbo) };
        unsafe { gl.delete_vertex_array(self.vao) };
        self.shader_program.close(gl);
        unsafe { gl.delete_texture(self.texture) };
    }
    fn scaled_dims(image : &ImageOwned2) -> (f32, f32) {
        let x_scale_factor = 2.0 / (image.width() as f32);
        let y_scale_factor = 2.0 / (image.height() as f32);
        
        if x_scale_factor < y_scale_factor {
            (2.0, x_scale_factor * (image.height() as f32))
        } else {
            (y_scale_factor * (image.width() as f32), 2.0)
        }
    }
    fn draw(&self, gl : &glow::Context) {
        let (width, height) = Self::scaled_dims(&self.frame);
        unsafe { gl.use_program(Some(self.shader_program.handle())) };
        unsafe { gl.bind_vertex_array(Some(self.vao)) };
        unsafe { gl.bind_texture(glow::TEXTURE_2D, Some(self.texture)) };
        unsafe { gl.uniform_2_f32(gl.get_uniform_location(self.shader_program.handle(), "center").as_ref(), 0.0, 0.0) };
        unsafe { gl.uniform_2_f32(gl.get_uniform_location(self.shader_program.handle(), "dims").as_ref(), width, height) };
        unsafe { gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RG as i32, self.frame.width() as i32, self.frame.height() as i32, 0, glow::RG, glow::UNSIGNED_BYTE, Some(self.frame.as_ref().data())) };
        unsafe { gl.draw_arrays(glow::TRIANGLES, 0, 6) };
    }
}

pub struct UiApp {
    receiver : Receiver<UiThreadMessage>,
    master_sender : Sender<MasterThreadMessage>,
    state : Arc<Mutex<UiState>>
}

impl UiApp {
    pub fn new(cc : &eframe::CreationContext, dims : (u32, u32), receiver : Receiver<UiThreadMessage>, master_sender : Sender<MasterThreadMessage>) -> UiApp {
        let state = Arc::new(Mutex::new(UiState::new(cc, dims)));
        UiApp {
            receiver,
            master_sender,
            state
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx : &egui::Context, _frame : &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for message in self.receiver.try_iter() {
                match message {
                    UiThreadMessage::Frame(frame) => {
                        let mut state = self.state.lock().unwrap();
                        state.frame = frame;
                    },
                    UiThreadMessage::NStep(n_step) => {
                        let mut state = self.state.lock().unwrap();
                        state.n_step = n_step;
                    }
                    UiThreadMessage::Master(message) => {
                        match message {
                            MasterMessage::Hold => {
                                unimplemented!();
                            }
                            _ => panic!("ui thread: bad message")
                        }
                    }
                    _ => panic!("ui thread: bad message")
                }
            }
            if ui.button("quit").clicked() {
                // TODO: terminate the application
            }
            {
                let state = self.state.lock().unwrap();
                ui.label(state.n_step.to_string());
            }
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let factor = 0.8;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(1920.0*factor, 1080.0*factor), egui::Sense::hover());
                let state = Arc::clone(&self.state);
                let callback = egui::PaintCallback {
                    rect,
                    callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                        state.lock().unwrap().draw(painter.gl());
                    }))
                };
                ui.painter().add(callback);
            });
            ui.ctx().request_repaint();
        });
    }
    fn on_exit(&mut self, gl : Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.state.lock().unwrap().close(gl);
        }
    }
}
