fn main() {
    vn_build::Embed::new("assets")
        .exclude(["schema.json", "characters/kaede"])
        .run();
}
