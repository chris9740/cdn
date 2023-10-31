macro_rules! error {
    ($($message:tt)*) => ({
        eprintln!($($message)*);
        std::process::exit(1);
    })
}

macro_rules! unwrap_or_return {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(val) => val,
            Err(_) => {
                return Err($error);
            }
        }
    };
}

pub(crate) use error;
pub(crate) use unwrap_or_return;
