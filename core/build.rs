fn main() {
    cc::Build::new()
        .file("../hal/hardware.c")
        .include("../hal")
        .compile("hal");
}
