mod syscall_test;
mod vulkano_test;
mod wayland_client_test;

fn main() {
    wayland_client_test::display_connect();
}
