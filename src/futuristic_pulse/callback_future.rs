use futures::channel::mpsc::Receiver;
use futures::{channel::mpsc::channel};
use libpulse_binding::{
    callbacks::ListResult,
    context::introspect::{SinkInfo, SinkPortInfo, SourceInfo, SourcePortInfo, SinkInputInfo, SourceOutputInfo},
};
use std::{borrow::Cow, boxed::Box};


pub trait MakeOwned {
    type Owned;
    fn make_owned(&self) -> Self::Owned;
}

impl<'a> MakeOwned for SourcePortInfo<'a> {
    type Owned = SourcePortInfo<'static>;

    fn make_owned(&self) -> Self::Owned {
        SourcePortInfo {
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            description: self.description.as_ref().map(|s| Cow::Owned(s.to_string())),
            priority: self.priority,
            available: self.available,
        }
    }
}

impl<'a> MakeOwned for SinkPortInfo<'a> {
    type Owned = SinkPortInfo<'static>;

    fn make_owned(&self) -> Self::Owned {
        SinkPortInfo {
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            description: self.description.as_ref().map(|s| Cow::Owned(s.to_string())),
            priority: self.priority,
            available: self.available,
        }
    }
}

impl<'a> MakeOwned for SinkInfo<'a> {
    type Owned = SinkInfo<'static>;
fn make_owned(&self) -> Self::Owned {
    SinkInfo {
        name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
        index: self.index.clone(),
        description: self
            .description
            .as_ref()
            .map(|s| Cow::Owned(s.to_string())),
        sample_spec: self.sample_spec.clone(),
        channel_map: self.channel_map.clone(),
        owner_module: self.owner_module.clone(),
        volume: self.volume.clone(),
        mute: self.mute.clone(),
        monitor_source: self.monitor_source.clone(),
        monitor_source_name: self
            .monitor_source_name
            .as_ref()
            .map(|s| Cow::Owned(s.to_string())),
        latency: self.latency.clone(),
        driver: self.driver.as_ref().map(|s| Cow::Owned(s.to_string())),
        flags: self.flags.clone(),
        proplist: self.proplist.clone(),
        configured_latency: self.configured_latency.clone(),
        base_volume: self.base_volume.clone(),
        state: self.state.clone(),
        n_volume_steps: self.n_volume_steps.clone(),
        card: self.card.clone(),
        ports: self
            .ports
            .iter()
            .map(|x| (&*x).make_owned())
            .collect(),
        active_port: self
            .active_port
            .as_ref()
            .map(|x| Box::new((&*x).make_owned())),
        formats: self.formats.clone(),
    }
 }
}

impl<'a> MakeOwned for SourceInfo<'a> {
    type Owned = SourceInfo<'static>;
fn make_owned(&self) -> Self::Owned {
    SourceInfo {
        name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
        index: self.index.clone(),
        description: self
            .description
            .as_ref()
            .map(|s| Cow::Owned(s.to_string())),
        sample_spec: self.sample_spec.clone(),
        channel_map: self.channel_map.clone(),
        owner_module: self.owner_module.clone(),
        volume: self.volume.clone(),
        mute: self.mute.clone(),
        latency: self.latency.clone(),
        driver: self.driver.as_ref().map(|s| Cow::Owned(s.to_string())),
        flags: self.flags.clone(),
        proplist: self.proplist.clone(),
        configured_latency: self.configured_latency.clone(),
        base_volume: self.base_volume.clone(),
        state: self.state.clone(),
        n_volume_steps: self.n_volume_steps.clone(),
        card: self.card.clone(),
        ports: self
            .ports
            .iter()
            .map(|x| x.make_owned())
            .collect(),
        active_port: self
            .active_port
            .as_ref()
            .map(|x| Box::new((&*x).make_owned())),
        formats: self.formats.clone(),
        monitor_of_sink: self.monitor_of_sink.clone(),
        monitor_of_sink_name: self
            .monitor_of_sink_name
            .as_ref()
            .map(|s| Cow::Owned(s.to_string())),
    }
}
}

impl<'a> MakeOwned for SourceOutputInfo<'a> {
    type Owned = SourceOutputInfo<'static>;
    fn make_owned(&self) -> Self::Owned {
        SourceOutputInfo {
            index: self.index.clone(),
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            owner_module: self.owner_module.clone(),
            client: self.client.clone(),
            source: self.source.clone(),
            sample_spec: self.sample_spec.clone(),
            channel_map: self.channel_map.clone(),
            buffer_usec: self.buffer_usec.clone(),
            source_usec: self.source_usec.clone(),
            resample_method: self.resample_method.as_ref().map(|s| Cow::Owned(s.to_string())),
            driver: self.driver.as_ref().map(|s| Cow::Owned(s.to_string())),
            proplist: self.proplist.clone(),
            corked: self.corked.clone(),
            volume: self.volume.clone(),
            mute: self.mute.clone(),
            has_volume: self.has_volume.clone(),
            volume_writable: self.volume_writable.clone(),
            format: self.format.clone(),
        }
    }
}

impl<'a> MakeOwned for SinkInputInfo<'a> {
    type Owned = SinkInputInfo<'static>;
    fn make_owned(&self) -> Self::Owned {
        SinkInputInfo {
            index: self.index.clone(),
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            owner_module: self.owner_module.clone(),
            client: self.client.clone(),
            sink: self.sink.clone(),
            sample_spec: self.sample_spec.clone(),
            channel_map: self.channel_map.clone(),
            volume: self.volume.clone(),
            buffer_usec: self.buffer_usec.clone(),
            sink_usec: self.sink_usec.clone(),
            resample_method: self.resample_method.as_ref().map(|s| Cow::Owned(s.to_string())),
            driver: self.driver.as_ref().map(|s| Cow::Owned(s.to_string())),
            mute: self.mute.clone(),
            proplist: self.proplist.clone(),
            corked: self.corked.clone(),
            has_volume: self.has_volume.clone(),
            volume_writable: self.volume_writable.clone(),
            format: self.format.clone(),
        }
    }
}


pub fn callback_stream_sink_info() -> (
    impl FnMut(ListResult<&SinkInfo<'_>>),
    Receiver<SinkInfo<'static>>,
) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&SinkInfo<'_>>| {
        match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => { eprintln!("Got an error on the C callback."); sender.disconnect()}, // TODO
        }
        }
    };
    (cb, recv)
}

pub fn callback_stream_source_info() -> (
    impl FnMut(ListResult<&SourceInfo<'_>>),
    Receiver<SourceInfo<'static>>,
) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&SourceInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => { eprintln!("Got an error on the C callback."); sender.disconnect()}, // TODO
        }
    };
    (cb, recv)
}

pub fn callback_stream_sink_input_info() -> (
    impl FnMut(ListResult<&SinkInputInfo<'_>>),
    Receiver<SinkInputInfo<'static>>,
) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&SinkInputInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => { eprintln!("Got an error on the C callback."); sender.disconnect()}, // TODO
        }
    };
    (cb, recv)
}

pub fn callback_stream_source_output_info() -> (
    impl FnMut(ListResult<&SourceOutputInfo<'_>>),
    Receiver<SourceOutputInfo<'static>>,
) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&SourceOutputInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => { eprintln!("Got an error on the C callback."); sender.disconnect()}, // TODO
        }
    };
    (cb, recv)
}

// pub fn callback_list_stream<T: MakeOwned >() -> (impl FnMut(ListResult<&T>), Receiver<<T as MakeOwned>::Owned>) {
//     let (mut sender, recv) = channel(1024); // TODO channel size?
//     let cb = {
//         move |c: ListResult<&T>| match c {
//             ListResult::Item(it) => match sender.try_send(it.make_owned()) {
//                 Ok(_) => (),
//                 Err(err) => eprintln!("Failed to send message {:?}", err),
//             },
//             ListResult::End => sender.disconnect(),
//             ListResult::Error => (), // TODO
//         }
//     };
//     (cb, recv)
// }