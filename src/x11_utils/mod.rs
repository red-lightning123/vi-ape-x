mod window;

pub use window::Window;

pub fn choose_matching_fbconfigs(
    display: *mut x11::xlib::Display,
    screen_num: i32,
) -> *mut x11::glx::GLXFBConfig {
    let mut fb_configs_cnt = 0;
    let atts = [
        // A double-buffered config is required because the program relies on
        // glXSwapBuffers for flushing the gl queue. On single-buffered configs,
        // buffer-swapping is a no-op, so flushing would have to be done
        // manually via glFlush in order to support them
        x11::glx::GLX_DOUBLEBUFFER,
        true as i32,
        0,
    ];
    unsafe { x11::glx::glXChooseFBConfig(display, screen_num, atts.as_ptr(), &mut fb_configs_cnt) }
}

pub struct X11Display<'a> {
    display: &'a mut x11::xlib::Display,
}

impl<'a> X11Display<'a> {
    pub fn open() -> Result<X11Display<'a>, ()> {
        let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
        let display = unsafe { display.as_mut() };
        let display = display.ok_or(())?;
        Ok(X11Display { display })
    }
    pub fn close(self) {
        unsafe { x11::xlib::XCloseDisplay(self.display) };
    }
    pub fn to_xcb_connection_mut(
        &mut self,
    ) -> Result<x11rb::xcb_ffi::XCBConnection, x11rb::rust_connection::ConnectError> {
        let conn = unsafe { x11::xlib_xcb::XGetXCBConnection(self.display) };
        let conn = unsafe { x11rb::xcb_ffi::XCBConnection::from_raw_xcb_connection(conn, false)? };
        Ok(conn)
    }
    pub fn default_screen(&mut self) -> i32 {
        unsafe { x11::xlib::XDefaultScreen(self.display) }
    }
    pub fn create_glx_context(&mut self, fb_configs: x11::glx::GLXFBConfig) -> GlxContext {
        GlxContext(unsafe {
            x11::glx::glXCreateNewContext(
                self.display,
                fb_configs,
                x11::glx::GLX_RGBA_TYPE,
                std::ptr::null_mut(),
                i32::from(true),
            )
        })
    }
    pub fn destroy_glx_context(&mut self, glx_context: GlxContext) {
        unsafe { x11::glx::glXDestroyContext(self.display, glx_context.0) };
    }
    pub fn make_glx_context_current(&mut self, win: u64, glx_context: &mut GlxContext) {
        unsafe { x11::glx::glXMakeCurrent(self.display, win, glx_context.0) };
    }
    pub fn swap_buffers(&mut self, win: u64) {
        unsafe { x11::glx::glXSwapBuffers(self.as_mut(), win) };
    }
}

impl AsMut<x11::xlib::Display> for X11Display<'_> {
    fn as_mut(&mut self) -> &mut x11::xlib::Display {
        self.display
    }
}

pub struct GlxContext(x11::glx::GLXContext);
