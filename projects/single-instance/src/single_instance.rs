#[macro_export]
macro_rules! single_instance {
    ($name:expr) => {
        use ntapi::ntexapi::NtCreateMutant;
        use winapi::{shared::{ntdef::{HANDLE, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING}, ntstatus::STATUS_OBJECT_NAME_COLLISION}, um::{winnt::MUTANT_ALL_ACCESS}};

        pub enum MutexState {
            Unchecked,
            NotSingle,
            Single(HANDLE)
        }
            
        pub struct SingleInstanceImpl<const N: usize>  {
            state: MutexState,
            buffer: [u16; N],
        }

        impl<const N: usize> SingleInstanceImpl<N> {

            pub const fn new_with_buffer(name: &[u8], buffer: [u16; N]) -> Self {
                const PREFIX: &[u8] = b"\\BaseNamedObjects\\";
                let mut wide_name: [u16; N] = buffer;
                let mut pos = 0;
                
                while pos < PREFIX.len() && pos < BUFFER_LEN {
                    wide_name[pos] = PREFIX[pos] as u16;
                    pos += 1;
                }
                
                let mut i = 0;
                while i < name.len() && pos < BUFFER_LEN {
                    wide_name[pos] = name[i] as u16;
                    pos += 1;
                    i += 1;
                }
                
                if pos < BUFFER_LEN {
                    wide_name[pos] = 0;
                }
                
                Self {
                    state: MutexState::Unchecked,
                    buffer: wide_name,
                }
            }

            pub fn setup_and_check() -> bool {
                let buffer = unsafe { &mut SINGLE_INSTANCE.buffer };
                let state = unsafe { &mut SINGLE_INSTANCE.state };

                match state {
                    MutexState::Unchecked => {},
                    MutexState::NotSingle => return false,
                    MutexState::Single(_) => return true,
                }

                let mut len = 0;
                while len < BUFFER_LEN && buffer[len] != 0 {
                    len += 1;
                }
                
                let mut uc_name = UNICODE_STRING {
                    Length: (len * 2) as u16,
                    MaximumLength: (len * 2 + 2) as u16,
                    Buffer: buffer.as_mut_ptr(),
                };
                
                let mut obj_attr = OBJECT_ATTRIBUTES {
                    Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
                    RootDirectory: core::ptr::null_mut(),
                    ObjectName: &mut uc_name,
                    Attributes: OBJ_CASE_INSENSITIVE,
                    SecurityDescriptor: core::ptr::null_mut(),
                    SecurityQualityOfService: core::ptr::null_mut(),
                };
                
                let mut handle: HANDLE = core::ptr::null_mut();
                let status = unsafe { NtCreateMutant(&mut handle, MUTANT_ALL_ACCESS, &mut obj_attr, 0) };
                
                if status == 0 {
                    *state = MutexState::Single(handle);
                    true
                } else if status == STATUS_OBJECT_NAME_COLLISION {
                    false
                } else {
                    false
                }
            }
        }

        const BUFFER_LEN: usize = 18 + $name.len() + 1;
        static mut SINGLE_INSTANCE: $crate::SingleInstanceImpl<BUFFER_LEN> = 
            $crate::SingleInstanceImpl::new_with_buffer($name, [0u16; BUFFER_LEN]);

        pub type SingleInstance = SingleInstanceImpl<BUFFER_LEN>;
    };
}