use std::fmt::Result as FmtResult;

use time::{format_description::FormatItem, macros::format_description};
use tracing::{metadata::LevelFilter, Event, Subscriber};
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling,
};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, UtcTime},
        FmtContext, FormatEvent, FormatFields, Layer,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer as _,
};

pub fn init(quiet: bool) -> WorkerGuard {
    let formatter = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    let stdout_layer = Layer::default().event_format(StdoutEventFormat::new(formatter));

    let file_appender = rolling::daily("./logs", "osekai-scripts.log");
    let (file_writer, guard) = NonBlocking::new(file_appender);

    let file_layer = Layer::default()
        .event_format(FileEventFormat::new(formatter))
        .with_writer(file_writer);

    let stdout_filter = if quiet {
        EnvFilter::default()
    } else {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy()
    };

    let file_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(stdout_layer.with_filter(stdout_filter))
        .with(file_layer.with_filter(file_filter))
        .init();

    guard
}

struct StdoutEventFormat<'f> {
    timer: UtcTime<&'f [FormatItem<'f>]>,
}

impl<'f> StdoutEventFormat<'f> {
    fn new(formatter: &'f [FormatItem<'f>]) -> Self {
        Self {
            timer: UtcTime::new(formatter),
        }
    }
}

impl<S, N> FormatEvent<S, N> for StdoutEventFormat<'_>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> FmtResult {
        self.timer.format_time(&mut writer)?;
        let metadata = event.metadata();

        write!(writer, " {:>5} ", metadata.level(),)?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

struct FileEventFormat<'f> {
    timer: UtcTime<&'f [FormatItem<'f>]>,
}

impl<'f> FileEventFormat<'f> {
    fn new(formatter: &'f [FormatItem<'f>]) -> Self {
        Self {
            timer: UtcTime::new(formatter),
        }
    }
}

impl<S, N> FormatEvent<S, N> for FileEventFormat<'_>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> FmtResult {
        self.timer.format_time(&mut writer)?;
        let metadata = event.metadata();

        write!(
            writer,
            " {:>5} [{}:{}] ",
            metadata.level(),
            metadata.file().unwrap_or_else(|| metadata.target()),
            metadata.line().unwrap_or(0),
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
