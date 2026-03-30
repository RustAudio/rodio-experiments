/// A very ugly macro that will ensure our users get a pretty error. When const
/// formatting becomes available we may remove this monstrosity. We may also get
/// rid of it when we can use expressions in const parameter positions. Then the
/// user will no longer need to pass in the output channel count (aka sum things
/// up in their head).
pub(super) const fn channel_count_mismatch<const OUT: u16, const IN: u16, const N: usize>() -> &'static str {
    use crate::const_source::route_channels::tuple::helpful_compile_time_error::*;
    const MSG_PART_A: &[u8] = b"Wrong output channel count (";
    const MSG_PART_B: &[u8] = b"). It should be the sum of the input channel counts (";
    const MSG_PART_C: &[u8] = b")";
    const PARTS_LEN: usize = MSG_PART_A.len() + MSG_PART_B.len() + MSG_PART_C.len();
    const MAX_NUMBER_LEN: usize = u16::MAX.ilog10() as usize + 1;
    const MAX_LEN: usize = PARTS_LEN + 12 * MAX_NUMBER_LEN;
    let buffer: &[u8] = &const {
        let mut buffer = [0; MAX_LEN];
        // slices are not const so we need this
        let (msg_part_a, template) = buffer.split_at_mut(MSG_PART_A.len());
        msg_part_a.copy_from_slice(MSG_PART_A);
        let bytes_used = write_number(template, OUT);
        let (_, rest) = template.split_at_mut(bytes_used);

        let (msg_part_b, mut template) = rest.split_at_mut(MSG_PART_B.len());
        msg_part_b.copy_from_slice(MSG_PART_B);

        // this is intersperse :) but const
        let mut i = 0;
        loop {
            let bytes_used = write_number(template, IN);
            let (_, rest) = template.split_at_mut(bytes_used);
            let (separator, new_template) = rest.split_at_mut(2);
            template = new_template;
            separator.copy_from_slice(b", ");

            i += 1;
            if i == N - 1 {
                break;
            }
        }

        let bytes_used = write_number(template, IN);
        let (_, rest) = template.split_at_mut(bytes_used);

        let (msg_part_c, _) = rest.split_at_mut(MSG_PART_C.len());
        msg_part_c.copy_from_slice(MSG_PART_C);
        buffer
    };

    let numbers_len = (OUT.ilog10() as usize + 1) + N * (IN.ilog10() as usize + 1);

    let separators_len = 2 * (N - 1);
    let buffer_len = PARTS_LEN + numbers_len + separators_len;
    let (used, _left) = buffer.split_at(buffer_len);

    match core::str::from_utf8(used) {
        Ok(x) => x,
        Err(_) => unreachable!(),
    }
}
