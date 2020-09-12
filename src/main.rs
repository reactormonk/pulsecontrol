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

use std::thread::sleep;
use druid::PlatformError;
use druid::Selector;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::stream::once;
use libpulse_binding::context::introspect::SourceInfo;
use pulse::context::Context;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use std::cell::RefCell;
use std::ops::Deref;
use std::{time, rc::Rc};
use tokio::runtime::Runtime;
use tokio::spawn;
use tokio::task::spawn_local;
use tokio::task::{self, JoinHandle};
// use pulse::mainloop::api::Mainloop as MainloopTrait; //Needs to be in scope
use druid::{widget::Label, ExtEventSink};
use druid::{AppLauncher, Widget, WindowDesc};
use futures::channel::{
    mpsc::{channel, SendError},
    oneshot,
};
use futures::stream::{empty, StreamExt};
use futuristic_pulse::*;
use pulse::context::subscribe::subscription_masks;
use pulse::context::{
    introspect::{SinkInfo, SinkInputInfo, SourceOutputInfo},
    subscribe::*,
};

use PulseMessage::*;

mod futuristic_pulse;

const PULSE_CHANGES: Selector<PulseMessage> = Selector::new("pulsecontrol.pulse-changes");

#[tokio::main]
async fn main() -> Result<(), PlatformError> {
    let launcher = AppLauncher::with_window(WindowDesc::new(build_ui));
    let event_sink = launcher.get_external_handle();
    // init_pulse().await;

    // pulse_stream.for_each(|pm| async move { eprintln!("Got message: {:?}", pm)}).await;

    let (send, mut recv): (Sender<PulseMessage>, Receiver<PulseMessage>) = channel(1024); // TODO channel size
    let (_one_send, one_recv) = oneshot::channel();

    eprintln!("Spawning...");
    std::thread::spawn(move || init_pulse(send, one_recv));
    eprintln!("Spawned.");

    while let Some(pm) = recv.next().await {
        eprintln!("Got Message: {:?}", pm);
        match event_sink.submit_command(PULSE_CHANGES, pm, None) {
            Err(err) => eprintln!("Error: {:?}", err),
            Ok(()) => (),
        };
    }

    eprintln!("Launch");
    launcher.launch(())?;
    Ok(())
}

fn build_ui() -> impl Widget<()> {
    Label::new("Hello world")
}

// async fn init_pulse<'a>() {
fn init_pulse<'a>(
    sender: Sender<PulseMessage<'static>>,
    mut wait_for: oneshot::Receiver<()>,
) -> () {
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
        Context::new_with_proplist(mainloop.borrow().deref(), "FooAppContext", &proplist)
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
    rt.enter(|| { spawn(pulse_stream.map(|x| Ok(x)).forward(sender))});
    eprintln!("Starting to loop...");
    while let Ok(None) = wait_for.try_recv() {
        let _ = &mainloop;
        sleep(time::Duration::from_millis(10));
    }
    eprintln!("And out.");
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
enum PulseMessage<'a> {
    MsgAdd { id: u32, msg: PulseAddMessage<'a> }, // add or update
    MsgDel { id: u32 },
}

#[derive(Clone, Debug)]
enum PulseAddMessage<'a> {
    MsgSink(SinkInfo<'a>),
    MsgSource(SourceInfo<'a>),
    MsgSourceOutput(SourceOutputInfo<'a>),
    MsgSinkInput(SinkInputInfo<'a>),
}
