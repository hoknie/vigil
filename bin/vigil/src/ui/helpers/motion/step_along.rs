pub fn step_along<C: Copy + PartialEq>(from: C, by: isize, shown: &[C]) -> C {
    if shown.is_empty() {
        return from;
    }
    let count = shown.len() as isize;
    let at = shown.iter().position(|name| *name == from).unwrap_or(0) as isize;
    shown[(at + by).rem_euclid(count) as usize]
}
