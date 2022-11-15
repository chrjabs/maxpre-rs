//! # Rust MaxPre Interface
//!
//! A Rust interface to the [MaxPre](https://bitbucket.org/coreo-group/maxpre2)
//! preprocessor for MaxSAT.

use core::ffi::{c_char, c_int, c_uint, CStr};

use rustsat::{
    instances::CNF,
    types::{Assignment, Clause, Lit, RsHashMap, Var},
};

mod ffi;

/// The main preprocessor type
pub struct MaxPre {
    /// The handle for the C API
    handle: *mut ffi::CMaxPre,
    /// The number of objectives in the preprocessor
    n_obj: usize,
}

impl MaxPre {
    /// Gets the signature of the preprocessor library
    pub fn signature() -> &'static str {
        let c_chars = unsafe { ffi::cmaxpre_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("MaxPre signature returned invalid UTF-8")
    }

    /// Initializes a new preprocessor with hard clauses and optional multiple sets of soft clauses.
    pub fn new(hards: CNF, softs: Vec<RsHashMap<Clause, usize>>, inprocessing: bool) -> Self {
        let mut top = 1;
        top = softs.iter().fold(top, |top, softs| {
            softs.iter().fold(top, |top, (_, w)| top + w)
        });
        let n_obj = softs.len();
        let handle = unsafe { ffi::cmaxpre_init_start(top as u64, ffi::map_bool(inprocessing)) };
        hards.into_iter().for_each(|cl| {
            cl.into_iter()
                .for_each(|l| unsafe { ffi::cmaxpre_init_add_lit(handle, l.to_ipasir()) });
            unsafe { ffi::cmaxpre_init_add_lit(handle, 0) };
        });
        softs.into_iter().enumerate().for_each(|(idx, softs)| {
            softs.into_iter().for_each(|(cl, w)| {
                // Add zero weight for all previous objectives
                (0..idx).for_each(|_| unsafe { ffi::cmaxpre_init_add_weight(handle, 0) });
                // Add weight for the objective with index
                unsafe { ffi::cmaxpre_init_add_weight(handle, w as u64) };
                // Add literals
                cl.into_iter()
                    .for_each(|l| unsafe { ffi::cmaxpre_init_add_lit(handle, l.to_ipasir()) });
                unsafe { ffi::cmaxpre_init_add_lit(handle, 0) };
            })
        });
        unsafe { ffi::cmaxpre_init_finalize(handle) };
        Self { handle, n_obj }
    }

    /// Performs preprocessing on the internal instance
    pub fn preprocess(
        &mut self,
        techniques: &str,
        log_level: c_int,
        time_limit: f64,
        add_removed_weight: bool,
    ) {
        unsafe {
            ffi::cmaxpre_preprocess(
                self.handle,
                techniques.as_ptr() as *mut c_char,
                log_level,
                time_limit,
                ffi::map_bool(add_removed_weight),
            )
        };
    }

    /// Gets the top weight of the preprocessor
    pub fn top_weight(&self) -> u64 {
        unsafe { ffi::cmaxpre_get_top_weight(self.handle) }
    }

    /// Gets the number of preprocessed clauses
    pub fn n_prepro_clauses(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_clauses(self.handle) }
    }

    /// Gets the number of preprocessed labels
    pub fn n_prepro_labels(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_labels(self.handle) }
    }

    /// Gets the number of fixed literals
    pub fn n_prepro_fixed_lits(&self) -> c_uint {
        unsafe { ffi::cmaxpre_get_n_prepro_fixed(self.handle) }
    }

    /// Gets the preprocessed instance
    pub fn prepro_instance(&self) -> (CNF, Vec<RsHashMap<Clause, usize>>) {
        let n_cls = self.n_prepro_clauses();
        let top = self.top_weight();
        let mut hards = CNF::new();
        let mut softs: Vec<RsHashMap<Clause, usize>> = Vec::new();
        for cl_idx in 0..n_cls {
            // Get clause
            let mut clause = Clause::new();
            let mut lit_idx = 0;
            loop {
                let lit = unsafe { ffi::cmaxpre_get_prepro_lit(self.handle, cl_idx, lit_idx) };
                if lit == 0 {
                    break;
                }
                clause.add(Lit::from_ipasir(lit).unwrap());
                lit_idx += 1;
            }
            // Get weights
            let mut is_hard = true;
            for obj_idx in 0..self.n_obj {
                let w = unsafe {
                    ffi::cmaxpre_get_prepro_weight(self.handle, cl_idx, obj_idx as c_uint)
                };
                if w != top {
                    // Soft clause
                    if softs.len() < obj_idx + 1 {
                        softs.resize(obj_idx + 1, Default::default());
                    }
                    softs[obj_idx].insert(clause.clone(), w as usize);
                    is_hard = false;
                }
            }
            if is_hard {
                // Hard clause
                hards.add_clause(clause);
            }
        }
        (hards, softs)
    }

    /// Gets the preprocessed labels
    pub fn prepro_labels(&self) -> Vec<Lit> {
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

    /// Gets the set of literals fixed to true by preprocessing
    pub fn prepro_fixed_lits(&self) -> Vec<Lit> {
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

    /// Gets the maximum original variable
    pub fn max_orig_var(&self) -> Var {
        Lit::from_ipasir(unsafe { ffi::cmaxpre_get_original_variables(self.handle) })
            .unwrap()
            .var()
    }

    /// Reconstructs an assignment
    pub fn reconstruct(&self, sol: Assignment) -> Assignment {
        sol.into_iter()
            .for_each(|l| unsafe { ffi::cmaxpre_assignment_add(self.handle, l.to_ipasir()) });
        unsafe { ffi::cmaxpre_reconstruct(self.handle) };
        let max_var = self.max_orig_var();
        (1..max_var.pos_lit().to_ipasir())
            .map(|l| {
                if unsafe { ffi::cmaxpre_reconstructed_val(self.handle, l) } > 0 {
                    Lit::from_ipasir(l).unwrap()
                } else {
                    Lit::from_ipasir(-l).unwrap()
                }
            })
            .collect()
    }

    /// Adds a new variable to the preprocessor and return the variable
    pub fn add_var(&mut self) -> Result<Var, ()> {
        let v = unsafe { ffi::cmaxpre_add_var(self.handle, 0) };
        if v == 0 {
            return Err(());
        }
        Ok(Lit::from_ipasir(v).unwrap().var())
    }

    /// Adds a clause to the preprocessor
    pub fn add_clause(&mut self, clause: Clause) -> Result<(), ()> {
        clause.into_iter().for_each(|l| unsafe {
            ffi::cmaxpre_add_lit(self.handle, l.to_ipasir());
        });
        if unsafe { ffi::cmaxpre_add_lit(self.handle, 0) } == ffi::FALSE {
            return Err(());
        }
        Ok(())
    }

    /// Adds a label to the preprocessor
    pub fn add_label(&mut self, label: Lit, weight: usize) -> Result<Lit, ()> {
        let l = unsafe { ffi::cmaxpre_add_label(self.handle, label.to_ipasir(), weight as u64) };
        if l == 0 {
            return Err(());
        }
        Ok(Lit::from_ipasir(l).unwrap())
    }

    /// Alters the weight of a label
    pub fn alter_weight(&mut self, label: Lit, weight: usize) -> Result<(), ()> {
        if unsafe { ffi::cmaxpre_alter_weight(self.handle, label.to_ipasir(), weight as u64) }
            == ffi::FALSE
        {
            return Err(());
        }
        Ok(())
    }

    /// Turns a label into a normal variable
    pub fn label_to_var(&mut self, label: Lit) -> Result<(), ()> {
        if unsafe { ffi::cmaxpre_label_to_var(self.handle, label.to_ipasir()) } == ffi::FALSE {
            return Err(());
        }
        Ok(())
    }

    /// Resets the removed weight
    pub fn reset_removed_weight(&mut self) -> Result<(), ()> {
        if unsafe { ffi::cmaxpre_reset_removed_weight(self.handle) } == ffi::FALSE {
            return Err(());
        }
        Ok(())
    }

    /// Gets the removed weight
    pub fn removed_weight(&self) -> Vec<usize> {
        (0..self.n_obj)
            .map(|obj_idx| unsafe {
                ffi::cmaxpre_get_removed_weight(self.handle, obj_idx as c_uint)
            } as usize)
            .collect()
    }

    /// Sets options for the preprocessor
    pub fn set_options(&mut self, opts: Options) {
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

    /// Prints the preprocessed instance to stdout
    pub fn print_instance(&self) {
        unsafe { ffi::cmaxpre_print_instance_stdout(self.handle) }
    }

    /// Reconstructs a solution and prints it to stdout
    pub fn print_solution(&self, sol: Assignment, weight: usize) {
        sol.into_iter()
            .for_each(|l| unsafe { ffi::cmaxpre_assignment_add(self.handle, l.to_ipasir()) });
        unsafe { ffi::cmaxpre_print_solution_stdout(self.handle, weight as u64) }
    }

    /// Prints the reconstruction map to stdout
    pub fn print_map(&self) {
        unsafe { ffi::cmaxpre_print_map_stdout(self.handle) }
    }

    /// Prints the technique log to stdout
    pub fn print_technique_log(&self) {
        unsafe { ffi::cmaxpre_print_technique_log_stdout(self.handle) }
    }

    /// Prints the info log to stdout
    pub fn print_info_log(&self) {
        unsafe { ffi::cmaxpre_print_info_log_stdout(self.handle) }
    }

    /// Prints statistics to stdout
    pub fn print_stats(&self) {
        unsafe { ffi::cmaxpre_print_preprocessor_stats_stdout(self.handle) }
    }
}

impl Drop for MaxPre {
    fn drop(&mut self) {
        unsafe { ffi::cmaxpre_release(self.handle) }
    }
}

/// Options that can be set for MaxPre
#[derive(Clone, Default)]
pub struct Options {
    pub bve_gate_extraction: Option<bool>,
    pub label_matching: Option<bool>,
    pub skip_technique: Option<c_int>,
    pub bve_sort_max_first: Option<bool>,
    pub bve_local_grow_limit: Option<c_int>,
    pub bve_global_grow_limit: Option<c_int>,
    pub max_bbtms_vars: Option<c_int>,
    pub harden_in_model_search: Option<bool>,
    pub model_search_iter_limits: Option<c_int>,
}

#[cfg(test)]
mod tests {
    use rustsat::{instances::CNF, lit, types::Lit};

    use super::MaxPre;

    #[test]
    fn construct() {
        let mut cnf = CNF::new();
        cnf.add_binary(lit![0], lit![2]);
        MaxPre::new(cnf, vec![], true);
    }
}
