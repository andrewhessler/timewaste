use std::{env, os::unix::net::UnixStream, path::Path};

pub fn display_connect() {
    let socket_name = env::var("WAYLAND_DISPLAY").unwrap();
    let xdg_dir = env::var("XDG_RUNTIME_DIR").unwrap();

    let socket_path = Path::new(&xdg_dir).join(socket_name);
    println!("{socket_path:?}");

    let mut stream = UnixStream::connect(socket_path);

    match stream {
        Ok(_) => println!("Got it!"),
        Err(_) => println!("Sad"),
    }
}
