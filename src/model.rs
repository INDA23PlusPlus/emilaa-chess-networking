use crate::shader;
use glm::*;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2
}

#[allow(dead_code)]
impl Transform {
    pub fn new() -> Transform {
        return Transform { 
            translation: Vec2{x: 0.0, y: 0.0}, 
            rotation: 0.0, 
            scale: Vec2{x: 1.0, y: 1.0}
        }
    }

    pub fn model(&self) -> Mat4 {
        let c = cos(self.rotation);
        let s0 = sin(self.rotation);
        let s1 = -sin(self.rotation);
        let sx = self.scale.x;
        let sy = self.scale.y;
        let tx = self.translation.x;
        let ty = self.translation.y;

        return mat4(
            sx*c,   sy*s0, 0.0, 0.0,
            -sx*s1, sy*c,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx, ty, 0.0, 1.0
        );
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Model2D {
    pub vao: u32,
    pub vao_len: u32,
    pub vbo: u32,
    pub color: Vec4,
    pub transform: Transform,
    pub default_color: Vec4
}

#[allow(dead_code)]
impl Model2D {
    fn tile(vertices: Vec<Vec2>, default_color: Vec4) -> Model2D {
        let mut vao: u32 = 0;
        let mut vbo: u32 = 0;
        
        unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<Vec2>()) as isize, vertices.as_ptr().cast(), gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vec2>().try_into().unwrap(), 0 as *const _);
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        }

        return Model2D {
            vao: vao,
            vao_len: vertices.len() as u32,
            vbo: vbo,
            color: default_color,
            transform: Transform::new(),
            default_color: default_color
        };
    }

    fn piece(vertices: Vec<(Vec2, Vec2)>) -> Model2D {
        let color: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);

        let mut vao: u32 = 0;
        let mut vbo: u32 = 0;
        
        unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 2 * std::mem::size_of::<Vec2>()) as isize, vertices.as_ptr().cast(), gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (std::mem::size_of::<Vec2>() * 2).try_into().unwrap(), 0 as *const _);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (std::mem::size_of::<Vec2>() * 2).try_into().unwrap(), 8 as *const _);
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        }

        return Model2D {
            vao: vao,
            vao_len: vertices.len() as u32,
            vbo: vbo,
            color: color,
            transform: Transform::new(),
            default_color: color
        };
    }

    pub fn character() -> Model2D {
        let color: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);

        let mut vao: u32 = 0;
        let mut vbo: u32 = 0;
        
        unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (6 * 4 * std::mem::size_of::<f32>()) as isize, 0 as *const _, gl::DYNAMIC_DRAW);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (std::mem::size_of::<f32>() * 4).try_into().unwrap(), 0 as *const _);
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        }

        return Model2D {
            vao: vao,
            vao_len: 4,
            vbo: vbo,
            color: color,
            transform: Transform::new(),
            default_color: color
        };
    }

    pub fn draw(&self, program: &shader::Shader) {
        unsafe {
        program.set_mat4("model", self.transform.model());
        program.set_vec4("color", self.color);

        gl::BindVertexArray(self.vao);
        gl::DrawArrays(gl::TRIANGLES, 0, self.vao_len as i32);
        }
    }

    pub fn black_tile() -> Model2D {
        let vertices: [Vec2; 6] = [
            Vec2{ x:  0.5, y:  0.5 },
            Vec2{ x:  0.5, y: -0.5 },
            Vec2{ x: -0.5, y: -0.5 },

            Vec2{ x: -0.5, y: -0.5 },
            Vec2{ x: -0.5, y:  0.5 },
            Vec2{ x:  0.5, y:  0.5 }
        ];

        return Self::tile(vertices.try_into().unwrap(), Vec4{x: 0.5, y: 0.3, z: 0.2, w: 1.0});
    }

    pub fn white_tile() -> Model2D {
        let vertices: [Vec2; 6] = [
            Vec2{ x:  0.5, y:  0.5 },
            Vec2{ x:  0.5, y: -0.5 },
            Vec2{ x: -0.5, y: -0.5 },

            Vec2{ x: -0.5, y: -0.5 },
            Vec2{ x: -0.5, y:  0.5 },
            Vec2{ x:  0.5, y:  0.5 }
        ];

        return Self::tile(vertices.try_into().unwrap(), Vec4{x: 0.9, y: 0.7, z: 0.5, w: 1.0});
    }

    pub fn white_piece(piece: i8) -> Model2D {
        let vertices: [(Vec2, Vec2); 6] = [
            (Vec2{ x:  0.5, y:  0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 1.0 }),
            (Vec2{ x:  0.5, y: -0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 0.5 }),
            (Vec2{ x: -0.5, y: -0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 0.5 }),

            (Vec2{ x: -0.5, y: -0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 0.5 }),
            (Vec2{ x: -0.5, y:  0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 1.0 }),
            (Vec2{ x:  0.5, y:  0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 1.0 })
        ];

        return Self::piece(vertices.try_into().unwrap());
    }

    pub fn black_piece(piece: i8) -> Model2D {
        let vertices: [(Vec2, Vec2); 6] = [
            (Vec2{ x:  0.5, y:  0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 0.5 }),
            (Vec2{ x:  0.5, y: -0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 0.0 }),
            (Vec2{ x: -0.5, y: -0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 0.0 }),

            (Vec2{ x: -0.5, y: -0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 0.0 }),
            (Vec2{ x: -0.5, y:  0.5 }, Vec2{ x: (5.0 / 6.0) - (1.0 / 6.0) * piece as f32,   y: 0.5 }),
            (Vec2{ x:  0.5, y:  0.5 }, Vec2{ x: 1.0 - (1.0 / 6.0) * piece as f32,           y: 0.5 })
        ];

        return Self::piece(vertices.try_into().unwrap());
    }

    pub fn dummy() -> Model2D {
        return Model2D {
            vao: 0,
            vao_len: 0,
            vbo: 0,
            color: vec4(0.0,0.0,0.0,0.0),
            transform: Transform::new(),
            default_color: vec4(0.0,0.0,0.0,0.0)
        };
    }
}

impl Drop for Model2D {
    fn drop(&mut self) {
        unsafe {
        gl::DeleteBuffers(1, &self.vbo);
        gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}