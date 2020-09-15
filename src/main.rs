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

use druid::widget::Controller;
use druid::widget::ListIter;
use crate::futuristic_pulse::PulseAddMessage::*;
use druid::Color;
use druid::WidgetExt;
use druid::widget::List;
use druid::widget::Scroll;
use crate::futuristic_pulse::init_pulse;
use crate::futuristic_pulse::PulseMessage;
use druid::Selector;
use druid::{widget::{Flex, Label, CrossAxisAlignment}, ExtEventSink};
use druid::{AppLauncher, Widget, WindowDesc, Data, Lens, UnitPoint, lens::self, LensExt};
use futures::channel::mpsc::channel;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::stream::{StreamExt};
use tokio::spawn;
use im;
use pulse::context::introspect;
use core::ops;

mod futuristic_pulse;


const PULSE_CHANGES: Selector<PulseMessage> = Selector::new("pulsecontrol.pulse-changes");

#[tokio::main]
async fn main() -> () {
    let launcher = AppLauncher::with_window(WindowDesc::new(build_ui).title("PulseControl"));
    let event_sink = launcher.get_external_handle();

    // pulse_stream.for_each(|pm| async move { eprintln!("Got message: {:?}", pm)}).await;

    let (send, mut recv): (Sender<PulseMessage>, Receiver<PulseMessage>) = channel(1024); // TODO channel size

    std::thread::spawn(move || init_pulse(send));

    spawn(async move {
        while let Some(pm) = recv.next().await {
            match event_sink.submit_command(PULSE_CHANGES, pm, None) {
                Err(err) => eprintln!("Error: {:?}", err),
                Ok(()) => (),
            };
        }
    });

    eprintln!("Launch");
    launcher.launch(Default::default()).expect("launch failed");
}

fn build_ui() -> impl Widget<PulseState> {
    let mut root = Flex::column();
    let mut lists: Flex<PulseState> = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);

    lists.add_flex_child(
        Scroll::new(List::new(|| {
            Flex::row()
                .with_child(
                    Label::new(|item: &SinkInfo, _env: &_| {
                        format!("List item #{}", item.name.as_ref().map(|x| x as &str).unwrap_or("No Name :-("))
                    })
                    .align_vertical(UnitPoint::LEFT),
                )
                .with_flex_spacer(1.0)
                .padding(10.0)
                .background(Color::rgb(0.5, 0.0, 0.5))
                .fix_height(50.0)
        }))
        .vertical()
        .lens(lens::Id.map(
            |d: &PulseState| d.sinks.values().cloned().collect(),
            |d: &mut PulseState, x: im::Vector<SinkInfo>| {
                // If shared data was changed reflect the changes in our AppData
                () // TODO
            },
        )),
        1.0,
    );

    root.add_flex_child(lists, 1.0);
    root.controller(PulseCommunication)
}

#[derive(Clone, Lens, Default, Data, Debug)]
// #[data(same_fn="PartialEq::eq")]
struct PulseState {
    sinks: im::HashMap<u32, SinkInfo>,
    sources: im::HashMap<u32, SourceInfo>,
    sourceoutputs: im::HashMap<u32, SourceOutputInfo>,
    sinkinputs: im::HashMap<u32, SinkInputInfo>,
}

#[derive(Clone, Data, Debug)]
struct SinkInfo(
    #[data(same_fn="PartialEq::eq")]
    introspect::SinkInfo<'static>);
impl ops::Deref for SinkInfo {
    type Target = introspect::SinkInfo<'static>;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target { &self.0 }
}

#[derive(Clone, Data, Debug)]
struct SourceInfo(
    #[data(same_fn="PartialEq::eq")]
    introspect::SourceInfo<'static>);
impl ops::Deref for SourceInfo {
    type Target = introspect::SourceInfo<'static>;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target { &self.0 }
}

#[derive(Clone, Data, Debug)]
struct SourceOutputInfo(
    #[data(same_fn="PartialEq::eq")]
    introspect::SourceOutputInfo<'static>);
impl ops::Deref for SourceOutputInfo {
    type Target = introspect::SourceOutputInfo<'static>;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target { &self.0 }
}

#[derive(Clone, Data, Debug)]
struct SinkInputInfo(
    #[data(same_fn="PartialEq::eq")]
    introspect::SinkInputInfo<'static>);
impl ops::Deref for SinkInputInfo {
    type Target = introspect::SinkInputInfo<'static>;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target { &self.0 }
}

struct PulseCommunication;

impl PulseCommunication {
    fn new() -> Self {
        PulseCommunication
    }
}

impl<W: Widget<PulseState>> Controller<PulseState, W> for PulseCommunication {
    fn event(&mut self, _child: &mut W, _ctx: &mut druid::EventCtx<'_, '_>, event: &druid::Event, data: &mut PulseState, _env: &druid::Env) {
        match event {
            druid::Event::Command(cmd) if cmd.is(PULSE_CHANGES) => {
                eprintln!("{:?}", data);
                match cmd.get_unchecked(PULSE_CHANGES).clone() {
                    PulseMessage::MsgAdd{id, msg} => {
                        match msg {
                            MsgSink(s) => {data.sinks.insert(id, SinkInfo(s));},
                            MsgSource(s) => {data.sources.insert(id, SourceInfo(s));},
                            MsgSinkInput(s) => {data.sinkinputs.insert(id, SinkInputInfo(s));},
                            MsgSourceOutput(s) => {data.sourceoutputs.insert(id, SourceOutputInfo(s));},
                        }
                    },
                    PulseMessage::MsgDel{id} => {
                        data.sinks.remove(&id);
                        data.sources.remove(&id);
                        data.sourceoutputs.remove(&id);
                        data.sinkinputs.remove(&id);
                    }
                }
            }
            _ => (),
        }
    }
}