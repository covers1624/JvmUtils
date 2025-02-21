use env_logger::Env;
use jvm_utils::locator::LocatorBuilder;

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let locator = LocatorBuilder::new()
        .with_platform_locator()
        .with_intellij_locator()
        .with_gradle_locator();

    for x in locator.locate() {
        let version = x.lang_version;
        let path = x.java_home;
        println!("Found java version {version:?} at {path:?}")
    }

    Ok(())
}
