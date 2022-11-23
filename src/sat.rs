//! # Interface for Preprocessing [`rustsat::instances::SatInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{ManageVars, SatInstance, CNF},
    types::{
        constraints::{CardConstraint, PBConstraint},
        Clause,
    },
};

use crate::PreproClauses;

pub trait PreproSat: PreproClauses {
    /// Initializes a new preprocessor from a [`SatInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<VM, CardEnc, PBEnc>(
        inst: SatInstance<VM>,
        card_encoder: CardEnc,
        pb_encoder: PBEnc,
        inprocessing: bool,
    ) -> Self
    where
        VM: ManageVars,
        CardEnc: FnMut(CardConstraint, &mut dyn ManageVars) -> CNF,
        PBEnc: FnMut(PBConstraint, &mut dyn ManageVars) -> CNF,
        Self: Sized,
    {
        let (cnf, _) = inst.as_cnf_with_encoders(card_encoder, pb_encoder);
        <Self as PreproClauses>::new::<Vec<(Clause, usize)>>(cnf, vec![], inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new<VM>(inst: SatInstance<VM>, inprocessing: bool) -> Self
    where
        VM: ManageVars,
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
    fn prepro_instance(&mut self) -> SatInstance {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        debug_assert!(objs.is_empty());
        SatInstance::from_iter(cnf)
    }
}

impl<PP: PreproClauses> PreproSat for PP {}
