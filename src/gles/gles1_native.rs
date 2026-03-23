use super::gles11_raw as gles11;
use super::gles11_raw::types::*;
use super::gles_generic::GLES;
use super::util::{try_decode_pvrtc, PalettedTextureFormat};
use super::GLESContext;
use crate::window::{GLContext, GLVersion, Window};
use std::ffi::CStr;
use std::marker::PhantomData;

pub struct GLES1NativeContext {
    gl_ctx: GLContext,
    is_loaded: bool,
    is_gles2: bool,
}

impl GLESContext for GLES1NativeContext {
    fn description() -> &'static str {
        "Native OpenGL ES 1.1"
    }

    fn new(window: &mut Window, options: &crate::options::Options) -> Result<Self, String> {
        let is_gles2 = options.gles_version == 2;
        let version = if is_gles2 {
            GLVersion::GLES20
        } else {
            GLVersion::GLES11
        };

        Ok(Self {
            gl_ctx: window.create_gl_context(version)?,
            is_loaded: false,
            is_gles2,
        })
    }

    fn make_current<'gl_ctx, 'win: 'gl_ctx>(
        &'gl_ctx mut self,
        window: &'win mut Window,
    ) -> Box<dyn GLES + 'gl_ctx> {
        if self.gl_ctx.is_current() && self.is_loaded {
            return Box::new(GLES1Native {
                _gl_lifetime: PhantomData,
                is_gles2: self.is_gles2,
            });
        }

        unsafe {
            window.make_gl_context_current(&self.gl_ctx);
        }
        gles11::load_with(|s| window.gl_get_proc_address(s));
        touchHLE_gl_bindings::gles20::load_with(|s| window.gl_get_proc_address(s));
        self.is_loaded = true;

        Box::new(GLES1Native {
            _gl_lifetime: PhantomData,
            is_gles2: self.is_gles2,
        })
    }

    unsafe fn make_current_unchecked_for_window<'gl_ctx>(
        &'gl_ctx mut self,
        make_current_fn: &mut dyn FnMut(&GLContext),
        loader_fn: &mut dyn FnMut(&'static str) -> *const std::ffi::c_void,
    ) -> Box<dyn GLES + 'gl_ctx> {
        if self.gl_ctx.is_current() && self.is_loaded {
            return Box::new(GLES1Native {
                _gl_lifetime: PhantomData,
                is_gles2: self.is_gles2,
            });
        }

        make_current_fn(&self.gl_ctx);
        gles11::load_with(&mut *loader_fn);
        touchHLE_gl_bindings::gles20::load_with(loader_fn);
        self.is_loaded = true;

        Box::new(GLES1Native {
            _gl_lifetime: PhantomData,
            is_gles2: self.is_gles2,
        })
    }
}

pub struct GLES1Native<'gl_ctx> {
    _gl_lifetime: PhantomData<&'gl_ctx ()>,
    is_gles2: bool,
}

impl GLES for GLES1Native<'_> {
    fn is_gles2(&self) -> bool {
        self.is_gles2
    }

    unsafe fn driver_description(&self) -> String {
        let version = CStr::from_ptr(gles11::GetString(gles11::VERSION) as *const _);
        let vendor = CStr::from_ptr(gles11::GetString(gles11::VENDOR) as *const _);
        let renderer = CStr::from_ptr(gles11::GetString(gles11::RENDERER) as *const _);
        format!(
            "{} / {} / {}",
            version.to_string_lossy(),
            vendor.to_string_lossy(),
            renderer.to_string_lossy()
        )
    }

    unsafe fn GetError(&mut self) -> GLenum {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GetError()
        } else {
            gles11::GetError()
        }
    }

    unsafe fn Enable(&mut self, cap: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Enable(cap)
        } else {
            gles11::Enable(cap)
        }
    }

    unsafe fn IsEnabled(&mut self, cap: GLenum) -> GLboolean {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::IsEnabled(cap)
        } else {
            gles11::IsEnabled(cap)
        }
    }

    unsafe fn Disable(&mut self, cap: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Disable(cap)
        } else {
            gles11::Disable(cap)
        }
    }

    unsafe fn ClientActiveTexture(&mut self, texture: GLenum) {
        gles11::ClientActiveTexture(texture);
    }

    unsafe fn EnableClientState(&mut self, array: GLenum) {
        if self.is_gles2 {
            match array {
                gles11::VERTEX_ARRAY => touchHLE_gl_bindings::gles20::EnableVertexAttribArray(0),
                gles11::NORMAL_ARRAY => touchHLE_gl_bindings::gles20::EnableVertexAttribArray(1),
                gles11::COLOR_ARRAY => touchHLE_gl_bindings::gles20::EnableVertexAttribArray(2),
                gles11::TEXTURE_COORD_ARRAY => {
                    let mut ct = 0;
                    gles11::GetIntegerv(gles11::CLIENT_ACTIVE_TEXTURE, &mut ct);
                    let attr = if ct == gles11::TEXTURE1 as GLint { 4 } else { 3 };
                    touchHLE_gl_bindings::gles20::EnableVertexAttribArray(attr);
                }
                _ => {}
            }
        } else {
            gles11::EnableClientState(array)
        }
    }

    unsafe fn DisableClientState(&mut self, array: GLenum) {
        if self.is_gles2 {
            match array {
                gles11::VERTEX_ARRAY => touchHLE_gl_bindings::gles20::DisableVertexAttribArray(0),
                gles11::NORMAL_ARRAY => touchHLE_gl_bindings::gles20::DisableVertexAttribArray(1),
                gles11::COLOR_ARRAY => touchHLE_gl_bindings::gles20::DisableVertexAttribArray(2),
                gles11::TEXTURE_COORD_ARRAY => {
                    let mut ct = 0;
                    gles11::GetIntegerv(gles11::CLIENT_ACTIVE_TEXTURE, &mut ct);
                    let attr = if ct == gles11::TEXTURE1 as GLint { 4 } else { 3 };
                    touchHLE_gl_bindings::gles20::DisableVertexAttribArray(attr);
                }
                _ => {}
            }
        } else {
            gles11::DisableClientState(array)
        }
    }

    unsafe fn GetBooleanv(&mut self, pname: GLenum, params: *mut GLboolean) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GetBooleanv(pname, params)
        } else {
            gles11::GetBooleanv(pname, params)
        }
    }

    unsafe fn GetFloatv(&mut self, pname: GLenum, params: *mut GLfloat) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GetFloatv(pname, params)
        } else {
            gles11::GetFloatv(pname, params)
        }
    }

    unsafe fn GetIntegerv(&mut self, pname: GLenum, params: *mut GLint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GetIntegerv(pname, params)
        } else {
            gles11::GetIntegerv(pname, params)
        }
    }

    unsafe fn GetTexEnviv(&mut self, target: GLenum, pname: GLenum, params: *mut GLint) {
        gles11::GetTexEnviv(target, pname, params)
    }

    unsafe fn GetTexEnvfv(&mut self, target: GLenum, pname: GLenum, params: *mut GLfloat) {
        gles11::GetTexEnvfv(target, pname, params)
    }

    unsafe fn GetPointerv(&mut self, pname: GLenum, params: *mut *const GLvoid) {
        gles11::GetPointerv(pname, params as *mut _ as *const _)
    }

    unsafe fn Hint(&mut self, target: GLenum, mode: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Hint(target, mode)
        } else {
            gles11::Hint(target, mode)
        }
    }

    unsafe fn Finish(&mut self) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Finish()
        } else {
            gles11::Finish()
        }
    }

    unsafe fn Flush(&mut self) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Flush()
        } else {
            gles11::Flush()
        }
    }

    unsafe fn GetString(&mut self, name: GLenum) -> *const GLubyte {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GetString(name)
        } else {
            gles11::GetString(name)
        }
    }

    unsafe fn AlphaFunc(&mut self, func: GLenum, ref_: GLclampf) {
        gles11::AlphaFunc(func, ref_)
    }

    unsafe fn AlphaFuncx(&mut self, func: GLenum, ref_: GLclampx) {
        gles11::AlphaFuncx(func, ref_)
    }

    unsafe fn BlendFunc(&mut self, sfactor: GLenum, dfactor: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BlendFunc(sfactor, dfactor)
        } else {
            gles11::BlendFunc(sfactor, dfactor)
        }
    }

    unsafe fn BlendEquationOES(&mut self, mode: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BlendEquation(mode)
        } else {
            gles11::BlendEquationOES(mode)
        }
    }

    unsafe fn ColorMask(
        &mut self,
        red: GLboolean,
        green: GLboolean,
        blue: GLboolean,
        alpha: GLboolean,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::ColorMask(red, green, blue, alpha)
        } else {
            gles11::ColorMask(red, green, blue, alpha)
        }
    }

    unsafe fn ClipPlanef(&mut self, plane: GLenum, equation: *const GLfloat) {
        gles11::ClipPlanef(plane, equation)
    }

    unsafe fn ClipPlanex(&mut self, plane: GLenum, equation: *const GLfixed) {
        gles11::ClipPlanex(plane, equation)
    }

    unsafe fn CullFace(&mut self, mode: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::CullFace(mode)
        } else {
            gles11::CullFace(mode)
        }
    }

    unsafe fn DepthFunc(&mut self, func: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DepthFunc(func)
        } else {
            gles11::DepthFunc(func)
        }
    }

    unsafe fn DepthMask(&mut self, flag: GLboolean) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DepthMask(flag)
        } else {
            gles11::DepthMask(flag)
        }
    }

    unsafe fn FrontFace(&mut self, mode: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::FrontFace(mode)
        } else {
            gles11::FrontFace(mode)
        }
    }

    unsafe fn DepthRangef(&mut self, near: GLclampf, far: GLclampf) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DepthRangef(near, far)
        } else {
            gles11::DepthRangef(near, far)
        }
    }

    unsafe fn DepthRangex(&mut self, near: GLclampx, far: GLclampx) {
        gles11::DepthRangex(near, far)
    }

    unsafe fn PolygonOffset(&mut self, factor: GLfloat, units: GLfloat) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::PolygonOffset(factor, units)
        } else {
            gles11::PolygonOffset(factor, units)
        }
    }

    unsafe fn PolygonOffsetx(&mut self, factor: GLfixed, units: GLfixed) {
        gles11::PolygonOffsetx(factor, units)
    }

    unsafe fn SampleCoverage(&mut self, value: GLclampf, invert: GLboolean) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::SampleCoverage(value, invert)
        } else {
            gles11::SampleCoverage(value, invert)
        }
    }

    unsafe fn SampleCoveragex(&mut self, value: GLclampx, invert: GLboolean) {
        gles11::SampleCoveragex(value, invert)
    }

    unsafe fn ShadeModel(&mut self, mode: GLenum) {
        gles11::ShadeModel(mode)
    }

    unsafe fn Scissor(&mut self, x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Scissor(x, y, width, height)
        } else {
            gles11::Scissor(x, y, width, height)
        }
    }

    unsafe fn Viewport(&mut self, x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Viewport(x, y, width, height)
        } else {
            gles11::Viewport(x, y, width, height)
        }
    }

    unsafe fn LineWidth(&mut self, val: GLfloat) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::LineWidth(val)
        } else {
            gles11::LineWidth(val)
        }
    }

    unsafe fn LineWidthx(&mut self, val: GLfixed) {
        gles11::LineWidthx(val)
    }

    unsafe fn StencilFunc(&mut self, func: GLenum, ref_: GLint, mask: GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::StencilFunc(func, ref_, mask)
        } else {
            gles11::StencilFunc(func, ref_, mask)
        }
    }

    unsafe fn StencilOp(&mut self, sfail: GLenum, dpfail: GLenum, dppass: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::StencilOp(sfail, dpfail, dppass)
        } else {
            gles11::StencilOp(sfail, dpfail, dppass)
        }
    }

    unsafe fn StencilMask(&mut self, mask: GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::StencilMask(mask)
        } else {
            gles11::StencilMask(mask)
        }
    }

    unsafe fn LogicOp(&mut self, opcode: GLenum) {
        gles11::LogicOp(opcode);
    }

    unsafe fn PointSize(&mut self, size: GLfloat) {
        gles11::PointSize(size)
    }

    unsafe fn PointSizex(&mut self, size: GLfixed) {
        gles11::PointSizex(size)
    }

    unsafe fn PointParameterf(&mut self, pname: GLenum, param: GLfloat) {
        gles11::PointParameterf(pname, param)
    }

    unsafe fn PointParameterx(&mut self, pname: GLenum, param: GLfixed) {
        gles11::PointParameterx(pname, param)
    }

    unsafe fn PointParameterfv(&mut self, pname: GLenum, params: *const GLfloat) {
        gles11::PointParameterfv(pname, params)
    }

    unsafe fn PointParameterxv(&mut self, pname: GLenum, params: *const GLfixed) {
        gles11::PointParameterxv(pname, params)
    }

    unsafe fn Fogf(&mut self, pname: GLenum, param: GLfloat) {
        gles11::Fogf(pname, param)
    }

    unsafe fn Fogx(&mut self, pname: GLenum, param: GLfixed) {
        gles11::Fogx(pname, param)
    }

    unsafe fn Fogfv(&mut self, pname: GLenum, params: *const GLfloat) {
        gles11::Fogfv(pname, params)
    }

    unsafe fn Fogxv(&mut self, pname: GLenum, params: *const GLfixed) {
        gles11::Fogxv(pname, params)
    }

    unsafe fn Lightf(&mut self, light: GLenum, pname: GLenum, param: GLfloat) {
        gles11::Lightf(light, pname, param)
    }

    unsafe fn Lightx(&mut self, light: GLenum, pname: GLenum, param: GLfixed) {
        gles11::Lightx(light, pname, param)
    }

    unsafe fn Lightfv(&mut self, light: GLenum, pname: GLenum, params: *const GLfloat) {
        gles11::Lightfv(light, pname, params)
    }

    unsafe fn Lightxv(&mut self, light: GLenum, pname: GLenum, params: *const GLfixed) {
        gles11::Lightxv(light, pname, params)
    }

    unsafe fn LightModelf(&mut self, pname: GLenum, param: GLfloat) {
        gles11::LightModelf(pname, param)
    }

    unsafe fn LightModelx(&mut self, pname: GLenum, param: GLfixed) {
        gles11::LightModelx(pname, param)
    }

    unsafe fn LightModelfv(&mut self, pname: GLenum, params: *const GLfloat) {
        gles11::LightModelfv(pname, params)
    }

    unsafe fn LightModelxv(&mut self, pname: GLenum, params: *const GLfixed) {
        gles11::LightModelxv(pname, params)
    }

    unsafe fn Materialf(&mut self, face: GLenum, pname: GLenum, param: GLfloat) {
        gles11::Materialf(face, pname, param)
    }

    unsafe fn Materialx(&mut self, face: GLenum, pname: GLenum, param: GLfixed) {
        gles11::Materialx(face, pname, param)
    }

    unsafe fn Materialfv(&mut self, face: GLenum, pname: GLenum, params: *const GLfloat) {
        gles11::Materialfv(face, pname, params)
    }

    unsafe fn Materialxv(&mut self, face: GLenum, pname: GLenum, params: *const GLfixed) {
        gles11::Materialxv(face, pname, params)
    }

    unsafe fn IsBuffer(&mut self, buffer: GLuint) -> GLboolean {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::IsBuffer(buffer)
        } else {
            gles11::IsBuffer(buffer)
        }
    }

    unsafe fn GenBuffers(&mut self, n: GLsizei, buffers: *mut GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GenBuffers(n, buffers)
        } else {
            gles11::GenBuffers(n, buffers)
        }
    }

    unsafe fn DeleteBuffers(&mut self, n: GLsizei, buffers: *const GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DeleteBuffers(n, buffers)
        } else {
            gles11::DeleteBuffers(n, buffers)
        }
    }

    unsafe fn BindBuffer(&mut self, target: GLenum, buffer: GLuint) {
        assert!(target == gles11::ARRAY_BUFFER || target == gles11::ELEMENT_ARRAY_BUFFER);
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BindBuffer(target, buffer)
        } else {
            gles11::BindBuffer(target, buffer)
        }
    }

    unsafe fn BufferData(
        &mut self,
        target: GLenum,
        size: GLsizeiptr,
        data: *const GLvoid,
        usage: GLenum,
    ) {
        assert!(target == gles11::ARRAY_BUFFER || target == gles11::ELEMENT_ARRAY_BUFFER);
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BufferData(target, size, data, usage)
        } else {
            gles11::BufferData(target, size, data, usage)
        }
    }

    unsafe fn BufferSubData(
        &mut self,
        target: GLenum,
        offset: GLintptr,
        size: GLsizeiptr,
        data: *const GLvoid,
    ) {
        assert!(target == gles11::ARRAY_BUFFER || target == gles11::ELEMENT_ARRAY_BUFFER);
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BufferSubData(target, offset, size, data)
        } else {
            gles11::BufferSubData(target, offset, size, data)
        }
    }

    unsafe fn Color4f(&mut self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        gles11::Color4f(red, green, blue, alpha)
    }

    unsafe fn Color4x(&mut self, red: GLfixed, green: GLfixed, blue: GLfixed, alpha: GLfixed) {
        gles11::Color4x(red, green, blue, alpha)
    }

    unsafe fn Color4ub(&mut self, red: GLubyte, green: GLubyte, blue: GLubyte, alpha: GLubyte) {
        gles11::Color4ub(red, green, blue, alpha)
    }

    unsafe fn Normal3f(&mut self, nx: GLfloat, ny: GLfloat, nz: GLfloat) {
        gles11::Normal3f(nx, ny, nz)
    }

    unsafe fn Normal3x(&mut self, nx: GLfixed, ny: GLfixed, nz: GLfixed) {
        gles11::Normal3x(nx, ny, nz)
    }

    unsafe fn ColorPointer(
        &mut self,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        pointer: *const GLvoid,
    ) {
        if self.is_gles2 {
            let n = if type_ == 0x1401 { 1 } else { 0 };
            touchHLE_gl_bindings::gles20::VertexAttribPointer(
                2, size, type_, n as _, stride, pointer,
            );
        } else {
            gles11::ColorPointer(size, type_, stride, pointer)
        }
    }

    unsafe fn NormalPointer(&mut self, type_: GLenum, stride: GLsizei, pointer: *const GLvoid) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::VertexAttribPointer(1, 3, type_, 0, stride, pointer);
        } else {
            gles11::NormalPointer(type_, stride, pointer)
        }
    }

    unsafe fn TexCoordPointer(
        &mut self,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        pointer: *const GLvoid,
    ) {
        if self.is_gles2 {
            let mut ct = 0;
            gles11::GetIntegerv(gles11::CLIENT_ACTIVE_TEXTURE, &mut ct);
            let attr = if ct == gles11::TEXTURE1 as GLint { 4 } else { 3 };
            touchHLE_gl_bindings::gles20::VertexAttribPointer(attr, size, type_, 0, stride, pointer);
        } else {
            gles11::TexCoordPointer(size, type_, stride, pointer)
        }
    }

    unsafe fn VertexPointer(
        &mut self,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        pointer: *const GLvoid,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::VertexAttribPointer(0, size, type_, 0, stride, pointer);
        } else {
            gles11::VertexPointer(size, type_, stride, pointer)
        }
    }

    unsafe fn DrawArrays(&mut self, mode: GLenum, first: GLint, count: GLsizei) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DrawArrays(mode, first, count)
        } else {
            gles11::DrawArrays(mode, first, count)
        }
    }

    unsafe fn DrawElements(
        &mut self,
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const GLvoid,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DrawElements(mode, count, type_, indices)
        } else {
            gles11::DrawElements(mode, count, type_, indices)
        }
    }

    unsafe fn Clear(&mut self, mask: GLbitfield) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::Clear(mask)
        } else {
            gles11::Clear(mask)
        }
    }

    unsafe fn ClearColor(
        &mut self,
        red: GLclampf,
        green: GLclampf,
        blue: GLclampf,
        alpha: GLclampf,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::ClearColor(red, green, blue, alpha)
        } else {
            gles11::ClearColor(red, green, blue, alpha)
        }
    }

    unsafe fn ClearColorx(
        &mut self,
        red: GLclampx,
        green: GLclampx,
        blue: GLclampx,
        alpha: GLclampx,
    ) {
        gles11::ClearColorx(red, green, blue, alpha)
    }

    unsafe fn ClearDepthf(&mut self, depth: GLclampf) {
        gles11::ClearDepthf(depth)
    }

    unsafe fn ClearDepthx(&mut self, depth: GLclampx) {
        gles11::ClearDepthx(depth)
    }

    unsafe fn ClearStencil(&mut self, s: GLint) {
        gles11::ClearStencil(s)
    }

    unsafe fn PixelStorei(&mut self, pname: GLenum, param: GLint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::PixelStorei(pname, param)
        } else {
            gles11::PixelStorei(pname, param)
        }
    }

    unsafe fn ReadPixels(
        &mut self,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *mut GLvoid,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::ReadPixels(x, y, width, height, format, type_, pixels)
        } else {
            gles11::ReadPixels(x, y, width, height, format, type_, pixels)
        }
    }

    unsafe fn GenTextures(&mut self, n: GLsizei, textures: *mut GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::GenTextures(n, textures)
        } else {
            gles11::GenTextures(n, textures)
        }
    }

    unsafe fn DeleteTextures(&mut self, n: GLsizei, textures: *const GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::DeleteTextures(n, textures)
        } else {
            gles11::DeleteTextures(n, textures)
        }
    }

    unsafe fn ActiveTexture(&mut self, texture: GLenum) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::ActiveTexture(texture)
        } else {
            gles11::ActiveTexture(texture)
        }
    }

    unsafe fn IsTexture(&mut self, texture: GLuint) -> GLboolean {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::IsTexture(texture)
        } else {
            gles11::IsTexture(texture)
        }
    }

    unsafe fn BindTexture(&mut self, target: GLenum, texture: GLuint) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::BindTexture(target, texture)
        } else {
            gles11::BindTexture(target, texture)
        }
    }

    unsafe fn TexParameteri(&mut self, target: GLenum, pname: GLenum, param: GLint) {
        if self.is_gles2 {
            if pname == 0x8191 || pname == 0x813D {
                return;
            }
            let mut p = param;
            if pname == gles11::TEXTURE_MIN_FILTER {
                if p == 0x2700 || p == 0x2701 {
                    p = 0x2600;
                }
                if p == 0x2702 || p == 0x2703 {
                    p = 0x2601;
                }
            }
            touchHLE_gl_bindings::gles20::TexParameteri(target, pname, p)
        } else {
            gles11::TexParameteri(target, pname, param)
        }
    }

    unsafe fn TexParameterf(&mut self, target: GLenum, pname: GLenum, param: GLfloat) {
        if self.is_gles2 {
            if pname == 0x8191 || pname == 0x813D {
                return;
            }
            let mut p = param;
            if pname == gles11::TEXTURE_MIN_FILTER {
                if p == 0x2700 as f32 || p == 0x2701 as f32 {
                    p = 0x2600 as f32;
                }
                if p == 0x2702 as f32 || p == 0x2703 as f32 {
                    p = 0x2601 as f32;
                }
            }
            touchHLE_gl_bindings::gles20::TexParameterf(target, pname, p)
        } else {
            gles11::TexParameterf(target, pname, param)
        }
    }

    unsafe fn TexParameterx(&mut self, target: GLenum, pname: GLenum, param: GLfixed) {
        gles11::TexParameterx(target, pname, param)
    }

    unsafe fn TexParameteriv(&mut self, target: GLenum, pname: GLenum, params: *const GLint) {
        gles11::TexParameteriv(target, pname, params)
    }

    unsafe fn TexParameterfv(&mut self, target: GLenum, pname: GLenum, params: *const GLfloat) {
        gles11::TexParameterfv(target, pname, params)
    }

    unsafe fn TexParameterxv(&mut self, target: GLenum, pname: GLenum, params: *const GLfixed) {
        gles11::TexParameterxv(target, pname, params)
    }

    unsafe fn TexImage2D(
        &mut self,
        target: GLenum,
        level: GLint,
        mut internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid,
    ) {
        if format == gles11::BGRA_EXT {
            internalformat = gles11::BGRA_EXT as GLint;
        }
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::PixelStorei(gles11::UNPACK_ALIGNMENT, 1);
            touchHLE_gl_bindings::gles20::TexImage2D(
                target, level, internalformat, width, height, border, format, type_, pixels,
            );
            let p_target = if (0x8515..=0x851A).contains(&target) {
                0x8513
            } else {
                target
            };
            touchHLE_gl_bindings::gles20::TexParameteri(
                p_target,
                gles11::TEXTURE_MIN_FILTER,
                gles11::LINEAR as _,
            );
        } else {
            gles11::TexImage2D(
                target, level, internalformat, width, height, border, format, type_, pixels,
            )
        }
    }

    unsafe fn TexSubImage2D(
        &mut self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid,
    ) {
        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::PixelStorei(gles11::UNPACK_ALIGNMENT, 1);
            touchHLE_gl_bindings::gles20::TexSubImage2D(
                target, level, xoffset, yoffset, width, height, format, type_, pixels,
            )
        } else {
            gles11::TexSubImage2D(
                target, level, xoffset, yoffset, width, height, format, type_, pixels,
            )
        }
    }

    unsafe fn CompressedTexImage2D(
        &mut self,
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        image_size: GLsizei,
        data: *const GLvoid,
    ) {
        let data_slice = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), image_size as usize) };
        if try_decode_pvrtc(
            self, target, level, internalformat, width, height, border, data_slice,
        ) {
            log::debug!("Decoded PVRTC");
            return;
        }

        let is_paletted = PalettedTextureFormat::get_info(internalformat).is_some();
        if !is_paletted {
            if internalformat == 0x8c92 {
                log::warn!("ATC RGB (0x8c92) detected. Passing to driver...");
            } else {
                log::error!("Unknown CompressedTexImage2D format: {:#x}", internalformat);
            }
        } else {
            log::debug!("Directly supported texture format: {:#x}", internalformat);
        }

        if self.is_gles2 {
            touchHLE_gl_bindings::gles20::PixelStorei(gles11::UNPACK_ALIGNMENT, 1);
            touchHLE_gl_bindings::gles20::CompressedTexImage2D(
                target, level, internalformat, width, height, border, image_size, data,
            );
            let p_target = if (0x8515..=0x851A).contains(&target) {
                0x8513
            } else {
                target
            };
            touchHLE_gl_bindings::gles20::TexParameteri(
                p_target,
                gles11::TEXTURE_MIN_FILTER,
                gles11::LINEAR as _,
            );
        } else {
            gles11::CompressedTexImage2D(
                target, level, internalformat, width, height, border, image_size, data,
            );
        }
    }

    unsafe fn CopyTexImage2D(
        &mut self
