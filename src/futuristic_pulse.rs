use crate::futuristic_pulse::PulseMessage::MsgDel;
use crate::futuristic_pulse::PulseMessage::MsgAdd;
use futures::channel::mpsc::Receiver;
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope

use self::callback_future::{callback_stream_sink_info, callback_stream_source_info};
use self::callback_future::{callback_stream_sink_input_info, callback_stream_source_output_info};
use pulse::context::introspect::*;

use futures::channel::mpsc::channel;
use futures::channel::mpsc::Sender;
use futures::stream::once;
use futures::stream::{empty, StreamExt};
use libpulse_binding::context::introspect::SourceInfo;
use pulse::context::subscribe::subscription_masks;
use pulse::context::Context;
use pulse::context::{
    introspect::{SinkInfo, SinkInputInfo, SourceOutputInfo},
    subscribe::*,
};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use tokio::runtime::Runtime;
use tokio::spawn;


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


pub fn init_pulse<'a>(sender: Sender<PulseMessage<'static>>) -> () {
    let spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_S16NE,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "FooApp")
        .unwrap();

    let mainloop = Rc::new(RefCell::new(
        Mainloop::new().expect("Failed to create mainloop"),
    ));

    let context = Rc::new(RefCell::new(
        Context::new_with_proplist(mainloop.borrow().deref(), "PulseControlContext", &proplist)
            .expect("Failed to create new context"),
    ));

    context
        .borrow_mut()
        .connect(None, pulse::context::flags::NOFLAGS, None)
        .expect("Failed to connect context");

    // recv implements Stream
    let (mut send, recv): (Sender<RawPulseMessage>, Receiver<RawPulseMessage>) = channel(1024); // TODO channel size

    // Wait for context to be ready
    loop {
        match mainloop.borrow_mut().iterate(false) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                panic!("Iterate state was not success, quitting...");
            }
            IterateResult::Success(_) => {}
        }
        match context.borrow().get_state() {
            pulse::context::State::Ready => {
                break;
            }
            pulse::context::State::Failed | pulse::context::State::Terminated => {
                panic!("Context state failed/terminated, quitting...");
            }
            _ => {}
        }
    }

    let interest = subscription_masks::ALL;

    context
        .borrow_mut()
        .set_subscribe_callback(Some(Box::new(move |fac, op, i| match (fac, op, i) {
            (Some(facility), Some(operation), index) => match send.try_send(RawPulseMessage {
                facility,
                operation,
                index,
            }) {
                Ok(_) => (),
                Err(err) => eprintln!("Got error {}", err),
            },
            _ => eprintln!("Got weird Message: {:?} | {:?} | {:?}", fac, op, i),
        })));
    eprintln!("Set callback.");

    let introspector = context.borrow_mut().introspect();

    let _op = context.borrow_mut().subscribe(
        interest, // Our interest mask
        |args| {
            eprintln!("Subscribed. {}", args);
        },
    );

    let init_sink_stream = introspector
        .stream_info_list()
        .map(|info: SinkInfo| MsgAdd {
            id: info.index,
            msg: PulseAddMessage::MsgSink(info),
        })
        .boxed();
    let init_source_stream = introspector
        .stream_info_list()
        .map(|info: SourceInfo| MsgAdd {
            id: info.index,
            msg: PulseAddMessage::MsgSource(info),
        })
        .boxed();
    let init_sink_input_stream = introspector
        .stream_info_list()
        .map(|info: SinkInputInfo| MsgAdd {
            id: info.index,
            msg: PulseAddMessage::MsgSinkInput(info),
        })
        .boxed();
    let init_source_output_stream = introspector
        .stream_info_list()
        .map(|info: SourceOutputInfo| MsgAdd {
            id: info.index,
            msg: PulseAddMessage::MsgSourceOutput(info),
        })
        .boxed();
    let live_stream = recv.flat_map({
        move |raw| match (raw.facility, raw.operation) {
            (_, Operation::Removed) => once(async move { MsgDel { id: raw.index } }).boxed(),
            (Facility::Sink, _) => introspector
                .stream_info_by_index(raw.index)
                .map(move |info| MsgAdd {
                    id: raw.index,
                    msg: PulseAddMessage::MsgSink(info),
                })
                .boxed(),
            (Facility::Source, _) => introspector
                .stream_info_by_index(raw.index)
                .map(move |info| MsgAdd {
                    id: raw.index,
                    msg: PulseAddMessage::MsgSource(info),
                })
                .boxed(),
            (Facility::SinkInput, _) => introspector
                .stream_info_by_index(raw.index)
                .map(move |info| MsgAdd {
                    id: raw.index,
                    msg: PulseAddMessage::MsgSinkInput(info),
                })
                .boxed(),
            (Facility::SourceOutput, _) => introspector
                .stream_info_by_index(raw.index)
                .map(move |info| MsgAdd {
                    id: raw.index,
                    msg: PulseAddMessage::MsgSourceOutput(info),
                })
                .boxed(),
            (Facility::Module, _) => empty().boxed(),
            (Facility::Client, _) => empty().boxed(),
            (Facility::SampleCache, _) => empty().boxed(),
            (Facility::Server, _) => empty().boxed(),
            (Facility::Card, _) => empty().boxed(),
        }
    });

    let pulse_stream = init_sink_stream
        .chain(init_source_stream)
        .chain(init_sink_input_stream)
        .chain(init_source_output_stream)
        .chain(live_stream);

    let rt = Runtime::new().unwrap(); // TODO
    rt.enter(|| spawn(pulse_stream.map(|x| Ok(x)).forward(sender)));
    mainloop.borrow_mut().run().expect("Mainloop failed");
}

#[derive(Clone, Debug)]
struct RawPulseMessage {
    facility: Facility,
    operation: Operation,
    index: u32,
}

unsafe impl Send for RawPulseMessage {}
unsafe impl Sync for RawPulseMessage {}

#[derive(Clone, Debug)]
pub enum PulseMessage<'a> {
    MsgAdd { id: u32, msg: PulseAddMessage<'a> }, // add or update
    MsgDel { id: u32 },
}

#[derive(Clone, Debug)]
pub enum PulseAddMessage<'a> {
    MsgSink(SinkInfo<'a>),
    MsgSource(SourceInfo<'a>),
    MsgSourceOutput(SourceOutputInfo<'a>),
    MsgSinkInput(SinkInputInfo<'a>),
}