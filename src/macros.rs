use regex::Regex;

pub fn strip_colors(input: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[mGK]").unwrap();
    re.replace_all(input, "").to_string()
}


#[macro_export]
macro_rules! error {
    ($($message:tt)*) => ({
        use colored::Colorize;
        use $crate::colors::RED;

        let formatted = format!("{}", format_args!($($message)*));
        log::error!("{}", $crate::macros::strip_colors(&formatted));
        eprintln!("{} {}", "[ERROR]".truecolor(RED.0, RED.1, RED.2), format_args!($($message)*));
        std::process::exit(1);
    })
}

#[macro_export]
macro_rules! info {
    ($($message:tt)*) => ({
        use colored::Colorize;
        let formatted = format!("{}", format_args!($($message)*));
        log::info!("{}", $crate::macros::strip_colors(&formatted));
        println!("{} {}", "[INFO]".blue(), format_args!($($message)*));
    })
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                use $crate::info;

                info!("Caught error: {}", err.to_string());

                return Err($error);
            }
        }
    };
}
