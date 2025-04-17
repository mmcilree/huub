use std::io::Write;

use tracing::{
	field::{Field, Visit},
	Event, Level, Subscriber,
};

use tracing_subscriber::{
	field::{RecordFields, VisitOutput},
	filter,
	fmt::{
		format::{FormatEvent, FormatFields, Writer},
		FmtContext, MakeWriter,
	},
	registry::LookupSpan,
	Layer, Registry,
};
#[derive(Debug)]
pub struct PBProofLineFormatter;

#[derive(Default, Debug)]
pub struct PBProofFieldsFormatter;
struct PBClauseVisitor<'a> {
	writer: Writer<'a>,
	result: std::fmt::Result,
}

pub fn create_proof_layer<S, W>(proof_file: Option<W>) -> Option<impl Layer<S>>
where
	S: Subscriber + for<'a> LookupSpan<'a>,
	W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
	proof_file.map(|writer| {
		let base_layer = tracing_subscriber::fmt::layer()
			.with_writer(writer)
			.with_ansi(false)
			.event_format(PBProofLineFormatter)
			.fmt_fields(PBProofFieldsFormatter::default());

		let proof_filter = filter::Targets::new().with_target("proof_log", Level::TRACE);
		base_layer.with_filter(proof_filter)
	})
}

// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.FormatEvent.html
// Basically get rid of almost all of the formatting and just leave the actual fields
impl<S, N> FormatEvent<S, N> for PBProofLineFormatter
where
	S: Subscriber + for<'a> LookupSpan<'a>,
	N: for<'a> FormatFields<'a> + 'static,
{
	fn format_event(
		&self,
		context: &FmtContext<'_, S, N>,
		mut writer: Writer<'_>,
		event: &Event<'_>,
	) -> std::fmt::Result {
		write!(&mut writer, "a ")?;
		context
			.field_format()
			.format_fields(writer.by_ref(), event)?;
		writeln!(writer)
	}
}

impl<'writer> FormatFields<'writer> for PBProofFieldsFormatter {
	fn format_fields<R: RecordFields>(
		&self,
		writer: Writer<'writer>,
		fields: R,
	) -> std::fmt::Result {
		let mut v = PBClauseVisitor::new(writer);
		fields.record(&mut v);
		v.finish()
	}
}

impl<'a> PBClauseVisitor<'a> {
	fn new(writer: Writer<'a>) -> Self {
		PBClauseVisitor {
			writer,
			result: Ok(()),
		}
	}
}

impl Visit for PBClauseVisitor<'_> {
	#[inline]
	fn record_bool(&mut self, _: &Field, _: bool) {}
	#[inline]
	fn record_f64(&mut self, _: &Field, _: f64) {}
	#[inline]
	fn record_i64(&mut self, _: &Field, _: i64) {}
	#[inline]
	fn record_str(&mut self, _: &Field, _: &str) {}
	#[inline]
	fn record_u64(&mut self, _: &Field, _: u64) {}

	#[inline]
	fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
		if field.name().starts_with("clause") || field.name().starts_with("lits") {
			let res: Result<Vec<i32>, _> = serde_json::from_str(&format!("{:?}", value));
			if let Ok(clause) = res {
				let mut v: Vec<String> = Vec::with_capacity(clause.len());
				for i in clause {
					if i > 0 {
						v.push(format!("x{}", i));
					} else {
						v.push(format!("~x{}", i.abs()));
					}
				}

				let mut pb_con = v
					.iter()
					.flat_map(|var| ["1 ", var.as_str(), " "])
					.collect::<Vec<_>>()
					.join("");

				pb_con.push_str(">= 1 ;");

				self.result = write!(self.writer, "{pb_con}");
			}
		}
		return;
	}
}

impl VisitOutput<std::fmt::Result> for PBClauseVisitor<'_> {
	fn finish(self) -> std::fmt::Result {
		self.result
	}
}
