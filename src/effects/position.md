Start tracking the elapsed duration since the start of the underlying
source.

If a speedup and or delay is applied after this that will not be reflected
in the position returned by [`get_pos`](TrackPosition::get_pos).

This can get confusing when using [`get_pos()`](TrackPosition::get_pos)
together with [`crate::DynamicSource::try_seek()`] as the latter does take all
speedup's and delay's into account. It's recommended therefore to apply
track_position after speedup's and delay's.
