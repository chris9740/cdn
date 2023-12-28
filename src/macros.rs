#[macro_export]
macro_rules! error {
    ($($message:tt)*) => ({
        use colored::Colorize;
        use $crate::colors::RED;
        eprintln!("{} {}", "[ERROR]".truecolor(RED.0, RED.1, RED.2), format_args!($($message)*));
        std::process::exit(1);
    })
}

#[macro_export]
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

#[macro_export]
macro_rules! info {
    ($($message:tt)*) => ({
        use colored::Colorize;
        println!("{} {}", "[INFO]".blue(), format_args!($($message)*));
    })
}
