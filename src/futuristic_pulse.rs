use futures::channel::mpsc::Receiver;
use futures::Stream;
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope

use pulse::context::introspect::*;
use self::callback_future::{callback_stream_sink_info, callback_stream_source_info};

pub mod callback_future;


pub trait FuturisticIntrospector<'r> {
    fn stream_sink_info_by_index(&self, index: u32) -> Receiver<SinkInfo<'static>>;
    fn stream_source_info_by_index(&self, index: u32) -> Receiver<SourceInfo<'static>>;
}

impl FuturisticIntrospector<'static> for Introspector {
    fn stream_sink_info_by_index(&self, index: u32) -> Receiver<SinkInfo<'static>> {
        let (callback, stream) = callback_stream_sink_info();
        self.get_sink_info_by_index(index, callback);
        stream
    }

    fn stream_source_info_by_index(&self, index: u32) -> Receiver<SourceInfo<'static>> {
        let (callback, stream) = callback_stream_source_info();
        self.get_source_info_by_index(index, callback);
        stream
    }
}