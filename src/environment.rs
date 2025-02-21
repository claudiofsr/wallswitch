use std::{borrow::Cow, env, sync::LazyLock};

/// Get environment variables
///
/// A value which is initialized on the first access
pub static ENVIRON: LazyLock<Environment> = LazyLock::new(Environment::new);

// https://doc.rust-lang.org/std/borrow/enum.Cow.html
// https://dhghomon.github.io/easy_rust/Chapter_42.html
// https://dev.to/kgrech/6-things-you-can-do-with-the-cow-in-rust-4l55

/// Environment variables
pub struct Environment<'a> {
    pub desktop: Cow<'a, str>,
    pub home: Cow<'a, str>,
    pub pkg_name: Cow<'a, str>,
}

impl Environment<'_> {
    pub fn new() -> Environment<'static> {
        let home: String = get_home();
        let desktop: String = get_desktop();
        let pkg_name: String = get_pkg_name("wallswitch");

        Environment {
            desktop: Cow::Owned(desktop),
            home: Cow::Owned(home),
            pkg_name: Cow::Owned(pkg_name),
        }
    }

    pub fn get_desktop(&self) -> &str {
        &self.desktop
    }

    pub fn get_home(&self) -> &str {
        &self.home
    }

    pub fn get_pkg_name(&self) -> &str {
        &self.pkg_name
    }
}

/// echo $HOME
fn get_home() -> String {
    match env::var("HOME") {
        Ok(home) => home,
        Err(why) => {
            eprintln!("echo $HOME");
            panic!("Error: Unable to get home path! {why}");
        }
    }
}

/// Get desktop name
///
/// env | grep -i desktop
///
/// POP OS Example:
///
/// * XDG_CURRENT_DESKTOP=pop:GNOME
///
/// * DESKTOP_SESSION=pop
///
/// * XDG_SESSION_DESKTOP=pop
///
/// echo $DESKTOP_SESSION
fn get_desktop() -> String {
    let mut desktops: Vec<String> = Vec::new();

    for key in [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ] {
        if let Ok(desktop) = env::var(key) {
            desktops.push(desktop.trim().to_lowercase());
        }
    }

    // Sort (in ascending order) a vector of strings according to string lengths.
    desktops.sort_by_key(|desktop| desktop.chars().count());

    // println!("desktops: {desktops:#?}");

    // The last item is the longest string and probably contains
    // the most information about the desktop name.
    match desktops.last() {
        Some(desktop) => desktop.to_string(),
        None => panic!("Error: Unable to get desktop type!"),
    }
}

/// Get the package name
///
/// std::env::current_exe()
///
/// <https://doc.rust-lang.org/std/env/fn.current_exe.html>
fn get_pkg_name(default_name: &str) -> String {
    /*
    let pkg_path = std::env::current_exe().expect("Error: path not found!");
    let pkg_name = pkg_path.file_name().expect("Error: pkg_name not found!");
    println!("pkg_name: {pkg_name:?}");
    */
    match env::var("CARGO_PKG_NAME") {
        Ok(cargo_pkg_name) => cargo_pkg_name,
        Err(_) => default_name.to_string(),
    }
}
