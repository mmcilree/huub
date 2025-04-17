pub enum ProofAction {
	Begin = 0,
	Comment = 1,
	Assert = 2,
	Conclude = 3,
}

#[macro_export]
macro_rules! proofbegin {
    ($($arg:tt)*) => {
        event!(target: "proof_log", Level::TRACE, action=huub_proofs::proof_logger::ProofAction::Begin as u8, $($arg)*);
    };
}

#[macro_export]
macro_rules! proofcomment {
    ($($arg:tt)*) => {
        event!(target: "proof_log", Level::TRACE, action=huub_proofs::proof_logger::ProofAction::Comment as u8, $($arg)*);
    };
}

#[macro_export]
macro_rules! proofassert {
    ($($arg:tt)*) => {
        event!(target: "proof_log", Level::TRACE, action=huub_proofs::proof_logger::ProofAction::Assert as u8, $($arg)*);
    };
}

#[macro_export]
macro_rules! proofconclude {
    ($($arg:tt)*) => {
        event!(target: "proof_log", Level::TRACE, action=huub_proofs::proof_logger::ProofAction::Conclude as u8, $($arg)*);
    };
}
