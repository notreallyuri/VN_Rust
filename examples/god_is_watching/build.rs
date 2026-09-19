fn main() {
    vn_build::Embed::new("assets")
        .exclude(["AUDIO_CREDITS.md", "schema.json"])
        .run();
}
