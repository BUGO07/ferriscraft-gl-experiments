use gl::types::*;
use glam::*;
use std::{
    ffi::CString,
    ptr::{null, null_mut},
};

#[derive(Default)]
pub struct MaterialOptions<'a> {
    pub base_texture: Option<&'a str>,
    pub base_color: Option<Vec4>,
}

pub enum UniformValue {
    Bool(bool),
    Int(GLint),
    Float(GLfloat),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
}

pub struct Material {
    pub program: GLuint,
    pub texture: Option<GLuint>,
    pub base_color: Vec4,
}

impl Material {
    pub fn new(shader: &str, options: MaterialOptions) -> Result<Self, String> {
        let program = Self::load_program(shader)?;
        let texture = if let Some(path) = options.base_texture {
            Some(Self::load_texture(path)?)
        } else {
            None
        };

        Ok(Material {
            program,
            texture,
            base_color: options.base_color.unwrap_or(Vec4::ONE),
        })
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);

            if let Some(texture) = self.texture {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                self.set_uniform("tex", UniformValue::Int(0));
            }

            self.set_uniform("base_color", UniformValue::Vec4(self.base_color));
        }
    }

    fn load_program(shader: &str) -> Result<GLuint, String> {
        let vert_src = std::fs::read_to_string(format!("assets/shaders/{shader}.vert"))
            .map_err(|_| format!("could not read {shader} vertex shader"))?;
        let frag_src = std::fs::read_to_string(format!("assets/shaders/{shader}.frag"))
            .map_err(|_| format!("could not read {shader} fragment shader"))?;

        unsafe {
            let vertex_shader = compile_shader(&vert_src, gl::VERTEX_SHADER)?;
            let fragment_shader = compile_shader(&frag_src, gl::FRAGMENT_SHADER)?;

            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_log = vec![0; 512];
                gl::GetProgramInfoLog(program, 512, null_mut(), info_log.as_mut_ptr() as *mut _);
                return Err(String::from_utf8_lossy(&info_log).into_owned());
            }

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(program)
        }
    }

    fn load_texture(path: &str) -> Result<GLuint, String> {
        let img = image::open(path)
            .map_err(|_| format!("could not load texture: {path}"))?
            .flipv()
            .into_rgba8();
        let (width, height) = img.dimensions();
        let data = img.into_raw();

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width as GLint,
                height as GLint,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        }

        Ok(texture)
    }

    pub fn set_uniform(&self, name: &str, value: UniformValue) {
        unsafe {
            let location =
                gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr());

            match value {
                UniformValue::Bool(b) => gl::Uniform1i(location, b as GLint),
                UniformValue::Int(i) => gl::Uniform1i(location, i),
                UniformValue::Float(f) => gl::Uniform1f(location, f),
                UniformValue::Vec2(v) => gl::Uniform2f(location, v.x, v.y),
                UniformValue::Vec3(v) => gl::Uniform3f(location, v.x, v.y, v.z),
                UniformValue::Vec4(v) => gl::Uniform4f(location, v.x, v.y, v.z, v.w),
                UniformValue::Mat4(m) => gl::UniformMatrix4fv(
                    location,
                    1,
                    gl::FALSE,
                    m.to_cols_array().as_ptr() as *const _,
                ),
            }
        }
    }
}

fn compile_shader(source: &str, shader_type: GLuint) -> Result<GLuint, String> {
    unsafe {
        let c_str = CString::new(source).unwrap();
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = vec![0; 512];
            gl::GetShaderInfoLog(shader, 512, null_mut(), info_log.as_mut_ptr() as *mut _);
            return Err(String::from_utf8_lossy(&info_log).into_owned());
        }
        Ok(shader)
    }
}
