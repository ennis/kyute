use kyute_shell::platform::Platform;

#[test]
fn test_simple() {
    let platform = unsafe { Platform::init().expect("could not initialize platform services") };

    // here: must be able to create a text layout
    let text_format = TextFormat::new(platform.renderer());
}
