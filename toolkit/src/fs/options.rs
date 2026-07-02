use ntapi::ntioapi::*;
use winapi::um::winnt::*;

#[derive(Clone, Copy)]
pub struct FileOptions {
    pub access: u32,
    pub share: u32,
    pub create_options: u32,
    pub create_disposition: u32,
    pub attributes: u32,
}

impl FileOptions {
    pub fn new() -> Self {
        Self {
            access: FILE_READ_DATA | SYNCHRONIZE,
            share: FILE_SHARE_READ,
            create_options: 0,
            create_disposition: FILE_OPEN,
            attributes: 0,
        }
    }

    pub fn read(&mut self) -> &mut Self {
        self.access |= FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.access |= FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
        self
    }

    pub fn read_write(&mut self) -> &mut Self {
        self.access |= FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
        self
    }

    pub fn append(&mut self) -> &mut Self {
        self.access |= FILE_APPEND_DATA | SYNCHRONIZE;
        self
    }

    pub fn share_read(&mut self) -> &mut Self {
        self.share |= FILE_SHARE_READ;
        self
    }

    pub fn share_write(&mut self) -> &mut Self {
        self.share |= FILE_SHARE_WRITE;
        self
    }

    pub fn share_delete(&mut self) -> &mut Self {
        self.share |= FILE_SHARE_DELETE;
        self
    }

    pub fn share_all(&mut self) -> &mut Self {
        self.share |= FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.create_disposition = FILE_CREATE;
        self
    }

    pub fn create_always(&mut self) -> &mut Self {
        self.create_disposition = FILE_CREATE;
        self
    }

    pub fn open_always(&mut self) -> &mut Self {
        self.create_disposition = FILE_OPEN_IF;
        self
    }

    pub fn truncate(&mut self) -> &mut Self {
        self.create_disposition = FILE_OVERWRITE;
        self
    }

    pub fn truncate_always(&mut self) -> &mut Self {
        self.create_disposition = FILE_OVERWRITE_IF;
        self
    }

    pub fn attributes(&mut self, attrs: u32) -> &mut Self {
        self.attributes = attrs;
        self
    }

    pub fn hidden(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_HIDDEN;
        self
    }

    pub fn readonly(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_READONLY;
        self
    }

    pub fn system(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_SYSTEM;
        self
    }

    pub fn temporary(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_TEMPORARY;
        self
    }

    pub fn archive(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_ARCHIVE;
        self
    }

    pub fn normal(&mut self) -> &mut Self {
        self.attributes |= FILE_ATTRIBUTE_NORMAL;
        self
    }

    pub fn synchronous(&mut self) -> &mut Self {
        self.create_options |= FILE_SYNCHRONOUS_IO_NONALERT;
        self
    }

    pub fn asynchronous(&mut self) -> &mut Self {
        self.create_options &= !FILE_SYNCHRONOUS_IO_NONALERT;
        self
    }

    pub fn no_intermediate_buffering(&mut self) -> &mut Self {
        self.create_options |= FILE_NO_INTERMEDIATE_BUFFERING;
        self
    }

    pub fn random_access(&mut self) -> &mut Self {
        self.create_options |= FILE_RANDOM_ACCESS;
        self
    }

    pub fn sequential_scan(&mut self) -> &mut Self {
        self.create_options |= FILE_SEQUENTIAL_ONLY;
        self
    }

    pub fn delete_on_close(&mut self) -> &mut Self {
        self.create_options |= FILE_DELETE_ON_CLOSE;
        self
    }

    pub fn open_reparse_point(&mut self) -> &mut Self {
        self.create_options |= FILE_OPEN_REPARSE_POINT;
        self
    }

    pub fn open_no_recall(&mut self) -> &mut Self {
        self.create_options |= FILE_OPEN_NO_RECALL;
        self
    }

    pub fn build(&self) -> (u32, u32, u32, u32, u32) {
        (self.access, self.share, self.create_options, self.create_disposition, self.attributes)
    }
}