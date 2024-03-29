use crate::GlxContext;
use crate::Window;
use crate::X11Display;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

fn choose_matching_fbconfigs(
    display: *mut x11::xlib::Display,
    screen_num: i32,
) -> *mut x11::glx::GLXFBConfig {
    let mut fb_configs_cnt = 0;
    let atts = [0];
    unsafe { x11::glx::glXChooseFBConfig(display, screen_num, atts.as_ptr(), &mut fb_configs_cnt) }
}

pub struct VSyncData {
    glx_context: GlxContext,
    win: u32,
}

impl VSyncData {
    pub fn new(
        display: &mut X11Display,
        conn: &XCBConnection,
        screen: &xproto::Screen,
        screen_num: i32,
        root_win: u32,
        ref_win: &Window,
    ) -> Self {
        let win = conn.generate_id().unwrap();
        conn.create_window(
            ref_win.depth(),
            win,
            root_win,
            0,
            0,
            1,
            1,
            0,
            xproto::WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &xproto::CreateWindowAux::default(),
        )
        .unwrap();
        let fb_configs = choose_matching_fbconfigs(display.as_mut(), screen_num);
        let mut glx_context = display.create_glx_context(unsafe { *fb_configs });
        unsafe { x11::xlib::XFree(fb_configs.cast::<core::ffi::c_void>()) };
        display.make_glx_context_current(u64::from(win), &mut glx_context);
        Self { glx_context, win }
    }
    pub fn close(self, display: &mut X11Display, conn: &XCBConnection) {
        display.destroy_glx_context(self.glx_context);
        conn.destroy_window(self.win).unwrap();
    }
    fn swap_buffers(&mut self, display: &mut X11Display) {
        display.swap_buffers(u64::from(self.win));
    }
    pub fn wait(&mut self, display: &mut X11Display) {
        self.swap_buffers(display);
        unsafe { x11::glx::glXWaitGL() };
    }
}
