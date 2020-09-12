use futures::channel::mpsc::Receiver;
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope

use self::callback_future::{callback_stream_sink_info, callback_stream_source_info};
use self::callback_future::{callback_stream_sink_input_info, callback_stream_source_output_info};
use pulse::context::introspect::*;

pub mod to_static;
pub mod callback_future;

// TODO different types for different ids
// #[derive(Clone, Debug)]
// struct SinkId(u32);

pub trait IntrospectorStream<T> {
    fn stream_info_by_index(&self, index: u32) -> Receiver<T>;
    fn stream_info_list(&self) -> Receiver<T>;
}

impl IntrospectorStream<SinkInfo<'static>> for Introspector {
    // fn stream_info_by_index(&self, index: u32) -> Chain<Receiver<SinkInfo<'static>>, Receiver<SinkInfo<'static>>> {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SinkInfo<'static>> {
        let (callback, stream) = callback_stream_sink_info();
        self.get_sink_info_by_index(index, callback);
        stream
    }

    fn stream_info_list(&self) -> Receiver<SinkInfo<'static>> {
        let (callback, stream) = callback_stream_sink_info();
        self.get_sink_info_list(callback);
        stream
    }
}

impl IntrospectorStream<SourceInfo<'static>> for Introspector {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SourceInfo<'static>> {
        let (callback, stream) = callback_stream_source_info();
        self.get_source_info_by_index(index, callback);
        stream
    }

    fn stream_info_list(&self) -> Receiver<SourceInfo<'static>> {
        let (callback, stream) = callback_stream_source_info();
        self.get_source_info_list(callback);
        stream
    }
}

impl IntrospectorStream<SourceOutputInfo<'static>> for Introspector {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SourceOutputInfo<'static>> {
        let (callback, stream) = callback_stream_source_output_info();
        self.get_source_output_info(index, callback);
        stream
    }

    fn stream_info_list(&self) -> Receiver<SourceOutputInfo<'static>> {
        let (callback, stream) = callback_stream_source_output_info();
        self.get_source_output_info_list(callback);
        stream
    }
}

impl IntrospectorStream<SinkInputInfo<'static>> for Introspector {
    fn stream_info_by_index(&self, index: u32) -> Receiver<SinkInputInfo<'static>> {
        let (callback, stream) = callback_stream_sink_input_info();
        self.get_sink_input_info(index, callback);
        stream
    }

    fn stream_info_list(&self) -> Receiver<SinkInputInfo<'static>> {
        let (callback, stream) = callback_stream_sink_input_info();
        self.get_sink_input_info_list(callback);
        stream
    }
}
