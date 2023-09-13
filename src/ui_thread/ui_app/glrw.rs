pub use gl;
pub struct Vao(u32);

fn i8_slice_to_u8_slice(slice : &[i8]) -> &[u8] {
    unsafe { &*(slice as *const [i8] as *const [u8]) }
}

impl Vao {
    pub fn new() -> Vao {
        let mut vao = Vao(0);
        unsafe { gl::GenVertexArrays(1, &mut vao.0) };
        vao
    }

    pub fn bind(&self) -> VaoBinding {
        unsafe { gl::BindVertexArray(self.0) };
        VaoBinding(self.0)
    }
    
    pub fn close(self) {
        unsafe { gl::DeleteVertexArrays(1, &self.0) }
    }
}

pub struct VaoBinding(u32);

impl Drop for VaoBinding {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe { gl::BindVertexArray(0) };
    }
}

pub struct Program(u32);

#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment
}

impl ShaderType {
    fn as_raw(&self) -> u32 {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER
        }
    }
    pub fn name(&self) -> &str {
        match self {
            ShaderType::Vertex => "vertex",
            ShaderType::Fragment => "fragment"
        }
    }
}

impl std::fmt::Display for ShaderType {
    fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            ShaderType::Vertex => "vertex",
            ShaderType::Fragment => "fragment"
        })
    }
}

#[derive(Debug)]
pub enum ProgramCreationError {
    CompilationFailed(ShaderType, String),
    LinkingFailed(String)
}

impl Program {
    pub fn new(source : &ProgramSource) -> Result<Program, ProgramCreationError> {
        let shader_program = unsafe { gl::CreateProgram() };
        if let Some(ref source) = source.vertex {
            let vertex_shader = Self::build_shader(ShaderType::Vertex, source)?;
            unsafe { gl::AttachShader(shader_program, vertex_shader) };
        }
        
        if let Some(ref source) = source.fragment {
            let fragment_shader = Self::build_shader(ShaderType::Fragment, source)?;
            unsafe { gl::AttachShader(shader_program, fragment_shader) };
        }
        Ok(Program(Self::link(shader_program)?))
    }
    
    fn build_shader(shader_type : ShaderType, shader_source : &[i8]) -> Result<u32, ProgramCreationError> {
        let shader_type_raw = shader_type.as_raw();
        let shader = unsafe { gl::CreateShader(shader_type_raw) };
        unsafe { gl::ShaderSource(shader, 1, &shader_source.as_ptr(), std::ptr::null()) };
        unsafe { gl::CompileShader(shader) };
        let mut success = 0;
        unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success) };
        if success == 0 {
            let mut compilation_log = [0i8; 512];
            unsafe { gl::GetShaderInfoLog(shader, 512, std::ptr::null_mut(), compilation_log.as_mut_ptr()) };
            return Err(ProgramCreationError::CompilationFailed(shader_type, String::from_utf8(i8_slice_to_u8_slice(compilation_log.as_slice()).to_vec()).unwrap()));
        }
        Ok(shader)
    }
    
    fn link(shader_program : u32) -> Result<u32, ProgramCreationError> {
        unsafe { gl::LinkProgram(shader_program) };

        let mut success = 0;
        unsafe { gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success) };
        if success == 0 {
            let mut linking_log = [0i8; 512];
            unsafe { gl::GetProgramInfoLog(shader_program, 512, std::ptr::null_mut(), linking_log.as_mut_ptr()) };
            return Err(ProgramCreationError::LinkingFailed(String::from_utf8(i8_slice_to_u8_slice(&linking_log).to_vec()).unwrap()));
        }
        Ok(shader_program)
    }

    pub fn bind(&self) -> ProgramBinding {
        unsafe { gl::UseProgram(self.0) };
        ProgramBinding(self.0)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
    
    pub fn close(self) {
        unsafe { gl::DeleteProgram(self.0) };
    }
}

pub struct ProgramBinding(u32);

impl Drop for ProgramBinding {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe { gl::UseProgram(0) };
    }
}

pub struct ProgramSource {
    vertex: Option<Vec<i8>>,
    fragment: Option<Vec<i8>>
}

impl ProgramSource {
    pub fn new() -> ProgramSource {
        ProgramSource {
            vertex: None,
            fragment: None
        }
    }

    pub fn vertex(mut self, source : Vec<i8>) -> ProgramSource {
        self.vertex = Some(source);
        self
    }

    pub fn fragment(mut self, source : Vec<i8>) -> ProgramSource {
        self.fragment = Some(source);
        self
    }
}
