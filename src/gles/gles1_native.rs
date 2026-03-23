unsafe fn CopyTexImage2D(
        &mut self,
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
    ) {
        gles11::CopyTexImage2D(target, level, internalformat, x, y, width, height, border)
    }
    unsafe fn CopyTexSubImage2D(
        &mut self,
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    ) {
        gles11::CopyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height)
    }
    unsafe fn TexEnvf(&mut self, target: GLenum, pname: GLenum, param: GLfloat) {
        gles11::TexEnvf(target, pname, param)
    }
    unsafe fn TexEnvx(&mut self, target: GLenum, pname: GLenum, param: GLfixed) {
        gles11::TexEnvx(target, pname, param)
    }
    unsafe fn TexEnvi(&mut self, target: GLenum, pname: GLenum, param: GLint) {
        gles11::TexEnvi(target, pname, param)
    }
    unsafe fn TexEnvfv(&mut self, target: GLenum, pname: GLenum, params: *const GLfloat) {
        if target == gles11::TEXTURE_FILTER_CONTROL_EXT {
            assert!(pname == gles11::TEXTURE_LOD_BIAS_EXT);
            unsafe {
                if !CStr::from_ptr(gles11::GetString(gles11::EXTENSIONS) as _)
                    .to_str()
                    .unwrap()
                    .contains("EXT_texture_lod_bias")
                {
                    log_dbg!(
                        "GL_EXT_texture_lod_bias is unsupported, skipping TexEnvfv({:#x}, {:#x}, ...) call",
                        target,
                        pname
                    );
                    return;
                }
            };
        }
        gles11::TexEnvfv(target, pname, params)
    }
    unsafe fn TexEnvxv(&mut self, target: GLenum, pname: GLenum, params: *const GLfixed) {
        gles11::TexEnvxv(target, pname, params)
    }
    unsafe fn TexEnviv(&mut self, target: GLenum, pname: GLenum, params: *const GLint) {
        gles11::TexEnviv(target, pname, params)
    }

    unsafe fn MultiTexCoord4f(
        &mut self,
        target: GLenum,
        s: GLfloat,
        t: GLfloat,
        r: GLfloat,
        q: GLfloat,
    ) {
        gles11::MultiTexCoord4f(target, s, t, r, q)
    }
    unsafe fn MultiTexCoord4x(
        &mut self,
        target: GLenum,
        s: GLfixed,
        t: GLfixed,
        r: GLfixed,
        q: GLfixed,
    ) {
        gles11::MultiTexCoord4x(target, s, t, r, q)
    }

    // Matrix stack operations
    unsafe fn MatrixMode(&mut self, mode: GLenum) {
        gles11::MatrixMode(mode)
    }
    unsafe fn LoadIdentity(&mut self) {
        gles11::LoadIdentity()
    }
    unsafe fn LoadMatrixf(&mut self, m: *const GLfloat) {
        gles11::LoadMatrixf(m)
    }
    unsafe fn LoadMatrixx(&mut self, m: *const GLfixed) {
        gles11::LoadMatrixx(m)
    }
    unsafe fn MultMatrixf(&mut self, m: *const GLfloat) {
        gles11::MultMatrixf(m)
    }
    unsafe fn MultMatrixx(&mut self, m: *const GLfixed) {
        gles11::MultMatrixx(m)
    }
    unsafe fn PushMatrix(&mut self) {
        gles11::PushMatrix()
    }
    unsafe fn PopMatrix(&mut self) {
        gles11::PopMatrix();
    }
    unsafe fn Orthof(
        &mut self,
        left: GLfloat,
        right: GLfloat,
        bottom: GLfloat,
        top: GLfloat,
        near: GLfloat,
        far: GLfloat,
    ) {
        gles11::Orthof(left, right, bottom, top, near, far)
    }
    unsafe fn Orthox(
        &mut self,
        left: GLfixed,
        right: GLfixed,
        bottom: GLfixed,
        top: GLfixed,
        near: GLfixed,
        far: GLfixed,
    ) {
        gles11::Orthox(left, right, bottom, top, near, far)
    }
    unsafe fn Frustumf(
        &mut self,
        left: GLfloat,
        right: GLfloat,
        bottom: GLfloat,
        top: GLfloat,
        near: GLfloat,
        far: GLfloat,
    ) {
        gles11::Frustumf(left, right, bottom, top, near, far)
    }
    unsafe fn Frustumx(
        &mut self,
        left: GLfixed,
        right: GLfixed,
        bottom: GLfixed,
        top: GLfixed,
        near: GLfixed,
        far: GLfixed,
    ) {
        gles11::Frustumx(left, right, bottom, top, near, far)
    }
    unsafe fn Rotatef(&mut self, angle: GLfloat, x: GLfloat, y: GLfloat, z: GLfloat) {
        gles11::Rotatef(angle, x, y, z)
    }
    unsafe fn Rotatex(&mut self, angle: GLfixed, x: GLfixed, y: GLfixed, z: GLfixed) {
        gles11::Rotatex(angle, x, y, z)
    }
    unsafe fn Scalef(&mut self, x: GLfloat, y: GLfloat, z: GLfloat) {
        gles11::Scalef(x, y, z)
    }
    unsafe fn Scalex(&mut self, x: GLfixed, y: GLfixed, z: GLfixed) {
        gles11::Scalex(x, y, z)
    }
    unsafe fn Translatef(&mut self, x: GLfloat, y: GLfloat, z: GLfloat) {
        gles11::Translatef(x, y, z)
    }
    unsafe fn Translatex(&mut self, x: GLfixed, y: GLfixed, z: GLfixed) {
        gles11::Translatex(x, y, z)
    }

    // OES_framebuffer_object -> EXT_framebuffer_object
    unsafe fn GenFramebuffersOES(&mut self, n: GLsizei, framebuffers: *mut GLuint) {
        gles11::GenFramebuffersOES(n, framebuffers)
    }
    unsafe fn GenRenderbuffersOES(&mut self, n: GLsizei, renderbuffers: *mut GLuint) {
        gles11::GenRenderbuffersOES(n, renderbuffers)
    }
    unsafe fn IsFramebufferOES(&mut self, renderbuffer: GLuint) -> GLboolean {
        gles11::IsFramebufferOES(renderbuffer)
    }
    unsafe fn IsRenderbufferOES(&mut self, renderbuffer: GLuint) -> GLboolean {
        gles11::IsRenderbufferOES(renderbuffer)
    }
    unsafe fn BindFramebufferOES(&mut self, target: GLenum, framebuffer: GLuint) {
        gles11::BindFramebufferOES(target, framebuffer)
    }
    unsafe fn BindRenderbufferOES(&mut self, target: GLenum, renderbuffer: GLuint) {
        gles11::BindRenderbufferOES(target, renderbuffer)
    }
    unsafe fn RenderbufferStorageOES(
        &mut self,
        target: GLenum,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    ) {
        gles11::RenderbufferStorageOES(target, internalformat, width, height)
    }
    unsafe fn FramebufferRenderbufferOES(
        &mut self,
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint,
    ) {
        gles11::FramebufferRenderbufferOES(target, attachment, renderbuffertarget, renderbuffer)
    }
    unsafe fn FramebufferTexture2DOES(
        &mut self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: i32,
    ) {
        gles11::FramebufferTexture2DOES(target, attachment, textarget, texture, level)
    }
    unsafe fn GetFramebufferAttachmentParameterivOES(
        &mut self,
        target: GLenum,
        attachment: GLenum,
        pname: GLenum,
        params: *mut GLint,
    ) {
        gles11::GetFramebufferAttachmentParameterivOES(target, attachment, pname, params)
    }
    unsafe fn GetRenderbufferParameterivOES(
        &mut self,
        target: GLenum,
        pname: GLenum,
        params: *mut GLint,
    ) {
        gles11::GetRenderbufferParameterivOES(target, pname, params)
    }
    unsafe fn CheckFramebufferStatusOES(&mut self, target: GLenum) -> GLenum {
        gles11::CheckFramebufferStatusOES(target)
    }
    unsafe fn DeleteFramebuffersOES(&mut self, n: GLsizei, framebuffers: *const GLuint) {
        gles11::DeleteFramebuffersOES(n, framebuffers)
    }
    unsafe fn DeleteRenderbuffersOES(&mut self, n: GLsizei, renderbuffers: *const GLuint) {
        gles11::DeleteRenderbuffersOES(n, renderbuffers)
    }
    unsafe fn GenerateMipmapOES(&mut self, target: GLenum) {
        gles11::GenerateMipmapOES(target)
    }
    unsafe fn GetBufferParameteriv(&mut self, target: GLenum, pname: GLenum, params: *mut GLint) {
        gles11::GetBufferParameteriv(target, pname, params)
    }
    unsafe fn MapBufferOES(&mut self, target: GLenum, access: GLenum) -> *mut GLvoid {
        gles11::MapBufferOES(target, access)
    }
    unsafe fn UnmapBufferOES(&mut self, target: GLenum) -> GLboolean {
        gles11::UnmapBufferOES(target)
    }
