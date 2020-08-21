// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate libpulse_binding as pulse;

use futures::stream::once;
use futures::channel::mpsc::Sender;
use futures::channel::mpsc::Receiver;
use libpulse_binding::context::introspect::SourceInfo;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use pulse::mainloop::threaded::Mainloop;
use pulse::context::Context;
use pulse::proplist::Proplist;
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope
use pulse::context::subscribe::subscription_masks;
use pulse::context::{introspect::{SourceOutputInfo, SinkInfo, SinkInputInfo}, subscribe::*};
use futures::channel::mpsc::channel;
use futuristic_pulse::*;
use futures::stream::{empty, StreamExt};

use PulseMessage::*;


mod futuristic_pulse;


#[tokio::main]
async fn main() {
    let spec = pulse::sample::Spec {
        format: pulse::sample::SAMPLE_S16NE,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    let mut proplist = Proplist::new().unwrap();
    proplist.set_str(pulse::proplist::properties::APPLICATION_NAME, "FooApp")
        .unwrap();

    let mainloop = Rc::new(RefCell::new(Mainloop::new()
        .expect("Failed to create mainloop")));

    let context = Rc::new(RefCell::new(Context::new_with_proplist(
        mainloop.borrow().deref(),
        "FooAppContext",
        &proplist
        ).expect("Failed to create new context")));

    // Context state change callback
    {
        let ml_ref = Rc::clone(&mainloop);
        let context_ref = Rc::clone(&context);
        context.borrow_mut().set_state_callback(Some(Box::new(move || {
            let state = unsafe { (*context_ref.as_ptr()).get_state() };
            match state {
                pulse::context::State::Ready |
                pulse::context::State::Failed |
                pulse::context::State::Terminated => {
                    unsafe { (*ml_ref.as_ptr()).signal(false); }
                },
                _ => {},
            }
        })));
    }

    context.borrow_mut().connect(None, pulse::context::flags::NOFLAGS, None)
        .expect("Failed to connect context");

    mainloop.borrow_mut().lock();
    mainloop.borrow_mut().start().expect("Failed to start mainloop");

    // Wait for context to be ready
    loop {
        match context.borrow().get_state() {
            pulse::context::State::Ready => { break; },
            pulse::context::State::Failed |
            pulse::context::State::Terminated => {
                eprintln!("Context state failed/terminated, quitting...");
                mainloop.borrow_mut().unlock();
                mainloop.borrow_mut().stop();
                return;
            },
            _ => { mainloop.borrow_mut().wait(); },
        }
    }
    context.borrow_mut().set_state_callback(None);

    let interest = subscription_masks::ALL;

    // recv implements Stream
    let (mut send, recv): (Sender<RawPulseMessage>, Receiver<RawPulseMessage>) = channel(1024); // TODO channel size

    context.borrow_mut().set_subscribe_callback(Some(Box::new(move |fac, op, i| {
        match (fac, op, i) {
            (Some(facility), Some(operation), index) =>
                match send.try_send(RawPulseMessage{facility, operation, index}) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Got error {}", err)
                },
            _ => eprintln!("Got weird Message: {:?} | {:?} | {:?}", fac, op, i),
        }
    })));
    eprintln!("Set callback.");

    let introspector = context.borrow_mut().introspect();

    let _op = context.borrow_mut().subscribe(
        interest,   // Our interest mask
        |args| {
            eprintln!("Subscribed. {}", args);
        }
    );

    let init_sink_stream = introspector.stream_info_list().map(move |info:SinkInfo| MsgAdd {id: info.index, msg: PulseAddMessage::MsgSink(info)}).boxed();
    let init_source_stream = introspector.stream_info_list().map(move |info:SourceInfo| MsgAdd {id: info.index, msg: PulseAddMessage::MsgSource(info)}).boxed();
    let init_sink_input_stream = introspector.stream_info_list().map(move |info:SinkInputInfo| MsgAdd {id: info.index, msg: PulseAddMessage::MsgSinkInput(info)}).boxed();
    let init_source_output_stream = introspector.stream_info_list().map(move |info:SourceOutputInfo| MsgAdd {id: info.index, msg: PulseAddMessage::MsgSourceOutput(info)}).boxed();

    let live_stream = recv.boxed_local().flat_map({|raw|
        match (raw.facility, raw.operation) {
            (_, Operation::Removed) => once(async move {MsgDel {id: raw.index}}).boxed(),
            (Facility::Sink, _) => introspector.stream_info_by_index(raw.index).map(move |info| MsgAdd {id: raw.index, msg: PulseAddMessage::MsgSink(info)}).boxed(),
            (Facility::Source, _) => introspector.stream_info_by_index(raw.index).map(move |info| MsgAdd {id: raw.index, msg: PulseAddMessage::MsgSource(info)}).boxed(),
            (Facility::SinkInput, _) => introspector.stream_info_by_index(raw.index).map(move |info| MsgAdd {id: raw.index, msg: PulseAddMessage::MsgSinkInput(info)}).boxed(),
            (Facility::SourceOutput, _) => introspector.stream_info_by_index(raw.index).map(move |info| MsgAdd {id: raw.index, msg: PulseAddMessage::MsgSourceOutput(info)}).boxed(),
            (Facility::Module, _) => empty().boxed(),
            (Facility::Client, _) => empty().boxed(),
            (Facility::SampleCache, _) => empty().boxed(),
            (Facility::Server, _) => empty().boxed(),
            (Facility::Card, _) => empty().boxed(),
        }
    });

    let pulse_stream = init_sink_stream.chain(init_source_stream).chain(init_sink_input_stream).chain(init_source_output_stream).chain(live_stream);

    mainloop.borrow_mut().unlock();

    pulse_stream.for_each(|pm| async move { eprintln!("Got Message: {:?}", pm)}).await;

    loop {}

}

#[derive(Clone, Debug)]
struct RawPulseMessage {
    facility: Facility,
    operation: Operation,
    index: u32
}

#[derive(Clone, Debug)]
enum PulseMessage<'a> {
    MsgAdd { id: u32, msg: PulseAddMessage<'a> }, // add or update
    MsgDel { id: u32 }
}

#[derive(Clone, Debug)]
enum PulseAddMessage<'a> {
    MsgSink(SinkInfo<'a>),
    MsgSource(SourceInfo<'a>),
    MsgSourceOutput(SourceOutputInfo<'a>),
    MsgSinkInput(SinkInputInfo<'a>),
}