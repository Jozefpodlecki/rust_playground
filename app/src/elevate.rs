fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    PCWSTR(wide.as_ptr() as *const u16)
}

fn to_pcwstr_owned(s: &str) -> (Vec<u16>, PCWSTR) {
    let wide: Vec<u16> = OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let ptr = PCWSTR(wide.as_ptr());
    (wide, ptr)
}

fn is_elevated() -> Result<bool> {
    unsafe {
        let mut token = Default::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_len = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as _),
            return_len,
            &mut return_len,
        )?;

        Ok(elevation.TokenIsElevated != 0)
    }
}


// let current_exe = std::env::current_exe()?;
    // let current_exe = current_exe.to_str().unwrap();
    // // let exe_path = to_pcwstr(r"C:\repos\rust_playground\app\target\debug\app.exe");
    // let exe_path = to_pcwstr(r"app.exe");
    // println!("{} {:?}", current_exe, exe_path);
    // let verb = to_pcwstr("runas");
    // let directory = to_pcwstr(r"C:\repos\rust_playground\app\target\debug");
    // let parameters = to_pcwstr("");

    // let result = unsafe {
    //     ShellExecuteW(
    //         None,
    //         verb,
    //         exe_path,
    //         parameters,
    //         directory,
    //         SW_SHOWNORMAL,
    //     )
    // };

    // if result.0 as usize <= 32 {
    //     eprintln!("ShellExecuteW failed with code: {:?}", result.0);
    //     // You may want to exit or handle specific errors here.
    // } else {
    //     println!("Process launched with elevation. {:?}", result.0);
    // }

    // if is_elevated()? {
    //     let test = File::create("C:\\test.txt")?;
    //     sleep(Duration::from_secs(60));
    // }
    // else {
    //     println!("Not elevated.");
    //     sleep(Duration::from_secs(60));
    //     exit(0);
    // }