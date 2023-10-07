use glow::HasContext;

fn i8_slice_to_u8_slice(slice: &[i8]) -> &[u8] {
    unsafe { &*(slice as *const [i8] as *const [u8]) }
}

pub struct Program(glow::Program);

#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn as_raw(&self) -> u32 {
        match self {
            Self::Vertex => glow::VERTEX_SHADER,
            Self::Fragment => glow::FRAGMENT_SHADER,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Vertex => "vertex",
            Self::Fragment => "fragment",
        }
    }
}

impl std::fmt::Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Vertex => "vertex",
                Self::Fragment => "fragment",
            }
        )
    }
}

#[derive(Debug)]
pub enum ProgramCreationError {
    GlError(String),
    CompilationFailed(ShaderType, String),
    LinkingFailed(String),
}

impl Program {
    pub fn new(gl: &glow::Context, source: &ProgramSource) -> Result<Self, ProgramCreationError> {
        let shader_program = unsafe { gl.create_program().map_err(ProgramCreationError::GlError)? };
        if let Some(ref source) = source.vertex {
            let vertex_shader = Self::build_shader(gl, ShaderType::Vertex, source)?;
            unsafe { gl.attach_shader(shader_program, vertex_shader) };
        }

        if let Some(ref source) = source.fragment {
            let fragment_shader = Self::build_shader(gl, ShaderType::Fragment, source)?;
            unsafe { gl.attach_shader(shader_program, fragment_shader) };
        }
        Ok(Self(Self::link(gl, shader_program)?))
    }

    fn build_shader(
        gl: &glow::Context,
        shader_type: ShaderType,
        shader_source: &str,
    ) -> Result<glow::Shader, ProgramCreationError> {
        let shader_type_raw = shader_type.as_raw();
        let shader = unsafe {
            gl.create_shader(shader_type_raw)
                .map_err(ProgramCreationError::GlError)?
        };
        unsafe { gl.shader_source(shader, shader_source) };
        unsafe { gl.compile_shader(shader) };
        if unsafe { !gl.get_shader_compile_status(shader) } {
            return Err(ProgramCreationError::CompilationFailed(
                shader_type,
                unsafe { gl.get_shader_info_log(shader) },
            ));
        }
        Ok(shader)
    }

    fn link(
        gl: &glow::Context,
        shader_program: glow::Program,
    ) -> Result<glow::Program, ProgramCreationError> {
        unsafe { gl.link_program(shader_program) };

        if unsafe { !gl.get_program_link_status(shader_program) } {
            return Err(ProgramCreationError::LinkingFailed(unsafe {
                gl.get_program_info_log(shader_program)
            }));
        }
        Ok(shader_program)
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe { gl.use_program(Some(self.0)) };
    }

    pub fn handle(&self) -> glow::Program {
        self.0
    }

    pub fn close(&self, gl: &glow::Context) {
        unsafe { gl.delete_program(self.0) };
    }
}

pub struct ProgramBinding(u32);

impl Drop for ProgramBinding {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe {
            gl.use_program(None)
        };
    }
}

pub struct ProgramSource {
    vertex: Option<String>,
    fragment: Option<String>,
}

impl ProgramSource {
    pub fn new() -> Self {
        Self {
            vertex: None,
            fragment: None,
        }
    }

    pub fn vertex(mut self, source: String) -> Self {
        self.vertex = Some(source);
        self
    }

    pub fn fragment(mut self, source: String) -> Self {
        self.fragment = Some(source);
        self
    }
}
