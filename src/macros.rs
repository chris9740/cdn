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

#[macro_export]
macro_rules! show_welcome_message {
    (
        $config:expr
    ) => {
        pub const VERSION: &str = env!("CARGO_PKG_VERSION");
        pub const EARS: &str = r"/\_/\";
        pub const FACE: &str = "( o.o )";
        pub const WHISK: &str = "> ^ <";
        pub const GITHUB_URL: &str = "https://github.com/chris9740/cdn";

        let ears = EARS.truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2);
        let face = FACE.truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2);
        let whisk = WHISK.truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2);

        println!(
            r"
               rs-cdn {VERSION}
     {ears}     {}
    {face}
     {whisk}     Configuration:
                    - firewall: {}
        ",
            GITHUB_URL.underline(),
            if $config.firewall.enabled {
                "enabled".truecolor(GREEN.0, GREEN.1, GREEN.2)
            } else {
                "disabled".truecolor(RED.0, RED.1, RED.2)
            }
        );
    };
}
