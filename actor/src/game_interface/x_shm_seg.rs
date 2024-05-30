use x11rb::connection::Connection;
use x11rb::protocol::shm::ConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

pub struct XShmSeg {
    address: *mut core::ffi::c_void,
    x_seg: u32,
}

impl XShmSeg {
    pub fn new(conn: &XCBConnection, len: usize) -> Self {
        let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, len, libc::IPC_CREAT | 0o777) };
        let address = unsafe { libc::shmat(shmid, std::ptr::null(), 0) };
        let x_seg = conn.generate_id().unwrap();
        conn.shm_attach(x_seg, shmid as u32, false).unwrap();
        Self { address, x_seg }
    }
    pub fn address(&self) -> *mut core::ffi::c_void {
        self.address
    }
    pub fn xid(&self) -> u32 {
        self.x_seg
    }
    pub fn close(self, conn: &XCBConnection) {
        conn.shm_detach(self.x_seg).unwrap();
        unsafe { libc::shmdt(self.address) };
    }
}
