pub trait OptionMutExt<T> {
    fn mutate<V, Fn>(&mut self, make: Fn) -> Option<V>
    where
        Fn: FnOnce(T) -> Option<V>;
}

impl<T> OptionMutExt<T> for Option<T> {
    fn mutate<V, Fn>(&mut self, make: Fn) -> Option<V>
    where
        Fn: FnOnce(T) -> Option<V>,
    {
        if self.is_some() {
            make(self.take().unwrap())
        } else {
            None
        }
    }
}
