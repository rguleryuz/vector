use codecs::{
    encoding::{Framer, FramingConfig},
    JsonSerializerConfig,
};
use futures::{future, FutureExt};
use tokio::io;
use vector_config::configurable_component;

use crate::{
    codecs::{Encoder, EncodingConfigWithFraming, SinkType},
    config::{AcknowledgementsConfig, GenerateConfig, Input, SinkConfig, SinkContext},
    sinks::{console::sink::WriterSink, Healthcheck, VectorSink},
};

/// Output target.
#[configurable_component]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// Standard output.
    #[derivative(Default)]
    Stdout,

    /// Standard error.
    Stderr,
}

/// Configuration for the `console` sink.
#[configurable_component(sink("console"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConsoleSinkConfig {
    #[configurable(derived)]
    #[serde(default)]
    pub target: Target,

    #[serde(flatten)]
    pub encoding: EncodingConfigWithFraming,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    pub acknowledgements: AcknowledgementsConfig,
}

impl GenerateConfig for ConsoleSinkConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            target: Target::Stdout,
            encoding: (None::<FramingConfig>, JsonSerializerConfig::new()).into(),
            acknowledgements: Default::default(),
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
impl SinkConfig for ConsoleSinkConfig {
    async fn build(&self, ctx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let sources_details = ctx.sources_details.clone();
        let transformer = self.encoding.transformer();
        let (framer, serializer) = self.encoding.build(SinkType::StreamBased)?;
        let encoder = Encoder::<Framer>::new(framer, serializer);

        let sink: VectorSink = match self.target {
            Target::Stdout => VectorSink::from_event_streamsink(WriterSink {
                output: io::stdout(),
                transformer,
                encoder,
                sources_details,
            }),
            Target::Stderr => VectorSink::from_event_streamsink(WriterSink {
                output: io::stderr(),
                transformer,
                encoder,
                sources_details,
            }),
        };

        Ok((sink, future::ok(()).boxed()))
    }

    fn input(&self) -> Input {
        Input::new(self.encoding.config().1.input_type())
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<ConsoleSinkConfig>();
    }
}
