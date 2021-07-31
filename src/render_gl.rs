use gl;
use std;
use std::ffi::{CString, CStr};

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };
        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }
        unsafe { gl::LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(program_id, gl::LINK_STATUS, &mut success);
        };

        if success == 0 {
            let mut error_len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut error_len);
            }
            let error_msg: CString = create_whitespace_cstring_with_len(error_len as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    error_len,
                    std::ptr::null_mut(),
                    error_msg.as_ptr() as *mut gl::types::GLchar
                );
            }
            return Err(error_msg.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        Ok(Program { id: program_id })
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn id (&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_source(
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    source: &CStr,
    kind: gl::types::GLuint
) -> Result<gl::types::GLuint, String> {
    let shader = unsafe { gl::CreateShader(kind) };

    unsafe {
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    };

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    };

    if success == 0 {
        let mut error_len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut error_len);
        }
        let error_msg: CString = create_whitespace_cstring_with_len(error_len as usize);
        unsafe {
            gl::GetShaderInfoLog(
                shader,
                error_len,
                std::ptr::null_mut(),
                error_msg.as_ptr() as *mut gl::types::GLchar
            );
        }
        return Err(error_msg.to_string_lossy().into_owned());
    }

    Ok(shader)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}