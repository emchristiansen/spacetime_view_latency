//! Tests pinning the whole-cell control-validity gate: only a control interval within the frozen band
//! yields a `Valid` cell carrying the arm response and prediction comparison; otherwise `InvalidControl`
//! exposes no arm claim.

mod gates_the_arm_claim_on_control_validity;
