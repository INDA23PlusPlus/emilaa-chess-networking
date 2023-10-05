pub struct Shader {
    program_id: u32
}

#[allow(dead_code)]
impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        unsafe {
        let id: u32 = gl::CreateProgram();
        
        let vertex_code = Self::read_shader_code(vertex_path);
        let fragment_code = Self::read_shader_code(fragment_path);
        
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        
        gl::ShaderSource(vertex_shader, 1, &(vertex_code.as_str().as_bytes().as_ptr().cast()), &(vertex_code.as_str().len().try_into().unwrap()));
        gl::CompileShader(vertex_shader);
        Self::check_errors(vertex_shader, "VERTEX");

        gl::ShaderSource(fragment_shader, 1, &(fragment_code.as_str().as_bytes().as_ptr().cast()), &(fragment_code.as_str().len().try_into().unwrap()));
        gl::CompileShader(fragment_shader);
        Self::check_errors(fragment_shader, "FRAGMENT");

        gl::AttachShader(id, vertex_shader);
        gl::AttachShader(id, fragment_shader);
        gl::LinkProgram(id);
        Self::check_errors(id, "PROGRAM");
        
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        return Shader{ program_id: id };
        }
    }

    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.program_id); }
    }

    pub fn set_f32(&self, name: &str, value: f32) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::Uniform1f(gl::GetUniformLocation(self.program_id, target.as_ptr()), value);
        }
    }

    pub fn set_vec2(&self, name: &str, value: glm::Vec2) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::Uniform2fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, value.as_array().as_ptr().cast());
        }
    }

    pub fn set_vec3(&self, name: &str, value: glm::Vec3) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::Uniform3fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, value.as_array().as_ptr().cast());
        }
    }

    pub fn set_vec4(&self, name: &str, value: glm::Vec4) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::Uniform4fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, value.as_array().as_ptr().cast());
        }
    }

    pub fn set_mat2(&self, name: &str, value: glm::Mat2) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::UniformMatrix2fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, gl::FALSE, value.as_array().as_ptr().cast());
        }
    }

    pub fn set_mat3(&self, name: &str, value: glm::Mat3) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::UniformMatrix3fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, gl::FALSE, value.as_array().as_ptr().cast());
        }
    }

    pub fn set_mat4(&self, name: &str, value: glm::Mat4) {
        unsafe {
        let target = std::ffi::CString::new(name).expect("cstring");
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.program_id, target.as_ptr()), 1, gl::FALSE, value.as_array().as_ptr().cast());
        }
    }

    fn read_shader_code(path: &str) -> String {
        return std::fs::read_to_string(path).expect("Failed to read file...");
    }

    fn check_errors(shader: u32, shader_type: &str) {
        unsafe {
        let mut success: i32 = 0;
            
        if shader_type != "PROGRAM" {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl::GetShaderInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr().cast());
                v.set_len(log_len.try_into().unwrap());
                panic!("Shader compile error: {}", String::from_utf8_lossy(&v));
            }
        } else {
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl::GetProgramInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr().cast());
                v.set_len(log_len.try_into().unwrap());
                panic!("Program linking error: {}", String::from_utf8_lossy(&v));
            }
        }
        }
    } 
}