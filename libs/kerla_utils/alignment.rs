pub const fn align_down(value: usize, align: usize) -> usize {
    (value) & !(align - 1)
}

pub const fn align_up(value: usize, align: usize) -> usize {
    align_down(value + align - 1, align)
}

pub const fn is_aligned(value: usize, align: usize) -> bool {
    value & (align - 1) == 0
}

pub const fn align_down_u64(value: u64, align: u64) -> u64 {
    (value) & !(align - 1)
}

pub const fn align_up_u64(value: u64, align: u64) -> u64 {
    align_down_u64(value + align - 1, align)
}
