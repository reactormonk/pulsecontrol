use futures::channel::mpsc::channel;
use futures::channel::mpsc::Receiver;
use libpulse_binding::{
    callbacks::ListResult,
    context::introspect::{
        SinkInfo, SinkInputInfo, SourceInfo, SourceOutputInfo,
    },
};
use super::to_static::ToStatic;

pub fn callback_stream_sink_info() -> (
    impl FnMut(ListResult<&SinkInfo<'_>>),
    Receiver<SinkInfo<'static>>,
) {
    let (mut sender, recv) = channel(64); // TODO channel size?
    let cb = {
        move |c: ListResult<&SinkInfo<'_>>| {
            match c {
                ListResult::Item(it) => match sender.try_send(it.to_static()) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Failed to send message {:?}", err),
                },
                ListResult::End => sender.disconnect(),
                ListResult::Error => {
                    eprintln!("Got an error on the C callback.");
                    sender.disconnect()
                } // TODO
            }
        }
    };
    (cb, recv)
}

pub fn callback_stream_source_info() -> (
    impl FnMut(ListResult<&SourceInfo<'_>>),
    Receiver<SourceInfo<'static>>,
) {
    let (mut sender, recv) = channel(64); // TODO channel size?
    let cb = {
        move |c: ListResult<&SourceInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.to_static()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => {
                eprintln!("Got an error on the C callback.");
                sender.disconnect()
            } // TODO
        }
    };
    (cb, recv)
}

pub fn callback_stream_sink_input_info() -> (
    impl FnMut(ListResult<&SinkInputInfo<'_>>),
    Receiver<SinkInputInfo<'static>>,
) {
    let (mut sender, recv) = channel(64); // TODO channel size?
    let cb = {
        move |c: ListResult<&SinkInputInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.to_static()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => {
                eprintln!("Got an error on the C callback.");
                sender.disconnect()
            } // TODO
        }
    };
    (cb, recv)
}

pub fn callback_stream_source_output_info() -> (
    impl FnMut(ListResult<&SourceOutputInfo<'_>>),
    Receiver<SourceOutputInfo<'static>>,
) {
    let (mut sender, recv) = channel(64); // TODO channel size?
    let cb = {
        move |c: ListResult<&SourceOutputInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.to_static()) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to send message {:?}", err),
            },
            ListResult::End => sender.disconnect(),
            ListResult::Error => {
                eprintln!("Got an error on the C callback.");
                sender.disconnect()
            } // TODO
        }
    };
    (cb, recv)
}

// pub fn callback_list_stream<T: ToStatic >() -> (impl FnMut(ListResult<&T>), Receiver<<T as ToStatic>::Static>) {
//     let (mut sender, recv) = channel(1024); // TODO channel size?
//     let cb = {
//         move |c: ListResult<&T>| match c {
//             ListResult::Item(it) => match sender.try_send(it.to_static()) {
//                 Ok(_) => (),
//                 Err(err) => eprintln!("Failed to send message {:?}", err),
//             },
//             ListResult::End => sender.disconnect(),
//             ListResult::Error => (), // TODO
//         }
//     };
//     (cb, recv)
// }
