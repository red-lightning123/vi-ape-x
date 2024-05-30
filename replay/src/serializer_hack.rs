use packets::SampleBatchErrorKind;
use replay_data::CompressedTransition;
use serde::Serialize;

// This type is a hack. It is almost identical to SampleBatchReply, except for
// taking the transitions as references. Serde derefs these transitions when
// serializing, so the serialized result can be deserialized as a normal
// SampleBatchReply. The replay server uses this type during batch sending to
// avoid unnecessary clones of sent transitions
#[derive(Serialize)]
pub struct SampleBatchReplySerializer<'a> {
    pub batch: (Vec<usize>, Vec<f64>, Vec<&'a CompressedTransition>),
    pub min_probability: f64,
    pub replay_len: usize,
}

// Similarly, this type is almost identical to SampleBatchResult, except for the
// caveat noted above
pub type SampleBatchResultSerializer<'a> =
    Result<SampleBatchReplySerializer<'a>, SampleBatchErrorKind>;
