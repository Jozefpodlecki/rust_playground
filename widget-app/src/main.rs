use libloading::{Library, Symbol};

#[repr(C)]
#[derive(Debug)]
pub struct Widget {
    pub id: u32,
    pub value: f64,
}

fn main() {
    unsafe {
        let lib = Library::new(r"C:\repos\rust_playground\widget-dll\target\debug\widget_dll.dll")
            .expect("Failed to load DLL");

        type MakeWidgetFn = extern "C" fn(u32, f64) -> Widget;

        let make_widget: Symbol<MakeWidgetFn> = lib.get(b"make_widget")
            .expect("Failed to load function");

        // Call the DLL function
        let w = make_widget(123, 4.56);
        println!("Got widget: {:?}", w);

        lib.close().unwrap();
    }
}