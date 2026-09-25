fn main() {
    novn_build::Embed::new("assets")
        .exclude(["schema.json", "characters/kaede"])
        .run();
}
