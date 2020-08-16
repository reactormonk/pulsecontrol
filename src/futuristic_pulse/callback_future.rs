use futures::channel::mpsc::Receiver;
use futures::{channel::mpsc::channel, Stream};
use libpulse_binding::{
    callbacks::ListResult,
    context::introspect::{SinkInfo, SinkPortInfo, SourceInfo, SourcePortInfo},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::{borrow::Cow, boxed::Box};

pub struct WakerSet {
    waker: Option<Waker>,
}

impl WakerSet {
    pub fn new() -> Self {
        Self { waker: None }
    }

    pub fn register(&mut self, w: &Waker) {
        match self.waker {
            Some(ref w) if (w.will_wake(w)) => {}
            Some(_) => panic!("Waker overflow"),
            None => self.waker = Some(w.clone()),
        }
    }

    pub fn wake(&mut self) {
        self.waker.take().map(|w| w.wake());
    }
}

enum CallbackFutureState<R> {
    Running(WakerSet),
    Done(Option<R>),
}

pub struct CallbackFuture<R> {
    state: Arc<Mutex<CallbackFutureState<R>>>,
}

impl<R: 'static> CallbackFuture<R> {
    pub fn new() -> (Self, impl FnMut(R) + 'static) {
        let state = Arc::new(Mutex::new(CallbackFutureState::Running(WakerSet::new())));

        let fut = Self {
            state: state.clone(),
        };

        let callback = move |r| {
            *state.lock().unwrap() = CallbackFutureState::Done(Some(r));
        };

        (fut, callback)
    }
}

impl<R> Future for CallbackFuture<R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut *self.state.lock().unwrap() {
            CallbackFutureState::Running(w) => {
                w.register(cx.waker());
                Poll::Pending
            }
            CallbackFutureState::Done(r) => Poll::Ready(r.take().unwrap()),
        }
    }
}

trait MakeOwned {
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

pub fn callback_stream_sink_info() -> (
    impl FnMut(ListResult<&SinkInfo<'_>>),
    Receiver<SinkInfo<'static>>,
) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&SinkInfo<'_>>| match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => (), // TODO
            },
            ListResult::End => (),
            ListResult::Error => (), // TODO
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
                Err(err) => (), // TODO
            },
            ListResult::End => (),
            ListResult::Error => (), // TODO
        }
    };
    (cb, recv)
}

// pub fn foo() -> (
//     impl FnMut(ListResult<&SourceInfo<'_>>),
//     Receiver<SourceInfo<'static>>,
// ) { callback_list_stream() }

pub fn callback_list_stream<T: MakeOwned >() -> (impl FnMut(ListResult<&T>), Receiver<<T as MakeOwned>::Owned>) {
    let (mut sender, recv) = channel(1024); // TODO channel size?
    let cb = {
        move |c: ListResult<&T>| match c {
            ListResult::Item(it) => match sender.try_send(it.make_owned()) {
                Ok(_) => (),
                Err(err) => (), // TODO
            },
            ListResult::End => (),
            ListResult::Error => (), // TODO
        }
    };
    (cb, recv)
}

async fn test() {
    let (mut fut, mut callback) = CallbackFuture::<i32>::new();

    let fut = Pin::new(&mut fut);

    // on one thread
    let _res = fut.await; // will block until callback is called

    // on other thread, or from C code
    callback(123);
}
