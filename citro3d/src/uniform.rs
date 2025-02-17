//! Common definitions for binding uniforms to shaders. This is primarily
//! done by implementing the [`Uniform`] trait for a given type.

use std::ops::Range;

use crate::math::{FVec4, IVec, Matrix4};
use crate::{shader, Instance};

/// The index of a uniform within a [`shader::Program`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u8);

impl From<u8> for Index {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Index> for i32 {
    fn from(value: Index) -> Self {
        value.0.into()
    }
}

/// A uniform which may be bound as input to a shader program
pub enum Uniform {
    /// Single float uniform (`.fvec name`)
    Float(FVec4),
    /// Two element float uniform (`.fvec name[2]`)
    Float2([FVec4; 2]),
    /// Three element float uniform (`.fvec name [3]`)
    Float3([FVec4; 3]),
    /// Matrix/4 element float uniform (`.fvec name[4]`)
    Float4(Matrix4),
    /// Bool uniform (`.bool name`)
    Bool(bool),
    /// Integer uniform (`.ivec name`)
    Int(IVec),
}
impl Uniform {
    /// Get range of valid indexes for this uniform to bind to
    pub fn index_range(&self) -> Range<Index> {
        // these indexes are from the uniform table in the shader see: https://www.3dbrew.org/wiki/SHBIN#Uniform_Table_Entry
        // the input registers then are excluded by libctru, see: https://github.com/devkitPro/libctru/blob/0da8705527f03b4b08ff7fee4dd1b7f28df37905/libctru/source/gpu/shbin.c#L93
        match self {
            Uniform::Float(_) | Uniform::Float2(_) | Uniform::Float3(_) | Uniform::Float4(_) => {
                Index(0)..Index(0x60)
            }
            Uniform::Int(_) => Index(0x60)..Index(0x64),
            // this gap is intentional
            Uniform::Bool(_) => Index(0x68)..Index(0x79),
        }
    }
    /// Get length of uniform, i.e. how many registers it will write to
    #[allow(clippy::len_without_is_empty)] // is_empty doesn't make sense here
    pub fn len(&self) -> usize {
        match self {
            Uniform::Float(_) => 1,
            Uniform::Float2(_) => 2,
            Uniform::Float3(_) => 3,
            Uniform::Float4(_) => 4,
            Uniform::Bool(_) | Uniform::Int(_) => 1,
        }
    }

    /// Bind a uniform
    ///
    /// Note: `_instance` is here to ensure unique access to the global uniform buffers
    /// otherwise we could race and/or violate aliasing
    pub(crate) fn bind(self, _instance: &mut Instance, ty: shader::Type, index: Index) {
        assert!(
            self.index_range().contains(&index),
            "tried to bind uniform to an invalid index (index: {}, valid range: {}..{})",
            index.0,
            self.index_range().start.0,
            self.index_range().end.0
        );
        assert!(self.index_range().end.0 as usize >= self.len() + index.0 as usize, "tried to bind a uniform that would overflow the uniform buffer. index was {}, size was {} max is {}", index.0, self.len(), self.index_range().end.0);
        let set_fvs = |fs: &[FVec4]| {
            for (off, f) in fs.iter().enumerate() {
                unsafe {
                    citro3d_sys::C3D_FVUnifSet(
                        ty.into(),
                        (index.0 as usize + off) as i32,
                        f.x(),
                        f.y(),
                        f.z(),
                        f.w(),
                    );
                }
            }
        };
        match self {
            Uniform::Bool(b) => unsafe {
                citro3d_sys::C3D_BoolUnifSet(ty.into(), index.into(), b);
            },
            Uniform::Int(i) => unsafe {
                citro3d_sys::C3D_IVUnifSet(
                    ty.into(),
                    index.into(),
                    i.x() as i32,
                    i.y() as i32,
                    i.z() as i32,
                    i.w() as i32,
                );
            },
            Uniform::Float(f) => set_fvs(&[f]),
            Uniform::Float2(fs) => {
                set_fvs(&fs);
            }
            Uniform::Float3(fs) => set_fvs(&fs),
            Uniform::Float4(m) => {
                set_fvs(&m.rows_wzyx());
            }
        }
    }
}

impl From<Matrix4> for Uniform {
    fn from(value: Matrix4) -> Self {
        Self::Float4(value)
    }
}
impl From<[FVec4; 3]> for Uniform {
    fn from(value: [FVec4; 3]) -> Self {
        Self::Float3(value)
    }
}

impl From<[FVec4; 2]> for Uniform {
    fn from(value: [FVec4; 2]) -> Self {
        Self::Float2(value)
    }
}
impl From<FVec4> for Uniform {
    fn from(value: FVec4) -> Self {
        Self::Float(value)
    }
}
impl From<IVec> for Uniform {
    fn from(value: IVec) -> Self {
        Self::Int(value)
    }
}
impl From<bool> for Uniform {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<&Matrix4> for Uniform {
    fn from(value: &Matrix4) -> Self {
        (*value).into()
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec4> for Uniform {
    fn from(value: glam::Vec4) -> Self {
        Self::Float(value.into())
    }
}

#[cfg(feature = "glam")]
impl From<glam::Mat4> for Uniform {
    fn from(value: glam::Mat4) -> Self {
        Self::Float4(value.into())
    }
}
