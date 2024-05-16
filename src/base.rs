//! # Base MaxPre Interface
//!
//! The low-abstraction MaxPre interface working on hard and soft clauses.

use core::ffi::{c_int, c_uint, CStr};
use std::ffi::CString;

use cpu_time::ProcessTime;
use rustsat::{
    instances::Cnf,
    types::{Assignment, Clause, Lit, Var, WClsIter},
};

use crate::Error;

use super::{ffi, Options, PreproClauses, Stats};

/// The main low-abstraction preprocessor type
pub struct MaxPre {
    /// The handle for the C API
    handle: *mut ffi::CMaxPre,
    /// Offsets of the objectives
    offsets: Vec<isize>,
    /// Statistics of the preprocessor
    stats: Stats,
}

impl PreproClauses for MaxPre {
    fn signature() -> &'static str {
        let c_chars = unsafe { ffi::cmaxpre_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("MaxPre signature returned invalid UTF-8")
    }

    fn new<CI: WClsIter>(hards: Cnf, softs: Vec<(CI, isize)>, inprocessing: bool) -> Self {
        let mut top = 1;
        let softs: Vec<(Vec<(Clause, usize)>, isize)> = softs
            .into_iter()
            .map(|(cls, ofs)| (cls.into_iter().collect(), ofs))
            .collect();
        top = softs.iter().fold(top, |top, softs| {
            softs.0.iter().fold(top, |top, (_, w)| top + w)
        });
        let mut stats = Stats {
            n_objs: softs.len(),
            n_orig_hard_clauses: hards.len(),
            ..Default::default()
        };
        let handle = unsafe { ffi::cmaxpre_init_start(top as u64, ffi::map_bool(inprocessing)) };
        hards.into_iter().for_each(|cl| {
            cl.into_iter().for_each(|l| {
                stats.max_orig_var = Self::track_max_var(stats.max_orig_var, l.var());
                unsafe { ffi::cmaxpre_init_add_lit(handle, l.to_ipasir()) }
            });
            unsafe { ffi::cmaxpre_init_add_lit(handle, 0) };
        });
        let mut offsets = Vec::new();
        softs.into_iter().enumerate().for_each(|(idx, softs)| {
            offsets.push(softs.1);
            stats.n_orig_soft_clauses.push(softs.0.len());
            softs.0.into_iter().for_each(|(cl, w)| {
                // Add zero weight for all previous objectives
                (0..idx).for_each(|_| unsafe { ffi::cmaxpre_init_add_weight(handle, 0) });
                // Add weight for the objective with index
                unsafe { ffi::cmaxpre_init_add_weight(handle, w as u64) };
                // Add literals
                cl.into_iter().for_each(|l| {
                    stats.max_orig_var = Self::track_max_var(stats.max_orig_var, l.var());
                    unsafe { ffi::cmaxpre_init_add_lit(handle, l.to_ipasir()) }
                });
                unsafe { ffi::cmaxpre_init_add_lit(handle, 0) };
            })
        });
        unsafe { ffi::cmaxpre_init_finalize(handle) };
        Self {
            handle,
            offsets,
            stats,
        }
    }

    fn preprocess(&mut self, techniques: &str, log_level: c_int, time_limit: f64) {
        let start = ProcessTime::now();
        let techniques = CString::new(techniques).unwrap();
        unsafe {
            ffi::cmaxpre_preprocess(
                self.handle,
                techniques.as_ptr(),
                log_level,
                time_limit,
                ffi::FALSE,
                ffi::FALSE,
            )
        };
        self.stats.prepro_time += start.elapsed();
    }

    fn top_weight(&self) -> u64 {
        unsafe { ffi::cmaxpre_get_top_weight(self.handle) }
    }

    fn n_prepro_clauses(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_clauses(self.handle) }
    }

    fn n_prepro_labels(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_labels(self.handle) }
    }

    fn n_prepro_fixed_lits(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_fixed(self.handle) }
    }

    fn prepro_instance(&mut self) -> (Cnf, Vec<(Vec<(Clause, usize)>, isize)>) {
        let n_cls = self.n_prepro_clauses();
        let top = self.top_weight();
        let mut hards = Cnf::new();
        let mut softs: Vec<Vec<(Clause, usize)>> = vec![Default::default(); self.stats.n_objs];
        for cl_idx in 0..n_cls {
            // Get clause
            let mut clause = Clause::new();
            let mut lit_idx = 0;
            loop {
                let lit = unsafe { ffi::cmaxpre_get_prepro_lit(self.handle, cl_idx, lit_idx) };
                if lit == 0 {
                    break;
                }
                let lit = Lit::from_ipasir(lit).unwrap();
                self.stats.max_prepro_var =
                    Self::track_max_var(self.stats.max_prepro_var, lit.var());
                clause.add(lit);
                lit_idx += 1;
            }
            // Get weights
            let mut is_hard = true;
            for obj_idx in 0..self.stats.n_objs {
                let w = unsafe {
                    ffi::cmaxpre_get_prepro_weight(self.handle, cl_idx, obj_idx as c_uint)
                };
                if w == 0 {
                    continue;
                }
                if w != top {
                    // Soft clause
                    if softs.len() < obj_idx + 1 {
                        softs.resize(obj_idx + 1, Default::default());
                    }
                    softs[obj_idx].push((clause.clone(), w as usize));
                    is_hard = false;
                }
            }
            if is_hard {
                // Hard clause
                hards.add_clause(clause);
            }
        }
        self.stats.n_prepro_hard_clauses = hards.len();
        self.stats.n_prepro_soft_clauses = softs.iter().map(|s| s.len()).collect();
        self.stats.removed_weight.clear();
        let softs = softs
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                let rem_weight =
                    unsafe { ffi::cmaxpre_get_removed_weight(self.handle, idx as c_uint) } as usize;
                self.stats.removed_weight.push(rem_weight);
                let offset = rem_weight as isize + self.offsets[idx];
                (s, offset)
            })
            .collect();
        (hards, softs)
    }

    fn prepro_labels(&self) -> Vec<Lit> {
        let n_lbls = self.n_prepro_labels();
        let mut lbls = Vec::new();
        for lbl_idx in 0..n_lbls {
            lbls.push(
                Lit::from_ipasir(unsafe { ffi::cmaxpre_get_prepro_label(self.handle, lbl_idx) })
                    .unwrap(),
            );
        }
        lbls
    }

    fn prepro_fixed_lits(&self) -> Vec<Lit> {
        let n_fixed = self.n_prepro_fixed_lits();
        let mut fixed = Vec::new();
        for fixed_idx in 0..n_fixed {
            fixed.push(
                Lit::from_ipasir(unsafe {
                    ffi::cmaxpre_get_prepro_fixed_lit(self.handle, fixed_idx)
                })
                .unwrap(),
            );
        }
        fixed
    }

    fn max_orig_var(&self) -> Var {
        Lit::from_ipasir(unsafe { ffi::cmaxpre_get_original_variables(self.handle) })
            .unwrap()
            .var()
    }

    fn upper_bound(&self) -> u64 {
        unsafe { ffi::cmaxpre_get_upper_bound(self.handle) }
    }

    fn reconstruct(&mut self, sol: Assignment) -> Assignment {
        let start = ProcessTime::now();
        sol.into_iter()
            .for_each(|l| unsafe { ffi::cmaxpre_assignment_add(self.handle, l.to_ipasir()) });
        unsafe { ffi::cmaxpre_reconstruct(self.handle) };
        let max_var = self.max_orig_var();
        let rec = (1..max_var.pos_lit().to_ipasir() + 1)
            .map(|l| {
                if unsafe { ffi::cmaxpre_reconstructed_val(self.handle, l) } > 0 {
                    Lit::from_ipasir(l).unwrap()
                } else {
                    Lit::from_ipasir(-l).unwrap()
                }
            })
            .collect();
        self.stats.reconst_time += start.elapsed();
        rec
    }

    fn add_var(&mut self) -> Result<Var, Error> {
        let v = unsafe { ffi::cmaxpre_add_var(self.handle, 0) };
        if v == 0 {
            return Err(Error::Generic);
        }
        Ok(Lit::from_ipasir(v).unwrap().var())
    }

    fn add_clause(&mut self, clause: Clause) -> Result<(), Error> {
        clause.into_iter().for_each(|l| unsafe {
            ffi::cmaxpre_add_lit(self.handle, l.to_ipasir());
        });
        if unsafe { ffi::cmaxpre_add_lit(self.handle, 0) } == ffi::FALSE {
            return Err(Error::Generic);
        }
        Ok(())
    }

    fn add_label(&mut self, label: Lit, weight: usize) -> Result<Lit, Error> {
        let l = unsafe { ffi::cmaxpre_add_label(self.handle, label.to_ipasir(), weight as u64) };
        if l == 0 {
            return Err(Error::Generic);
        }
        Ok(Lit::from_ipasir(l).unwrap())
    }

    fn alter_weight(&mut self, label: Lit, weight: usize) -> Result<(), Error> {
        if unsafe { ffi::cmaxpre_alter_weight(self.handle, label.to_ipasir(), weight as u64) }
            == ffi::FALSE
        {
            return Err(Error::Generic);
        }
        Ok(())
    }

    fn label_to_var(&mut self, label: Lit) -> Result<(), Error> {
        if unsafe { ffi::cmaxpre_label_to_var(self.handle, label.to_ipasir()) } == ffi::FALSE {
            return Err(Error::Generic);
        }
        Ok(())
    }

    fn reset_removed_weight(&mut self) -> Result<(), Error> {
        if unsafe { ffi::cmaxpre_reset_removed_weight(self.handle) } == ffi::FALSE {
            return Err(Error::Generic);
        }
        Ok(())
    }

    fn removed_weight(&mut self) -> Vec<usize> {
        self.stats.removed_weight = (0..self.stats.n_objs)
            .map(|obj_idx| unsafe {
                ffi::cmaxpre_get_removed_weight(self.handle, obj_idx as c_uint)
            } as usize)
            .collect();
        self.stats.removed_weight.clone()
    }

    fn set_options(&mut self, opts: Options) {
        if let Some(val) = opts.bve_sort_max_first {
            unsafe { ffi::cmaxpre_set_bve_gate_extraction(self.handle, ffi::map_bool(val)) };
        }
        if let Some(val) = opts.label_matching {
            unsafe { ffi::cmaxpre_set_label_matching(self.handle, ffi::map_bool(val)) };
        }
        if let Some(val) = opts.skip_technique {
            unsafe { ffi::cmaxpre_set_skip_technique(self.handle, val) };
        }
        if let Some(val) = opts.bve_sort_max_first {
            unsafe { ffi::cmaxpre_set_bve_sort_max_first(self.handle, ffi::map_bool(val)) };
        }
        if let Some(val) = opts.bve_local_grow_limit {
            unsafe { ffi::cmaxpre_set_bve_local_grow_limit(self.handle, val) };
        }
        if let Some(val) = opts.bve_global_grow_limit {
            unsafe { ffi::cmaxpre_set_bve_global_grow_limit(self.handle, val) };
        }
        if let Some(val) = opts.max_bbtms_vars {
            unsafe { ffi::cmaxpre_set_max_bbtms_vars(self.handle, val) };
        }
        if let Some(val) = opts.harden_in_model_search {
            unsafe { ffi::cmaxpre_set_harden_in_model_search(self.handle, ffi::map_bool(val)) };
        }
        if let Some(val) = opts.model_search_iter_limits {
            unsafe { ffi::cmaxpre_set_model_search_iter_limit(self.handle, val) };
        }
    }

    fn print_instance(&self) {
        unsafe { ffi::cmaxpre_print_instance_stdout(self.handle) }
    }

    fn print_solution(&self, sol: Assignment, weight: usize) {
        sol.into_iter()
            .for_each(|l| unsafe { ffi::cmaxpre_assignment_add(self.handle, l.to_ipasir()) });
        unsafe { ffi::cmaxpre_print_solution_stdout(self.handle, weight as u64) }
    }

    fn print_map(&self) {
        unsafe { ffi::cmaxpre_print_map_stdout(self.handle) }
    }

    fn print_technique_log(&self) {
        unsafe { ffi::cmaxpre_print_technique_log_stdout(self.handle) }
    }

    fn print_info_log(&self) {
        unsafe { ffi::cmaxpre_print_info_log_stdout(self.handle) }
    }

    fn print_stats(&self) {
        unsafe { ffi::cmaxpre_print_preprocessor_stats_stdout(self.handle) }
    }

    fn stats(&self) -> Stats {
        self.stats.clone()
    }
}

impl MaxPre {
    /// Tracks a maximum variable
    fn track_max_var(max_var: Option<Var>, new_var: Var) -> Option<Var> {
        match max_var {
            Some(max_var) => {
                if new_var > max_var {
                    Some(new_var)
                } else {
                    Some(max_var)
                }
            }
            None => Some(new_var),
        }
    }
}

impl Drop for MaxPre {
    fn drop(&mut self) {
        unsafe { ffi::cmaxpre_release(self.handle) }
    }
}

#[cfg(test)]
mod tests {
    use rustsat::{instances::Cnf, lit, types::Clause};

    use crate::PreproClauses;

    use super::MaxPre;

    #[test]
    fn construct() {
        let mut cnf = Cnf::new();
        cnf.add_binary(lit![0], lit![2]);
        MaxPre::new::<Vec<(Clause, usize)>>(cnf, vec![], true);
    }
}
