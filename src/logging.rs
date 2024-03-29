use std::fmt::Result as FmtResult;

use time::{format_description::FormatItem, macros::format_description};
use tracing::{Event, Subscriber};
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
    EnvFilter, Layer as _, Registry,
};

pub fn init(quiet: bool) -> WorkerGuard {
    let formatter = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    let stdout_filter = if quiet {
        EnvFilter::default()
    } else {
        "osekai_scripts=info,error".parse().unwrap()
    };

    let stdout_layer = Layer::new()
        .event_format(StdoutEventFormat::new(formatter))
        .with_filter(stdout_filter);

    let file_appender = rolling::daily("./logs", "osekai-scripts.log");
    let (file_writer, guard) = NonBlocking::new(file_appender);

    let file_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "osekai_scripts=debug,info".parse().unwrap());

    let file_layer = Layer::new()
        .event_format(FileEventFormat::new(formatter))
        .with_writer(file_writer)
        .with_filter(file_filter);

    Registry::default()
        .with(stdout_layer)
        .with(file_layer)
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
