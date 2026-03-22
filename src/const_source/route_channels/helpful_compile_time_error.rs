pub(crate) const fn assert_channel_counts<const OUT: u16, const IN1: u16, const IN2: u16>() {
    const {
        assert!(
            IN1 + IN2 == OUT,
            "{}",
            channel_count_mismatch::<OUT, IN1, IN2>()
        )
    }
}

pub(crate) const fn write_number(slice: &mut [u8], mut n: u16) -> usize {
    let digits = n.ilog10() as usize + 1;
    let mut i = digits;
    while i > 0 {
        i -= 1;
        slice[i] = (n % 10) as u8 + b'0';
        n /= 10;
    }

    digits
}

pub(crate) const fn channel_count_mismatch<const OUT: u16, const IN1: u16, const IN2: u16>()
-> &'static str {
    const MSG_PART_A: &[u8] = b"Wrong output channel count (";
    const MSG_PART_B: &[u8] = b"). Output must be the sum of the inputs (";
    const MSG_PART_C: &[u8] = b")";

    const PARTS_LEN: usize = MSG_PART_A.len() + MSG_PART_B.len() + MSG_PART_C.len();
    const MAX_NUMBER_LEN: usize = u16::MAX.ilog10() as usize + 1;
    const MAX_LEN: usize = PARTS_LEN + 2 * MAX_NUMBER_LEN;
    let buffer: &[u8] = &const {
        // let buffer: &[u8] = &{
        let mut buffer = [0; MAX_LEN];
        // slices are not const so we need this
        let (msg_part_a, template) = buffer.split_at_mut(MSG_PART_A.len());
        msg_part_a.copy_from_slice(MSG_PART_A);
        let bytes_used = write_number(template, OUT);
        let (_, rest) = template.split_at_mut(bytes_used);

        let (msg_part_b, template) = rest.split_at_mut(MSG_PART_B.len());
        msg_part_b.copy_from_slice(MSG_PART_B);
        let bytes_used = write_number(template, IN1);
        let (_, rest) = template.split_at_mut(bytes_used);

        let (separator, template) = rest.split_at_mut(2);
        separator.copy_from_slice(b", ");
        let bytes_used = write_number(template, IN2);
        let (_, rest) = template.split_at_mut(bytes_used);

        let (msg_part_c, _) = rest.split_at_mut(MSG_PART_C.len());
        msg_part_c.copy_from_slice(MSG_PART_C);
        buffer
    };

    let numbers_len =
        (OUT.ilog10() as usize + 1) + IN1.ilog10() as usize + 1 + IN2.ilog10() as usize + 1;
    let separators_len = (2 - 1) * 2;
    let buffer_len = PARTS_LEN + numbers_len + separators_len;
    let (used, _left) = buffer.split_at(buffer_len);

    match core::str::from_utf8(used) {
        Ok(x) => x,
        Err(_) => unreachable!(),
    }
    // }
}

