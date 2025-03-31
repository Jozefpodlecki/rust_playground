use tokio::sync::mpsc;

pub trait AsBoxedReceiver {
    fn as_boxed_receiver<T>(self) -> Box<mpsc::UnboundedReceiver<T>>;
}

impl AsBoxedReceiver for *mut () {
    fn as_boxed_receiver<T>(self) -> Box<mpsc::UnboundedReceiver<T>> {
        assert!(!self.is_null(), "Received a null pointer!");
        unsafe { Box::from_raw(self as *mut mpsc::UnboundedReceiver<T>) }
    }
}