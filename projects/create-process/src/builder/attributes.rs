use core::{mem::zeroed, ptr::null_mut};

use ntapi::{ntapi_base::CLIENT_ID, ntmmapi::SECTION_IMAGE_INFORMATION, ntpsapi::{NtCurrentProcessId, NtOpenProcess, PS_ATTRIBUTE, PS_ATTRIBUTE_CLIENT_ID, PS_ATTRIBUTE_IMAGE_INFO, PS_ATTRIBUTE_IMAGE_NAME, PS_ATTRIBUTE_LIST, PS_ATTRIBUTE_PARENT_PROCESS}};
use winapi::{ctypes::c_void, shared::{basetsd::SIZE_T, ntdef::{HANDLE, OBJECT_ATTRIBUTES}}, um::winnt::PROCESS_CREATE_PROCESS};

use crate::builder::{error::ProcessBuilderError, types::AlignedBuffer};

#[repr(C)]
pub struct GenericAttributeList<const N: usize> {
    pub length: SIZE_T,
    pub attributes: [PS_ATTRIBUTE; N],
}

pub struct AttributesBuilder<'a, const N: usize> {
    parent_pid: Option<u32>,
    client_id_ptr: *const CLIENT_ID,
    image_info_ptr: *const SECTION_IMAGE_INFORMATION,
    attribute_list_ptr: *mut PS_ATTRIBUTE_LIST,
    nt_image_path_ptr: *mut u16,
    image_path: Option<&'a str>,
    buffer: AlignedBuffer<N>,
    length: usize,
    attribute_count: u32,
}

impl<'a, const N: usize> AttributesBuilder<'a, N> {
    const ATTRIBUTE_SIZE: usize = size_of::<PS_ATTRIBUTE>();
    
    pub fn new() -> Self {
        Self {
            parent_pid: None,
            client_id_ptr: null_mut(),
            image_info_ptr: null_mut(),
            attribute_list_ptr: null_mut(),
            nt_image_path_ptr: null_mut(),
            image_path: None,
            buffer: AlignedBuffer::new(),
            length: 0,
            attribute_count: 0,
        }   
    }

    pub fn as_attr_list<const M: usize>(&self) -> &GenericAttributeList<M> {
        unsafe { &*(self.attribute_list_ptr as *const GenericAttributeList<M>) }
    }

    pub fn as_mut_ptr(&mut self) -> *mut PS_ATTRIBUTE_LIST {
        self.attribute_list_ptr
    }

    pub fn set_image_path(&mut self, path: &'a str) {
        self.image_path = Some(path);
    }

    pub fn set_parent_pid(&mut self, pid: u32) {
        self.parent_pid = Some(pid);
    }

    pub fn client_id(&self) -> &CLIENT_ID {
        unsafe { &*self.client_id_ptr }
    }

    pub fn image_info(&self) -> &SECTION_IMAGE_INFORMATION {
        unsafe { &*self.image_info_ptr }
    }

    fn encode_nt_path(&mut self, path: &str) -> Result<(*mut u16, usize), ProcessBuilderError> {
        let nt_prefix = r"\??\";
        let start_pos = self.length;
        let mut pos = start_pos;
        let ptr = self.buffer.as_mut_ptr() as *mut u16;

        for ch in nt_prefix.encode_utf16() {
            if pos + 2 > N {
                return Err(ProcessBuilderError::BufferOverflow);
            }
            unsafe {
                ptr.add(pos / 2).write(ch);
            }
            pos += 2;
        }

        for ch in path.encode_utf16() {
            if pos + 2 > N {
                return Err(ProcessBuilderError::BufferOverflow);
            }
            unsafe {
                ptr.add(pos / 2).write(ch);
            }
            pos += 2;
        }

        if pos + 2 > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }
        // unsafe {
        //     ptr.add(pos / 2).write(0);
        // }
        // pos += 2;

        self.length = pos;
        unsafe { Ok((ptr.add(start_pos / 2), pos - start_pos)) }
    }

    fn add_attribute_ptr(&mut self, attribute: usize, ptr: *mut c_void, size: usize) -> Result<(), ProcessBuilderError> {

        if self.length + Self::ATTRIBUTE_SIZE > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }

        unsafe {
            let attr_ptr = self.buffer.as_mut_ptr().add(self.length) as *mut PS_ATTRIBUTE;
            let attr = &mut *attr_ptr;
            attr.Attribute = attribute;
            attr.Size = size;
            attr.u.ValuePtr = ptr;
            self.length += Self::ATTRIBUTE_SIZE;
            self.attribute_count += 1;

            let total_length_mut = self.total_length_mut();
            *total_length_mut += size_of::<PS_ATTRIBUTE>();
        }

        Ok(())
    }

    fn add_attribute_value(&mut self, attribute: usize, value: usize, size: usize) -> Result<(), ProcessBuilderError> {

        if self.length + Self::ATTRIBUTE_SIZE > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }

        unsafe {
            let attr_ptr = self.buffer.as_mut_ptr().add(self.length) as *mut PS_ATTRIBUTE;
            let attr = &mut *attr_ptr;
            attr.Attribute = attribute;
            attr.Size = size;
            attr.u.Value = value;
            self.length += Self::ATTRIBUTE_SIZE;
            self.attribute_count += 1;

            let total_length_mut = self.total_length_mut();
            *total_length_mut += size_of::<PS_ATTRIBUTE>();
        }

        Ok(())
    }

    fn align_offset(&self, offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }

    fn alloc_zeroed<T>(&mut self) -> Result<*mut T, ProcessBuilderError> {
        let offset = self.align_offset(self.length, 8);
        self.length = offset;
        
        if self.length + size_of::<T>() > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }
        
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(offset) as *mut T;
            ptr.write(zeroed());
            self.length += size_of::<T>();
            Ok(ptr)
        }
    }

    fn total_length_mut(&mut self) -> &mut usize {
        unsafe { &mut *(self.attribute_list_ptr as *mut usize) }
    }

    fn write_attribute_list_header(&mut self) -> Result<*mut PS_ATTRIBUTE_LIST, ProcessBuilderError> {
        self.length = self.align_offset(self.length, 8);
        
        if self.length + size_of::<PS_ATTRIBUTE_LIST>() > N {
            return Err(ProcessBuilderError::BufferOverflow);
        }
        
        unsafe {
            let list_ptr = self.buffer.as_mut_ptr().add(self.length) as *mut PS_ATTRIBUTE_LIST;
            let list = &mut *list_ptr;
            list.TotalLength = size_of::<usize>();
            self.length += size_of::<usize>();
            Ok(list_ptr)
        }
    }

    fn open_parent_process(&mut self, pid: u32) -> Result<HANDLE, ProcessBuilderError> {
        let mut handle: HANDLE = null_mut();
        let mut obj_attr: OBJECT_ATTRIBUTES = unsafe { zeroed() };
        let mut parent_client: CLIENT_ID = unsafe { zeroed() };
        parent_client.UniqueProcess = pid as _;
        
        let status = unsafe {
            NtOpenProcess(
                &mut handle as *mut _ as _,
                PROCESS_CREATE_PROCESS,
                &mut obj_attr,
                &mut parent_client
            )
        };
        
        if status < 0 {
            return Err(ProcessBuilderError::NtOpenProcessFailed(status));
        }
        
        Ok(handle)
    }

    pub fn build(&mut self) -> Result<(), ProcessBuilderError> {
        let image_path = self.image_path.ok_or(ProcessBuilderError::ImagePathEmpty)?;
        
        let (nt_image_path_ptr, image_name_size) = self.encode_nt_path(image_path)?;
        self.nt_image_path_ptr = nt_image_path_ptr;
        
        self.client_id_ptr = self.alloc_zeroed::<CLIENT_ID>()?;
        self.image_info_ptr = self.alloc_zeroed::<SECTION_IMAGE_INFORMATION>()?;
        self.attribute_list_ptr = self.write_attribute_list_header()?;
        
        self.add_attribute_ptr(
            PS_ATTRIBUTE_IMAGE_NAME,
            nt_image_path_ptr as _,
            image_name_size
        )?;
        
        self.add_attribute_ptr(
            PS_ATTRIBUTE_CLIENT_ID,
            self.client_id_ptr as _,
            size_of::<CLIENT_ID>()
        )?;
        
        self.add_attribute_ptr(
            PS_ATTRIBUTE_IMAGE_INFO,
            self.image_info_ptr as _,
            size_of::<SECTION_IMAGE_INFORMATION>()
        )?;
        
        if let Some(parent_pid) = self.parent_pid {
            let handle = self.open_parent_process(parent_pid)?;
            
            self.add_attribute_value(
                PS_ATTRIBUTE_PARENT_PROCESS,
                handle as usize,
                size_of::<HANDLE>()
            )?;
        }
        
        Ok(())
    }
}