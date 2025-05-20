pub struct EventEntry<F>
where
    F: FnOnce(i32),
{
    time_stamp: i32,
    event: F,
}

pub trait Callable {
    fn call(self);
}

impl<F> Callable for EventEntry<F>
where
    F: FnOnce(i32),
{
    fn call(self) {
        (self.event)(self.time_stamp);
    }
}

impl<F> EventEntry<F>
where
    F: FnOnce(i32),
{
    pub fn new(time_stamp: i32, event: F) -> Option<Self> {
        Some(Self { time_stamp, event })
    }
}
