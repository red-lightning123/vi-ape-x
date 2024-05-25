use x11rb::protocol::xproto;

#[derive(Clone)]
pub struct Window {
    handle: u32,
    dims: xproto::GetGeometryReply,
    attribs: xproto::GetWindowAttributesReply,
    image_len: usize,
}

impl Window {
    pub fn new(
        handle: u32,
        dims: xproto::GetGeometryReply,
        attribs: xproto::GetWindowAttributesReply,
        image_len: usize,
    ) -> Self {
        Self {
            handle,
            dims,
            attribs,
            image_len,
        }
    }
    pub fn handle(&self) -> u32 {
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
    pub fn image_len(&self) -> usize {
        self.image_len
    }
    pub fn class(&self) -> xproto::WindowClass {
        self.attribs.class
    }
}
