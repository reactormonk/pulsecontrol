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

use futures::Stream;
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
use pulse::context::{introspect::SinkInfo, subscribe::*};
use futures::channel::mpsc::channel;
use futuristic_pulse::*;
use futures::{future::BoxFuture, stream::{empty, StreamExt, BoxStream}};

use PulseMessage::*;


mod futuristic_pulse;

fn main() {
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
    let (send, recv): (Sender<RawPulseMessage>, Receiver<RawPulseMessage>) = channel(1024); // TODO channel size

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

    let pulse_stream: Box<dyn Stream<Item=PulseMessage>> = StreamExt::flat_map(recv, {|raw|
        match (raw.facility, raw.operation) {
            (_, Removed) => BoxStream::new(once(async {MsgDel {id: Id(raw.index)}})),
            (Sink, _) => BoxStream::new(introspector.stream_sink_info_by_index(raw.index).map(|info| MsgAdd {id: Id(raw.index), msg: PulseAddMessage::MsgSink(info)})),
            (Source, _) => BoxStream::new(introspector.stream_source_info_by_index(raw.index).map(|info| MsgAdd {id: Id(raw.index), msg: PulseAddMessage::MsgSource(info)})),
            (SinkInput, _) => empty(),
            (SourceOutput, _) => empty(),
            (Module, _) => empty(),
            (Client, _) => empty(),
            (SampleCache, _) => empty(),
            (Server, _) => empty(),
            (Card, _) => empty(),
        }
    });

    mainloop.borrow_mut().unlock();

    loop {}

}

struct RawPulseMessage {
    facility: Facility,
    operation: Operation,
    index: u32
}

struct Id(u32);

enum PulseMessage<'a> {
    MsgAdd { id: Id, msg: PulseAddMessage<'a> }, // add or update
    MsgDel { id: Id }
}

enum PulseAddMessage<'a> {
    MsgSink(SinkInfo<'a>),
    MsgSource(SourceInfo<'a>),
}