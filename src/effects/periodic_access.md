 Calls the `access` closure on `Self` the first time the source is iterated and every
 time `period` elapses.

 Later changes in either `sample_rate()` or `channels_count()` won't be reflected in
 the rate of access.

 The rate is based on playback speed, so both the following will call `access` when the
 same samples are reached:
 `periodic_access(Duration::from_secs(1), ...).speed(2.0)`
 `speed(2.0).periodic_access(Duration::from_secs(2), ...)`
