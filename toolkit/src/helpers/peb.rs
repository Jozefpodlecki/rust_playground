
use core::{fmt, ops::Deref, slice};

use ntapi::{ntpebteb::PEB, ntrtl::RTL_USER_PROCESS_PARAMETERS};
use winapi::{ctypes::c_void, shared::ntdef::UNICODE_STRING};

use crate::{CommandLineArgs, Environment, ExecutablePath, U16CStackString, Utf16Path, println, types::HEAP};



#[unsafe(naked)]
pub unsafe fn get_peb() -> *mut PEB {
    core::arch::naked_asm!(
        "mov rax, gs:[0x60]",
        "ret"
    );
}

pub struct ProcessEnvironmentBlock(*mut PEB);

impl core::ops::Deref for ProcessEnvironmentBlock {
    type Target = PEB;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl ProcessEnvironmentBlock {
    pub fn current_process() -> Self {
        let peb: *mut PEB = unsafe { get_peb() };
        Self(peb)
    }

    pub fn process_params(&self) -> ProcessParameters {
        ProcessParameters(unsafe { (*self.0).ProcessParameters })
    }
    
    pub fn image_base(&self) -> *mut c_void {
        unsafe { (*self.0).ImageBaseAddress }
    }

    pub fn process_heap(&self) -> *mut HEAP {
        unsafe {
            (*self.0).ProcessHeap as *mut HEAP
        }
    }

    pub fn environment(&self) -> Environment {
        let params = unsafe { &*(*self.0).ProcessParameters };
        Environment(params.Environment)
    }

    pub fn command_line(&self) -> CommandLineArgs {
        let params = unsafe { &*(*self.0).ProcessParameters };
        CommandLineArgs(params.CommandLine)
    }

    pub fn executable_path(&self) -> ExecutablePath {
        let params = unsafe { &*(*self.0).ProcessParameters };
        let image_path: UNICODE_STRING = params.ImagePathName;
        let path = Utf16Path::new(image_path.Buffer, (image_path.Length / 2) as usize);
        ExecutablePath::new(path)
    }

}

impl fmt::Display for ProcessEnvironmentBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exe_path = self.executable_path();
        let cmd_line = self.command_line();
        let env = self.environment();
        
        writeln!(f, "Process Environment Block:")?;
        writeln!(f, "  Image Base:              {:p}", self.image_base())?;
        writeln!(f, "  Process Heap:            {:p}", self.process_heap())?;
        writeln!(f, "  Executable Path:         {}", exe_path.display::<260>())?;
        writeln!(f, "  Command Line:            {}", cmd_line)?;
        writeln!(f, "  Inherited Address Space: {}", self.InheritedAddressSpace != 0)?;
        writeln!(f, "  Read Image File Exec Options: {}", self.ReadImageFileExecOptions != 0)?;
        writeln!(f, "  Being Debugged:          {}", self.BeingDebugged != 0)?;
        writeln!(f, "  Mutant:                  {:p}", self.Mutant)?;
        writeln!(f, "  LDR:                     {:p}", self.Ldr)?;
        writeln!(f, "  SubSystem Data:          {:p}", self.SubSystemData)?;
        writeln!(f, "  Fast PEB Lock:           {:p}", self.FastPebLock)?;
        writeln!(f, "  IFEO Key:                {:p}", self.IFEOKey)?;
        writeln!(f, "  Cross Process Flags:     0x{:X}", self.CrossProcessFlags)?;
        writeln!(f, "  API Set Map:             {:p}", self.ApiSetMap)?;
        writeln!(f, "  TLS Expansion Counter:   {}", self.TlsExpansionCounter)?;
        writeln!(f, "  TLS Bitmap:              {:p}", self.TlsBitmap)?;
        writeln!(f, "  Number of Processors:    {}", self.NumberOfProcessors)?;
        writeln!(f, "  Nt Global Flag:          0x{:X}", self.NtGlobalFlag)?;
        writeln!(f, "  Number of Heaps:         {}", self.NumberOfHeaps)?;
        writeln!(f, "  Maximum Number of Heaps: {}", self.MaximumNumberOfHeaps)?;
        writeln!(f, "  Process Heaps:           {:p}", self.ProcessHeaps)?;
        writeln!(f, "  OS Major Version:        {}", self.OSMajorVersion)?;
        writeln!(f, "  OS Minor Version:        {}", self.OSMinorVersion)?;
        writeln!(f, "  OS Build Number:         {}", self.OSBuildNumber)?;
        writeln!(f, "  OS CSD Version:          {}", self.OSCSDVersion)?;
        writeln!(f, "  OS Platform ID:          0x{:X}", self.OSPlatformId)?;
        writeln!(f, "  Image Subsystem:         {}", self.ImageSubsystem)?;
        writeln!(f, "  Image Subsystem Major:   {}", self.ImageSubsystemMajorVersion)?;
        writeln!(f, "  Image Subsystem Minor:   {}", self.ImageSubsystemMinorVersion)?;
        writeln!(f, "  Session ID:              {}", self.SessionId)?;
        writeln!(f, "  Minimum Stack Commit:    {}", self.MinimumStackCommit)?;
        writeln!(f, "Environment Variables:")?;
        
        for (key, value) in env.iter() {
            writeln!(f, "    {}={}", key, value)?;
        }
        
        Ok(())
    }
}

pub struct ProcessParameters(*mut RTL_USER_PROCESS_PARAMETERS);

impl ProcessParameters {
    pub fn as_ptr(&self) -> *mut RTL_USER_PROCESS_PARAMETERS {
        self.0
    }
}

impl Deref for ProcessParameters {
    type Target = RTL_USER_PROCESS_PARAMETERS;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}