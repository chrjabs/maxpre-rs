//! # Interface for Preprocessing [`rustsat::instances::MultiOptInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{Cnf, ManageVars, MultiOptInstance, Objective, SatInstance},
    types::constraints::{CardConstraint, PBConstraint},
};

use crate::PreproClauses;

pub trait PreproMultiOpt: PreproClauses {
    /// Initializes a new preprocessor from a [`MultioptInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<VM, CardEnc, PBEnc>(
        inst: MultiOptInstance<VM>,
        card_encoder: CardEnc,
        pb_encoder: PBEnc,
        inprocessing: bool,
    ) -> Self
    where
        VM: ManageVars,
        CardEnc: FnMut(CardConstraint, &mut Cnf, &mut dyn ManageVars),
        PBEnc: FnMut(PBConstraint, &mut Cnf, &mut dyn ManageVars),
        Self: Sized,
    {
        let (constrs, objs) = inst.decompose();
        let (cnf, _) = constrs.into_cnf_with_encoders(card_encoder, pb_encoder);
        let softs: Vec<(_, isize)> = objs.into_iter().map(|o| o.into_soft_cls()).collect();
        <Self as PreproClauses>::new(cnf, softs, inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new<VM>(inst: MultiOptInstance<VM>, inprocessing: bool) -> Self
    where
        VM: ManageVars,
        Self: Sized,
    {
        Self::new_with_encoders(
            inst,
            |constr, cnf, vm| {
                card::default_encode_cardinality_constraint(constr, cnf, vm)
                    .expect("cardinality encoding ran out of memory")
            },
            |constr, cnf, vm| {
                pb::default_encode_pb_constraint(constr, cnf, vm)
                    .expect("pb encoding ran out of memory")
            },
            inprocessing,
        )
    }
    /// Gets the preprocessed instance as a [`SatInstance`]
    fn prepro_instance(&mut self) -> MultiOptInstance {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        let constrs = SatInstance::from(cnf);
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

impl<PP: PreproClauses> PreproMultiOpt for PP {}
