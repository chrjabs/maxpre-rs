//! # Interface for Preprocessing [`rustsat::instances::SatInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{ManageVars, SatInstance, CNF},
    types::constraints::{CardConstraint, PBConstraint},
};

use crate::PreproClauses;

pub trait PreproSat<VM: ManageVars>: PreproClauses {
    /// Initializes a new preprocessor from a [`SatInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<CardEnc, PBEnc>(
        inst: SatInstance<VM>,
        card_encoder: CardEnc,
        pb_encoder: PBEnc,
        inprocessing: bool,
    ) -> Self
    where
        CardEnc: FnMut(CardConstraint, &mut dyn ManageVars) -> CNF,
        PBEnc: FnMut(PBConstraint, &mut dyn ManageVars) -> CNF,
        Self: Sized,
    {
        let (cnf, _) = inst.as_cnf_with_encoders(card_encoder, pb_encoder);
        <Self as PreproClauses>::new(cnf, vec![], inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new(inst: SatInstance<VM>, inprocessing: bool) -> Self
    where
        Self: Sized,
    {
        Self::new_with_encoders(
            inst,
            card::default_encode_cardinality_constraint,
            pb::default_encode_pb_constraint,
            inprocessing,
        )
    }
    /// Gets the preprocessed instance as a [`SatInstance`]
    fn prepro_instance(&mut self) -> SatInstance<VM> {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        debug_assert!(objs.is_empty());
        SatInstance::from_iter(cnf)
    }
}

impl<PP: PreproClauses, VM: ManageVars> PreproSat<VM> for PP {}
