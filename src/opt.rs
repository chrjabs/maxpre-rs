//! # Interface for Preprocessing [`rustsat::instances::OptInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{ManageVars, Objective, OptInstance, SatInstance, CNF},
    types::constraints::{CardConstraint, PBConstraint},
};

use crate::PreproClauses;

pub trait PreproOpt<VM: ManageVars>: PreproClauses {
    /// Initializes a new preprocessor from a [`OptInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<CardEnc, PBEnc>(
        inst: OptInstance<VM>,
        card_encoder: CardEnc,
        pb_encoder: PBEnc,
        inprocessing: bool,
    ) -> Self
    where
        CardEnc: FnMut(CardConstraint, &mut dyn ManageVars) -> CNF,
        PBEnc: FnMut(PBConstraint, &mut dyn ManageVars) -> CNF,
        Self: Sized,
    {
        let (constrs, obj) = inst.decompose();
        let (cnf, _) = constrs.as_cnf_with_encoders(card_encoder, pb_encoder);
        let softs = obj.as_soft_cls();
        <Self as PreproClauses>::new(cnf, vec![softs], inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new(inst: OptInstance<VM>, inprocessing: bool) -> Self
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
    fn prepro_instance(&mut self) -> OptInstance<VM> {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        debug_assert_eq!(objs.len(), 1);
        let constrs = SatInstance::from_iter(cnf);
        let obj = if let Some((softs, offset)) = objs.into_iter().last() {
            let mut obj = Objective::from_iter(softs);
            obj.set_offset(offset);
            obj
        } else {
            panic!()
        };
        OptInstance::compose(constrs, obj)
    }
}

impl<PP: PreproClauses, VM: ManageVars> PreproOpt<VM> for PP {}
