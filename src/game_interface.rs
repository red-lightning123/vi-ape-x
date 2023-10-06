use crate::keycodes;
use crate::GlxContext;
use crate::ImageRef4;
use crate::X11Display;
use std::cell::OnceCell;
use std::process::Command;
use x11rb::connection::Connection;
use x11rb::connection::RequestConnection;
use x11rb::properties::WmClass;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::xcb_ffi::XCBConnection;

fn choose_matching_fbconfigs(
    display: *mut x11::xlib::Display,
    screen_num: i32,
) -> *mut x11::glx::GLXFBConfig {
    let mut fb_configs_cnt = 0;
    let atts = [0];
    unsafe { x11::glx::glXChooseFBConfig(display, screen_num, atts.as_ptr(), &mut fb_configs_cnt) }
}

fn get_children(
    conn: &XCBConnection,
    win: u32,
) -> Result<Vec<u32>, x11rb::rust_connection::ReplyError> {
    let tree = conn.query_tree(win)?.reply()?;
    Ok(tree.children)
}

enum GetWindowTitleError {
    Reply(x11rb::rust_connection::ReplyError),
    FromUtf8(std::string::FromUtf8Error),
}

fn get_window_title_bytes(
    conn: &XCBConnection,
    win: u32,
) -> Result<Vec<u8>, x11rb::rust_connection::ReplyError> {
    Ok(conn
        .get_property(
            false,
            win,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            0,
            32,
        )?
        .reply()?
        .value)
}

fn get_window_title(conn: &XCBConnection, win: u32) -> Result<String, GetWindowTitleError> {
    let title_bytes = get_window_title_bytes(conn, win).map_err(GetWindowTitleError::Reply)?;
    let title = String::from_utf8(title_bytes).map_err(GetWindowTitleError::FromUtf8)?;
    Ok(title)
}

fn get_window_instance_name(
    conn: &XCBConnection,
    win: u32,
) -> Result<String, x11rb::rust_connection::ReplyError> {
    Ok(
        std::str::from_utf8(WmClass::get(conn, win)?.reply()?.instance())
            .unwrap()
            .to_string(),
    )
}

fn find_descendant_win_with_title(conn: &XCBConnection, parent: u32, name: &str) -> Option<u32> {
    // can fail on non-UTF8 titles (or possibly if there's an x11 connection error), but such a
    // window wouldn't be the one we're looking for anyways so errors are ignored
    if let Ok(instance_name) = get_window_title(conn, parent) {
        if instance_name == name {
            return Some(parent);
        }
    }
    // can fail e. g. if a window was moved/deleted during the call
    // assumes that it's not the right window, errors are ignored
    if let Ok(children) = get_children(conn, parent) {
        for child in children {
            if let Some(win) = find_descendant_win_with_title(conn, child, name) {
                return Some(win);
            }
        }
    }
    None
}

struct XShmSeg {
    address: *mut core::ffi::c_void,
    x_seg: u32,
}

impl XShmSeg {
    fn new(conn: &XCBConnection, len: usize) -> XShmSeg {
        let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, len, libc::IPC_CREAT | 0o777) };
        let address = unsafe { libc::shmat(shmid, std::ptr::null(), 0) };
        let x_seg = conn.generate_id().unwrap();
        conn.shm_attach(x_seg, shmid as u32, false).unwrap();
        XShmSeg { address, x_seg }
    }
    fn address(&self) -> *mut core::ffi::c_void {
        self.address
    }
    fn xid(&self) -> u32 {
        self.x_seg
    }
    fn close(self, conn: &XCBConnection) {
        conn.shm_detach(self.x_seg).unwrap();
        unsafe { libc::shmdt(self.address) };
    }
}

#[derive(Clone)]
pub struct Window {
    handle: u32,
    dims: xproto::GetGeometryReply,
    attribs: xproto::GetWindowAttributesReply,
    image_len: usize,
}

impl Window {
    fn new(
        handle: u32,
        dims: xproto::GetGeometryReply,
        attribs: xproto::GetWindowAttributesReply,
        image_len: usize,
    ) -> Window {
        Window {
            handle,
            dims,
            attribs,
            image_len,
        }
    }
    fn handle(&self) -> u32 {
        self.handle
    }
    pub fn width(&self) -> u16 {
        self.dims.width
    }
    pub fn height(&self) -> u16 {
        self.dims.height
    }
    pub fn depth(&self) -> u8 {
        self.dims.depth
    }
    fn image_len(&self) -> usize {
        self.image_len
    }
    pub fn class(&self) -> xproto::WindowClass {
        self.attribs.class
    }
}

#[derive(Clone, Copy)]
pub enum GameKey {
    S,
    Space,
}

#[derive(Clone, Copy)]
pub enum KeyEventKind {
    Press,
    Release,
}

pub struct VSyncData {
    glx_context: GlxContext,
    win: u32,
}

impl VSyncData {
    fn new(
        display: &mut X11Display,
        conn: &XCBConnection,
        screen: &xproto::Screen,
        screen_num: i32,
        root_win: u32,
        ref_win: &Window,
    ) -> VSyncData {
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
        VSyncData { glx_context, win }
    }
    fn close(self, display: &mut X11Display, conn: &XCBConnection) {
        display.destroy_glx_context(self.glx_context);
        conn.destroy_window(self.win).unwrap();
    }
    fn swap_buffers(&mut self, display: &mut X11Display) {
        display.swap_buffers(u64::from(self.win));
    }
    fn wait(&mut self, display: &mut X11Display) {
        self.swap_buffers(display);
        unsafe { x11::glx::glXWaitGL() };
    }
}

pub struct GameInterface<'a> {
    display: X11Display<'a>,
    conn: XCBConnection,
    root_win: u32,
    screen: xproto::Screen,
    screen_num: i32,
    vsync: OnceCell<VSyncData>,
    process: Option<std::process::Child>,
    win: Option<Window>,
    image_seg: OnceCell<XShmSeg>,
}

impl<'a> GameInterface<'a> {
    pub fn new() -> GameInterface<'a> {
        let mut display = X11Display::open().expect("Can't open a connection to the X server");
        let conn = display
            .to_xcb_connection_mut()
            .expect("Can't convert display to xcb connection");
        let screen_num = display.default_screen();
        let shm_extension_present = conn
            .extension_information(x11rb::protocol::shm::X11_EXTENSION_NAME)
            .expect("Can't query extension information")
            .is_some();
        assert!(shm_extension_present, "This program assumes the presence of xshm extension, yet the extension isn't supported on this machine");
        let screen = &conn.setup().roots[screen_num as usize];
        let root_win = screen.root;
        let screen_temp = screen.clone();
        GameInterface {
            display,
            conn,
            root_win,
            screen: screen_temp,
            screen_num,
            vsync: OnceCell::new(),
            process: None,
            win: None,
            image_seg: OnceCell::new(),
        }
    }
    fn wait_for_win_with_title(&self, title: &str) -> u32 {
        loop {
            if let Some(win_handle) =
                find_descendant_win_with_title(&self.conn, self.root_win, title)
            {
                return win_handle;
            }
        }
    }
    pub fn start(&mut self) {
        const WIN_TITLE: &str = "Pixave's Journey";
        self.process = Some(
            Command::new("python")
                .current_dir("pixaves_journey")
                .args(["main.py"])
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
        let win_handle = self.wait_for_win_with_title(WIN_TITLE);
        let win_dims = self.conn.get_geometry(win_handle).unwrap().reply().unwrap();
        let win_attribs = self
            .conn
            .get_window_attributes(win_handle)
            .unwrap()
            .reply()
            .unwrap();
        let win_image_len =
            (win_dims.depth as usize) * (win_dims.width as usize) * (win_dims.height as usize);
        let win = Window::new(win_handle, win_dims, win_attribs, win_image_len);
        let image_seg = self
            .image_seg
            .get_or_init(|| XShmSeg::new(&self.conn, win.image_len()));
        self.vsync.get_or_init(|| {
            VSyncData::new(
                &mut self.display,
                &self.conn,
                &self.screen,
                self.screen_num,
                self.root_win,
                &win,
            )
        });

        // waiting here to make sure win has already been drawn to
        // otherwise frame grabbing may fail
        // the requirements of shm_get_image on the window aren't clear to me
        // so just wait until it doesn't return an error
        while self
            .conn
            .shm_get_image(
                win.handle(),
                0,
                0,
                win.width(),
                win.height(),
                !0,
                xproto::ImageFormat::Z_PIXMAP.into(),
                image_seg.xid(),
                0,
            )
            .unwrap()
            .reply()
            .is_err()
        {}
        self.win = Some(win);

        // wait a bit since the window hasn't been drawn to yet (ignore the first few frames)
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
    pub fn end(&mut self) {
        if let Some(process) = &mut self.process {
            // this might seem superfluous as the window would be destroyed anyways with process.kill().
            // However, the window only dies shortly after the process does. This can cause
            // problems if Self::start() is called while the window is still alive, as then the
            // zombie window may be used to initiate a new game
            self.conn
                .destroy_window(self.win.as_ref().unwrap().handle())
                .unwrap();
            process.kill().unwrap();
            // TODO: it's possible that waiting actually isn't necessary after process.kill()
            process.wait().unwrap();
            self.process = None;
            self.win = None;
        }
    }
    pub fn next_frame(&mut self) {
        let win = self.win.as_ref().expect("next_frame no win");
        let image_seg = self.image_seg.get().expect("next_frame no image_seg");
        let vsync = self.vsync.get_mut().expect("next_frame no vsync");
        // wait until vblank before grabbing the frame. Prevents tearing
        vsync.wait(&mut self.display);
        self.conn
            .shm_get_image(
                win.handle(),
                0,
                0,
                win.width(),
                win.height(),
                !0,
                xproto::ImageFormat::Z_PIXMAP.into(),
                image_seg.xid(),
                0,
            )
            .unwrap()
            .reply()
            .unwrap();
    }
    pub fn wait_vsync(&mut self) {
        let vsync = self.vsync.get_mut().expect("wait_vsync no vsync");
        vsync.wait(&mut self.display);
    }
    pub fn get_current_frame(&self) -> ImageRef4 {
        let win = self.win.as_ref().expect("get_current_frame no win");
        let image_seg = self
            .image_seg
            .get()
            .expect("get_current_frame no image seg");
        let full_window = ImageRef4::new(u32::from(win.width()), u32::from(win.height()), unsafe {
            std::slice::from_raw_parts(image_seg.address().cast::<u8>(), win.image_len())
        });
        full_window
    }
    pub fn send(&mut self, key: GameKey, kind: KeyEventKind) {
        let win = self.win.as_ref().expect("send no win");
        let keycode = match key {
            GameKey::S => keycodes::KEYCODE_S,
            GameKey::Space => keycodes::KEYCODE_SPACE,
        };
        match kind {
            KeyEventKind::Press => {
                let event = xproto::KeyPressEvent {
                    response_type: xproto::KEY_PRESS_EVENT,
                    root: self.root_win,
                    detail: keycode,
                    event: win.handle(),
                    same_screen: true,
                    time: 0,
                    ..Default::default()
                };
                self.conn
                    .send_event(false, win.handle(), xproto::EventMask::default(), event)
                    .unwrap();
            }
            KeyEventKind::Release => {
                let event = xproto::KeyPressEvent {
                    response_type: xproto::KEY_RELEASE_EVENT,
                    root: self.root_win,
                    detail: keycode,
                    event: win.handle(),
                    same_screen: true,
                    time: 0,
                    ..Default::default()
                };
                self.conn
                    .send_event(false, win.handle(), xproto::EventMask::default(), event)
                    .unwrap();
            }
        }
        self.conn.flush().unwrap();
    }
    pub fn terminate(mut self) {
        if let Some(vsync) = self.vsync.take() {
            vsync.close(&mut self.display, &self.conn);
        }
        if let Some(image_seg) = self.image_seg.take() {
            image_seg.close(&self.conn);
        }
        self.display.close();
    }
    pub fn win(&self) -> &Window {
        self.win.as_ref().expect("win no win")
    }
}
