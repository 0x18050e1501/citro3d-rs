//! Safe Rust bindings to `citro3d`.
#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run_gdb)]

pub mod attrib;
pub mod buffer;
pub mod error;
pub mod math;
pub mod render;
pub mod shader;

pub use error::{Error, Result};
pub use math::Matrix;

pub mod macros {
    //! Helper macros for working with shaders.
    pub use citro3d_macros::*;
}

/// The single instance for using `citro3d`. This is the base type that an application
/// should instantiate to use this library.
#[non_exhaustive]
#[must_use]
#[derive(Debug)]
pub struct Instance;

impl Instance {
    /// Initialize the default `citro3d` instance.
    ///
    /// # Errors
    ///
    /// Fails if `citro3d` cannot be initialized.
    pub fn new() -> Result<Self> {
        Self::with_cmdbuf_size(citro3d_sys::C3D_DEFAULT_CMDBUF_SIZE.try_into().unwrap())
    }

    /// Initialize the instance with a specified command buffer size.
    ///
    /// # Errors
    ///
    /// Fails if `citro3d` cannot be initialized.
    pub fn with_cmdbuf_size(size: usize) -> Result<Self> {
        if unsafe { citro3d_sys::C3D_Init(size) } {
            Ok(Self)
        } else {
            Err(Error::FailedToInitialize)
        }
    }

    /// Select the given render target for drawing the frame.
    ///
    /// # Errors
    ///
    /// Fails if the given target cannot be used for drawing.
    pub fn select_render_target(&mut self, target: &render::Target<'_>) -> Result<()> {
        let _ = self;
        if unsafe { citro3d_sys::C3D_FrameDrawOn(target.as_raw()) } {
            Ok(())
        } else {
            Err(Error::InvalidRenderTarget)
        }
    }

    /// Render a frame. The passed in function/closure can mutate the instance,
    /// such as to [select a render target](Self::select_render_target).
    pub fn render_frame_with(&mut self, f: impl FnOnce(&mut Self)) {
        unsafe {
            citro3d_sys::C3D_FrameBegin(
                // TODO: begin + end flags should be configurable
                citro3d_sys::C3D_FRAME_SYNCDRAW
                    .try_into()
                    .expect("const is valid u8"),
            );
        }

        f(self);

        unsafe {
            citro3d_sys::C3D_FrameEnd(0);
        }
    }

    /// Get the buffer info being used, if it exists. Note that the resulting
    /// [`buffer::Info`] is copied from the one currently in use.
    pub fn buffer_info(&self) -> Option<buffer::Info> {
        let raw = unsafe { citro3d_sys::C3D_GetBufInfo() };
        buffer::Info::copy_from(raw)
    }

    /// Set the buffer info to use for any following draw calls.
    pub fn set_buffer_info(&mut self, buffer_info: &buffer::Info) {
        let raw: *const _ = &buffer_info.0;
        // SAFETY: C3D_SetBufInfo actually copies the pointee instead of mutating it.
        unsafe { citro3d_sys::C3D_SetBufInfo(raw.cast_mut()) };
    }

    /// Get the attribute info being used, if it exists. Note that the resulting
    /// [`attrib::Info`] is copied from the one currently in use.
    pub fn attr_info(&self) -> Option<attrib::Info> {
        let raw = unsafe { citro3d_sys::C3D_GetAttrInfo() };
        attrib::Info::copy_from(raw)
    }

    /// Set the attribute info to use for any following draw calls.
    pub fn set_attr_info(&mut self, attr_info: &attrib::Info) {
        let raw: *const _ = &attr_info.0;
        // SAFETY: C3D_SetAttrInfo actually copies the pointee instead of mutating it.
        unsafe { citro3d_sys::C3D_SetAttrInfo(raw.cast_mut()) };
    }

    /// Draw the specified primitivearrays. The
    pub fn draw_arrays(&mut self, primitive: buffer::Primitive, index: buffer::Slice) {
        self.set_buffer_info(index.info());

        // TODO: should we also require the attrib info directly here?

        unsafe {
            citro3d_sys::C3D_DrawArrays(
                primitive as ctru_sys::GPU_Primitive_t,
                index.index(),
                index.len(),
            );
        }
    }

    // TODO: need separate versions for vertex/geometry and different dimensions?
    // Maybe we could do something nicer with const generics, or something, although
    // it will probably be tricker
    pub fn update_vertex_uniform_mat4x4(&mut self, index: i8, matrix: &Matrix) {
        unsafe {
            citro3d_sys::C3D_FVUnifMtx4x4(
                ctru_sys::GPU_VERTEX_SHADER,
                index.into(),
                matrix.as_raw(),
            )
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            citro3d_sys::C3D_Fini();
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum AspectRatio {
    TopScreen,
    BottomScreen,
    Other(f32),
}

impl From<AspectRatio> for f32 {
    fn from(ratio: AspectRatio) -> Self {
        match ratio {
            AspectRatio::TopScreen => citro3d_sys::C3D_AspectRatioTop as f32,
            AspectRatio::BottomScreen => citro3d_sys::C3D_AspectRatioBot as f32,
            AspectRatio::Other(ratio) => ratio,
        }
    }
}
