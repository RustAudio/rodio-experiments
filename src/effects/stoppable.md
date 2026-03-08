Add a method that allows stopping a source before it ends by calling [`Stoppable::stop`] on the source. See [`PeriodicAccess`] to see how to call methods on sources while they are being played.

This can be used to skip items in a queue by calling stop on them ending them
early and triggering the queue to go to the next item.
