pub trait ResourceProvider<T, C> {
    fn get_texture<'a>(&'a mut self, path: &str, context: &mut C) -> &'a T;
    fn clear(&mut self);
}
