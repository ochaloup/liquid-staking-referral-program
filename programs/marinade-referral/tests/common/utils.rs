pub fn find_value<T, F: FnMut() -> Option<T>>(mut gen: F) -> T {
    loop {
        if let Some(update) = gen() {
            break update;
        }
    }
}

pub fn change_value<T: Eq, F: FnMut() -> T>(old: T, mut gen: F) -> T {
    find_value(|| {
        let update = gen();
        if update != old {
            Some(update)
        } else {
            None
        }
    })
}
