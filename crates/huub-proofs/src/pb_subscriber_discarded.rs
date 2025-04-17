use tracing::{
	field::{Field, Visit},
	Event, Level, Subscriber,
};

use std::fmt;

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

use crate::proof_logger::ProofAction;

#[derive(Default, Debug)]
struct ProofActionVisitor {
	action: ProofAction,
}
#[derive(Default, Debug)]
pub struct PBProofLineFormatter;
#[derive(Default, Debug)]
pub struct PBProofFieldsFormatter;
struct PBClauseVisitor<'a> {
	writer: Writer<'a>,
	result: std::fmt::Result,
}

pub fn create_proof_layer<S, W>(proof_file: &Option<W>) -> Option<impl Layer<S>>
where
	S: Subscriber + for<'a> LookupSpan<'a>,
	W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
	proof_file.as_ref().map(|writer| {
		let base_layer = tracing_subscriber::fmt::layer()
			.with_writer(writer)
			.with_ansi(false)
			.event_format(PBProofLineFormatter)
			.fmt_fields(PBProofFieldsFormatter::default());

		let proof_filter = filter::Targets::new().with_target("proof_log", Level::TRACE);
		base_layer.with_filter(proof_filter)
	})
}

impl Visit for ProofActionVisitor {
	fn record_u64(&mut self, field: &Field, value: u64) {
		if field.name() == "action" {
			self.action = ProofAction::from(value);
		}
	}

	fn record_debug(&mut self, _field: &Field, _value: &dyn fmt::Debug) {
		// do nothing
	}
}

impl VisitOutput<ProofAction> for ProofActionVisitor {
	fn finish(self) -> ProofAction {
		return self.action;
	}
}
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
	) -> fmt::Result {
		let mut determine_proof_action = ProofActionVisitor::default();
		event.record(&mut determine_proof_action);
		let proof_action = determine_proof_action.finish();

		match proof_action {
			ProofAction::Begin => {
				dbg!("HERE");
				writeln!(writer, "begin pseudo-Boolean proof version 3.0")
			}
			ProofAction::Comment => {
				writeln!(writer, "* Will write comments like this")
			}
			ProofAction::Assert => {
				writeln!(writer, "a Will write assertions like this")
			}
			ProofAction::Conclude => {
				// TODO handle proper conclusions
				writeln!(writer, "output NONE")?;
				writeln!(writer, "conclusion NONE")?;
				writeln!(writer, "end pseudo-Boolean proof")
			}
			_ => {
				writeln!(
					writer,
					"******[pb_subscriber: Unrecognised proof action, skipping]"
				)
			}
		}
		// context
		// 	.field_format()
		// 	.format_fields(writer.by_ref(), event)?;
		//writeln!(writer)
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
