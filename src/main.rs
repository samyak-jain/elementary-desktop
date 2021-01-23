use druid::{widget::Label, AppLauncher, PlatformError, Widget, WindowDesc};

fn build_ui() -> impl Widget<()> {
    Label::new("Hello World")
}

fn main() -> Result<(), PlatformError> {
    AppLauncher::with_window(WindowDesc::new(build_ui)).launch(())?;
    Ok(())
}
