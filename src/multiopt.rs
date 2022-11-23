//! # Interface for Preprocessing [`rustsat::instances::MultiOptInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{ManageVars, MultiOptInstance, Objective, SatInstance, CNF},
    types::constraints::{CardConstraint, PBConstraint},
};

use crate::PreproClauses;

pub trait PreproMultiOpt<VM: ManageVars>: PreproClauses {
    /// Initializes a new preprocessor from a [`MultioptInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<CardEnc, PBEnc>(
        inst: MultiOptInstance<VM>,
        card_encoder: CardEnc,
        pb_encoder: PBEnc,
        inprocessing: bool,
    ) -> Self
    where
        CardEnc: FnMut(CardConstraint, &mut dyn ManageVars) -> CNF,
        PBEnc: FnMut(PBConstraint, &mut dyn ManageVars) -> CNF,
        Self: Sized,
    {
        let (constrs, objs) = inst.decompose();
        let (cnf, _) = constrs.as_cnf_with_encoders(card_encoder, pb_encoder);
        let softs = objs.into_iter().map(|o| o.as_soft_cls()).collect();
        <Self as PreproClauses>::new(cnf, softs, inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new(inst: MultiOptInstance<VM>, inprocessing: bool) -> Self
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
    fn prepro_instance(&mut self) -> MultiOptInstance<VM> {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        let constrs = SatInstance::from_iter(cnf);
        let objs = objs
            .into_iter()
            .map(|(softs, offset)| {
                let mut obj = Objective::from_iter(softs);
                obj.set_offset(offset);
                obj
            })
            .collect();
        MultiOptInstance::compose(constrs, objs)
    }
}

impl<PP: PreproClauses, VM: ManageVars> PreproMultiOpt<VM> for PP {}
