mod v_sync_data;
mod x_shm_seg;

use crate::keycodes;
use crate::Window;
use crate::X11Display;
use image::ImageRef4;
use std::cell::OnceCell;
use std::process::Command;
use v_sync_data::VSyncData;
use x11rb::connection::Connection;
use x11rb::connection::RequestConnection;
use x11rb::properties::WmClass;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::xcb_ffi::XCBConnection;
use x_shm_seg::XShmSeg;

fn get_children(
    conn: &XCBConnection,
    win: u32,
) -> Result<Vec<u32>, x11rb::rust_connection::ReplyError> {
    let tree = conn.query_tree(win)?.reply()?;
    Ok(tree.children)
}

enum GetWindowPidError {
    Connection(x11rb::rust_connection::ConnectionError),
    Reply(x11rb::rust_connection::ReplyError),
    PropertyFormat,
    EmptyProperty,
}

fn get_window_pid(conn: &XCBConnection, win: u32) -> Result<u32, GetWindowPidError> {
    let wm_pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")
        .map_err(GetWindowPidError::Connection)?
        .reply()
        .map_err(GetWindowPidError::Reply)?
        .atom;
    conn.get_property(false, win, wm_pid_atom, xproto::AtomEnum::CARDINAL, 0, 1)
        .map_err(GetWindowPidError::Connection)?
        .reply()
        .map_err(GetWindowPidError::Reply)?
        .value32()
        .ok_or(GetWindowPidError::PropertyFormat)?
        .next()
        .ok_or(GetWindowPidError::EmptyProperty)
}

fn win_pid_eq(conn: &XCBConnection, win: u32, pid: u32) -> bool {
    // get_window_title may fail on non-UTF8 titles (or possibly if there's an
    // x11 connection error), but such a window wouldn't be the one we're
    // looking for anyway so errors are ignored
    get_window_pid(conn, win).is_ok_and(|win_pid| win_pid == pid)
}

fn find_descendant_win<P>(conn: &XCBConnection, parent: u32, predicate: &mut P) -> Option<u32>
where
    P: FnMut(u32) -> bool,
{
    if predicate(parent) {
        return Some(parent);
    }
    // get_children may fail e. g. if a window was moved/deleted during the call
    // assumes that it's not the right window, errors are ignored
    if let Ok(children) = get_children(conn, parent) {
        for child in children {
            if let Some(win) = find_descendant_win(conn, child, predicate) {
                return Some(win);
            }
        }
    }
    None
}

fn find_descendant_win_with_pid(conn: &XCBConnection, parent: u32, pid: u32) -> Option<u32> {
    find_descendant_win(conn, parent, &mut |win| win_pid_eq(conn, win, pid))
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
    fn wait_for_win_with_pid(&self, pid: u32) -> u32 {
        loop {
            if let Some(win_handle) = find_descendant_win_with_pid(&self.conn, self.root_win, pid) {
                return win_handle;
            }
        }
    }
    pub fn start(&mut self) {
        let process = Command::new("python")
            .current_dir("pixaves_journey")
            .args(["main.py"])
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        // Wait for the game to set up and prepare a window.
        // Technically, waiting for a fixed duration does not guarantee that the
        // game will be ready afterwards. However, I haven't found a more
        // precise way to ensure this, so waiting shall suffice. The duration
        // must be long enough for the game to set up
        std::thread::sleep(std::time::Duration::from_millis(5000));
        let win_handle = self.wait_for_win_with_pid(process.id());
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
        self.image_seg
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
        self.process = Some(process);
        self.win = Some(win);
    }
    pub fn end(&mut self) {
        if let Some(process) = &mut self.process {
            // Empirically, it seems that the window is indirectly destroyed
            // once its owning process is killed. However, I haven't found any
            // documentation of this behavior. It is critical for the program
            // that the window is destroyed before the end of this function.
            // Therefore, to be safe, we destroy it manually
            self.conn
                .destroy_window(self.win.as_ref().unwrap().handle())
                .unwrap();
            // Sync to make sure that the window is destroyed right here
            self.conn.sync().unwrap();
            process.kill().unwrap();
            // Waiting for the process's death is not required since the window
            // is destroyed manually, but it seems cleaner to start without a
            // dangling process, so wait anyway
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
