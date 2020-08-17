use futures::channel::mpsc::Receiver;
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope

use self::callback_future::{callback_stream_sink_info, callback_stream_source_info};
use pulse::context::introspect::*;

pub mod callback_future;

// TODO different types for different ids
// #[derive(Clone, Debug)]
// struct SinkId(u32);

pub trait IntrospectorStream<T> {
    fn stream_info_by_index(&self, index: u32) -> Receiver<T>;
}

impl IntrospectorStream<SinkInfo<'static>> for Introspector {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SinkInfo<'static>> {
        let (callback, stream) = callback_stream_sink_info();
        self.get_sink_info_by_index(index, callback);
        stream
    }
}

impl IntrospectorStream<SourceInfo<'static>> for Introspector {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SourceInfo<'static>> {
        let (callback, stream) = callback_stream_source_info();
        self.get_source_info_by_index(index, callback);
        stream
    }
}
