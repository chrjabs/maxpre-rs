//! # Interface for Preprocessing [`rustsat::instances::OptInstance`] types

use rustsat::{
    encodings::{card, pb},
    instances::{Cnf, ManageVars, Objective, OptInstance, SatInstance},
    types::constraints::{CardConstraint, PBConstraint},
};

use crate::PreproClauses;

pub trait PreproOpt: PreproClauses {
    /// Initializes a new preprocessor from a [`OptInstance`] where the instance
    /// is converted to [`CNF`] with the given encoders.
    fn new_with_encoders<VM, CardEnc, PBEnc>(
        inst: OptInstance<VM>,
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
        let (constrs, obj) = inst.decompose();
        let (cnf, _) = constrs.into_cnf_with_encoders(card_encoder, pb_encoder);
        let softs = obj.into_soft_cls();
        <Self as PreproClauses>::new(cnf, vec![softs], inprocessing)
    }
    /// Initializes a new preprocessor from a [`SatInstance`]
    fn new<VM>(inst: OptInstance<VM>, inprocessing: bool) -> Self
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
    fn prepro_instance(&mut self) -> OptInstance {
        let (cnf, objs) = <Self as PreproClauses>::prepro_instance(self);
        debug_assert_eq!(objs.len(), 1);
        let constrs = SatInstance::from(cnf);
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

impl<PP: PreproClauses> PreproOpt for PP {}
