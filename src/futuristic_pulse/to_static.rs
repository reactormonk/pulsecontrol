use libpulse_binding::format::Info;
use libpulse_binding::volume::Volume;
use libpulse_binding::time::MicroSeconds;
use libpulse_binding::channelmap::Map;
use libpulse_binding::{
    context::introspect::{
        SinkInfo, SinkInputInfo, SinkPortInfo, SourceInfo, SourceOutputInfo, SourcePortInfo,
    }, sample::Spec, volume::ChannelVolumes, proplist::Proplist, def::{SinkState, SourceState},
};
use std::{borrow::Cow, boxed::Box};

pub trait ToStatic {
    type Static;
    fn to_static(&self) -> Self::Static;
}

impl<'a> ToStatic for SourcePortInfo<'a> {
    type Static = SourcePortInfo<'static>;

    fn to_static(&self) -> Self::Static {
        SourcePortInfo {
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            description: self.description.as_ref().map(|s| Cow::Owned(s.to_string())),
            priority: self.priority,
            available: self.available,
        }
    }
}

impl<'a> ToStatic for SinkPortInfo<'a> {
    type Static = SinkPortInfo<'static>;

    fn to_static(&self) -> Self::Static {
        SinkPortInfo {
            name: self.name.as_ref().map(|s| Cow::Owned(s.to_string())),
            description: self.description.as_ref().map(|s| Cow::Owned(s.to_string())),
            priority: self.priority,
            available: self.available,
        }
    }
}

impl<'a> ToStatic for Cow<'a, str> {
    type Static = Cow<'static, str>;

    fn to_static(&self) -> <std::borrow::Cow<'a, str> as ToStatic>::Static {
        Cow::Owned(self.to_string())
    }
}

impl<T: ToStatic> ToStatic for Option<T> {
    type Static = Option<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.as_ref().map(|s| s.to_static())
    }
}

impl<T: ToStatic> ToStatic for Box<T> {
    type Static = Box<T::Static>;

    fn to_static(&self) -> Self::Static {
        Box::new((*self.as_ref()).to_static())
    }
}

impl<T: ToStatic> ToStatic for Vec<T> {
    type Static = Vec<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.iter().map(|s| s.to_static()).collect()
    }
}

// impl<T> ToStatic for T where T:Clone {
//     type Static = T;

//     fn to_static(&self) -> Self::Static {
//         *self.clone()
//     }
// }

impl ToStatic for u32 {
    type Static = u32;
    fn to_static(&self) -> Self::Static {
        *self
    }
}

impl ToStatic for bool {
    type Static = bool;
    fn to_static(&self) -> Self::Static {
        *self
    }
}

impl ToStatic for Spec {
    type Static = Spec;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for Map {
    type Static = Map;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for ChannelVolumes {
    type Static = ChannelVolumes;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for MicroSeconds {
    type Static = MicroSeconds;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for Proplist {
    type Static = Proplist;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for Volume {
    type Static = Volume;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for SinkState {
    type Static = SinkState;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for SourceState {
    type Static = SourceState;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl ToStatic for Info {
    type Static = Info;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

impl<'a> ToStatic for SinkInfo<'a> {
    type Static = SinkInfo<'static>;
    fn to_static(&self) -> Self::Static {
        SinkInfo {
            name: self.name.to_static(),
            index: self.index.to_static(),
            description: self.description.to_static(),
            sample_spec: self.sample_spec.to_static(),
            channel_map: self.channel_map.to_static(),
            owner_module: self.owner_module.to_static(),
            volume: self.volume.to_static(),
            mute: self.mute.to_static(),
            monitor_source: self.monitor_source.to_static(),
            monitor_source_name: self
                .monitor_source_name
                .to_static(),
            latency: self.latency.to_static(),
            driver: self.driver.to_static(),
            flags: self.flags.to_static(),
            proplist: self.proplist.to_static(),
            configured_latency: self.configured_latency.to_static(),
            base_volume: self.base_volume.to_static(),
            state: self.state.to_static(),
            n_volume_steps: self.n_volume_steps.to_static(),
            card: self.card.to_static(),
            ports: self.ports.to_static(),
            active_port: self
                .active_port
                .to_static(),
            formats: self.formats.to_static(),
        }
    }
}

impl<'a> ToStatic for SourceInfo<'a> {
    type Static = SourceInfo<'static>;
    fn to_static(&self) -> Self::Static {
        SourceInfo {
            name: self.name.to_static(),
            index: self.index.to_static(),
            description: self.description.to_static(),
            sample_spec: self.sample_spec.to_static(),
            channel_map: self.channel_map.to_static(),
            owner_module: self.owner_module.to_static(),
            volume: self.volume.to_static(),
            mute: self.mute.to_static(),
            latency: self.latency.to_static(),
            driver: self.driver.to_static(),
            flags: self.flags.to_static(),
            proplist: self.proplist.to_static(),
            configured_latency: self.configured_latency.to_static(),
            base_volume: self.base_volume.to_static(),
            state: self.state.to_static(),
            n_volume_steps: self.n_volume_steps.to_static(),
            card: self.card.to_static(),
            ports: self.ports.to_static(),
            active_port: self.active_port.to_static(),
            formats: self.formats.to_static(),
            monitor_of_sink: self.monitor_of_sink.to_static(),
            monitor_of_sink_name: self.monitor_of_sink_name.to_static()
        }
    }
}

impl<'a> ToStatic for SourceOutputInfo<'a> {
    type Static = SourceOutputInfo<'static>;
    fn to_static(&self) -> Self::Static {
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
            resample_method: self
                .resample_method
                .as_ref()
                .map(|s| Cow::Owned(s.to_string())),
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

impl<'a> ToStatic for SinkInputInfo<'a> {
    type Static = SinkInputInfo<'static>;
    fn to_static(&self) -> Self::Static {
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
            resample_method: self
                .resample_method
                .as_ref()
                .map(|s| Cow::Owned(s.to_string())),
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
