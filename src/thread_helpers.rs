#[macro_export]
macro_rules! send_error_command {
    ($cmd_sender:expr, $error:expr) => {
        // Unwrapping as thread error should not have to be dealt by users.
        $cmd_sender.send(Err(::anyhow::anyhow!($error))).unwrap()
    };
}

pub use send_error_command;
