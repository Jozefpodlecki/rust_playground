use core::mem;

use toolkit::U16CStackString;
use winapi::{shared::{guiddef::GUID, minwindef::{DWORD, FALSE}}, um::setupapi::{DIGCF_ALLCLASSES, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, PSP_DEVICE_INTERFACE_DATA, PSP_DEVICE_INTERFACE_DETAIL_DATA_W, PSP_DEVINFO_DATA, SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA, SPDRP_CLASS, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME, SPDRP_HARDWAREID, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW}};

pub struct HidDeviceInfo {
    pub device_info_set: HDEVINFO,
    pub device_path: U16CStackString<260>,
    pub interface_data: SP_DEVICE_INTERFACE_DATA,
    pub device_info_data: SP_DEVINFO_DATA,
}

impl HidDeviceInfo {
    pub fn get_device_path(&self) -> Option<U16CStackString<260>> {
        unsafe {
            let mut required_size = 0;
            let mut interface_data = self.interface_data;
            let mut device_info_data = self.device_info_data;
            
            // First call to get required size
            let status = SetupDiGetDeviceInterfaceDetailW(
                self.device_info_set,
                &mut interface_data as *mut _,
                core::ptr::null_mut(),
                0,
                &mut required_size,
                &mut device_info_data as *mut _,
            );

            if status == FALSE && required_size == 0 {
                return None;
            }

            // Allocate exact size
            let mut detail_buffer = vec![0u8; required_size as usize];
            let detail_data = detail_buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*detail_data).cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

            let status = SetupDiGetDeviceInterfaceDetailW(
                self.device_info_set,
                &mut interface_data as *mut _,
                detail_data,
                required_size,
                &mut required_size,
                &mut device_info_data as *mut _,
            );

            if status == FALSE {
                return None;
            }

            let device_path_ptr = &(*detail_data).DevicePath as *const u16;
            let mut path = U16CStackString::<260>::new();
            
            let mut i = 0;
            while i < 260 {
                let ch = *device_path_ptr.add(i);
                if ch == 0 {
                    break;
                }
                path.push(ch);
                i += 1;
            }

            Some(path)
        }
    }

     pub fn get_device_property(&self, property: DWORD) -> Option<U16CStackString<260>> {
        unsafe {
            let mut required_size = 0;
            
            // First call to get required size
            let status = SetupDiGetDeviceRegistryPropertyW(
                self.device_info_set,
                &mut self.device_info_data.clone(),
                property,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                &mut required_size,
            );

            if status == FALSE && required_size == 0 {
                return None;
            }

            let mut buffer = vec![0u16; (required_size / 2) as usize + 1];
            let status = SetupDiGetDeviceRegistryPropertyW(
                self.device_info_set,
                &mut self.device_info_data.clone(),
                property,
                core::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                required_size,
                &mut required_size,
            );

            if status == FALSE {
                return None;
            }

            let mut path = U16CStackString::<260>::new();
            let mut i = 0;
            while i < buffer.len() && buffer[i] != 0 {
                path.push(buffer[i]);
                i += 1;
            }
            Some(path)
        }
    }

    pub fn description(&self) -> Option<U16CStackString<260>> {
        self.get_device_property(SPDRP_DEVICEDESC)
    }

    pub fn friendly_name(&self) -> Option<U16CStackString<260>> {
        self.get_device_property(SPDRP_FRIENDLYNAME)
    }

    pub fn hardware_id(&self) -> Option<U16CStackString<260>> {
        self.get_device_property(SPDRP_HARDWAREID)
    }

    pub fn class_name(&self) -> Option<U16CStackString<260>> {
        self.get_device_property(SPDRP_CLASS)
    }
}

pub struct HidDeviceIterator {
    device_info_set: HDEVINFO,
    member_index: DWORD,
    class_guid: Option<GUID>,
    all_devices: bool,
    current_device: Option<HidDeviceInfo>,
}

impl HidDeviceIterator {
    pub fn all() -> Result<Self, &'static str> {
        unsafe {
            let device_info_set = SetupDiGetClassDevsW(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                DIGCF_ALLCLASSES | DIGCF_PRESENT,
            );

            if device_info_set == 0xFFFFFFFF as HDEVINFO {
                return Err("Failed to get device class");
            }

            Ok(HidDeviceIterator {
                device_info_set,
                member_index: 0,
                class_guid: None,
                all_devices: true,
                current_device: None,
            })
        }
    }

    pub fn new(class_guid: GUID) -> Result<Self, &'static str> {
        unsafe {
            let device_info_set = SetupDiGetClassDevsW(
                &class_guid,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );

            if device_info_set == 0xFFFFFFFF as HDEVINFO {
                return Err("Failed to get device class");
            }

            Ok(HidDeviceIterator {
                device_info_set,
                member_index: 0,
                class_guid: Some(class_guid),
                all_devices: false,
                current_device: None,
            })
        }
    }
}

impl Iterator for HidDeviceIterator {
    type Item = HidDeviceInfo;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.all_devices {
                // Use SetupDiEnumDeviceInfo for all devices
                let mut device_info_data: SP_DEVINFO_DATA = mem::zeroed();
                device_info_data.cbSize = mem::size_of::<SP_DEVINFO_DATA>() as u32;

                let status = SetupDiEnumDeviceInfo(
                    self.device_info_set,
                    self.member_index,
                    &mut device_info_data,
                );

                if status == FALSE {
                    return None;
                }

                self.member_index += 1;

                // For all devices, we don't have an interface path
                // Return a placeholder with empty path
                let interface_data: SP_DEVICE_INTERFACE_DATA = mem::zeroed();
                let path = U16CStackString::<260>::new();

                return Some(HidDeviceInfo {
                    device_info_set: self.device_info_set,
                    device_path: path,
                    interface_data,
                    device_info_data,
                });
            }

            // Original HID device interface enumeration
            let mut interface_data: SP_DEVICE_INTERFACE_DATA = mem::zeroed();
            interface_data.cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;

            let mut device_info_data: SP_DEVINFO_DATA = mem::zeroed();
            device_info_data.cbSize = mem::size_of::<SP_DEVINFO_DATA>() as u32;

            let class_guid = match self.class_guid.as_mut() {
                Some(guid) => guid,
                None => core::ptr::null_mut(),
            };

            let status = SetupDiEnumDeviceInterfaces(
                self.device_info_set,
                core::ptr::null_mut(),
                class_guid,
                self.member_index,
                &mut interface_data,
            );

            if status == FALSE {
                return None;
            }

            self.member_index += 1;

            let mut required_size = 0;
            let mut detail_buffer = vec![0u8; 1024];
            let detail_data = detail_buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*detail_data).cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

            let status = SetupDiGetDeviceInterfaceDetailW(
                self.device_info_set,
                &mut interface_data,
                detail_data,
                detail_buffer.len() as u32,
                &mut required_size,
                &mut device_info_data,
            );

            if status == FALSE {
                return None;
            }

            let device_path_ptr = &(*detail_data).DevicePath as *const u16;
            let mut path = U16CStackString::<260>::new();
            
            let mut i = 0;
            while i < 260 {
                let ch = *device_path_ptr.add(i);
                if ch == 0 {
                    break;
                }
                path.push(ch);
                i += 1;
            }

            Some(HidDeviceInfo {
                device_info_set: self.device_info_set,
                device_path: path,
                interface_data,
                device_info_data,
            })
        }
    }
}

impl Drop for HidDeviceIterator {
    fn drop(&mut self) {
        unsafe {
            if self.device_info_set != 0xFFFFFFFF as HDEVINFO {
                SetupDiDestroyDeviceInfoList(self.device_info_set);
            }
        }
    }
}
