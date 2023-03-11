//! # Rust MaxPre Interface
//!
//! A Rust interface to the [MaxPre](https://bitbucket.org/coreo-group/maxpre2)
//! preprocessor for MaxSAT.

use core::{
    ffi::{c_int, c_uint},
    time::Duration,
};

use rustsat::{
    instances::Cnf,
    types::{Assignment, Clause, Lit, Var, WClsIter},
};

mod base;
mod ffi;
#[cfg(feature = "multiopt")]
mod multiopt;
#[cfg(feature = "optimization")]
mod opt;
mod sat;

// Rexports
pub use base::MaxPre;
#[cfg(feature = "multiopt")]
pub use multiopt::PreproMultiOpt;
#[cfg(feature = "optimization")]
pub use opt::PreproOpt;
pub use sat::PreproSat;

pub type SoftClauses = Vec<(Clause, usize)>;

/// Errors in MaxPre
pub enum Error {
    /// Generic MaxPre Error that is not further specified
    Generic,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unspecified error")
    }
}

pub trait PreproClauses {
    /// Gets the signature of the preprocessor library
    fn signature() -> &'static str;
    /// Initializes a new preprocessor with hard clauses and optional multiple sets of soft clauses.
    fn new<CI: WClsIter>(hards: Cnf, softs: Vec<(CI, isize)>, inprocessing: bool) -> Self;
    /// Performs preprocessing on the internal instance
    fn preprocess(&mut self, techniques: &str, log_level: c_int, time_limit: f64);
    /// Gets the top weight of the preprocessor
    fn top_weight(&self) -> u64;
    /// Gets the number of preprocessed clauses
    fn n_prepro_clauses(&self) -> c_uint;
    /// Gets the number of preprocessed labels
    fn n_prepro_labels(&self) -> c_uint;
    /// Gets the number of fixed literals
    fn n_prepro_fixed_lits(&self) -> c_uint;
    /// Gets the preprocessed instance
    fn prepro_instance(&mut self) -> (Cnf, Vec<(SoftClauses, isize)>);
    /// Gets the preprocessed labels
    fn prepro_labels(&self) -> Vec<Lit>;
    /// Gets the set of literals fixed to true by preprocessing
    fn prepro_fixed_lits(&self) -> Vec<Lit>;
    /// Gets the maximum original variable
    fn max_orig_var(&self) -> Var;
    /// Gets the upper bound on the objective found by preprocessing
    fn upper_bound(&self) -> u64;
    /// Reconstructs an assignment
    fn reconstruct(&mut self, sol: Assignment) -> Assignment;
    /// Adds a new variable to the preprocessor and return the variable
    fn add_var(&mut self) -> Result<Var, Error>;
    /// Adds a clause to the preprocessor
    fn add_clause(&mut self, clause: Clause) -> Result<(), Error>;
    /// Adds a label to the preprocessor
    fn add_label(&mut self, label: Lit, weight: usize) -> Result<Lit, Error>;
    /// Alters the weight of a label
    fn alter_weight(&mut self, label: Lit, weight: usize) -> Result<(), Error>;
    /// Turns a label into a normal variable
    fn label_to_var(&mut self, label: Lit) -> Result<(), Error>;
    /// Resets the removed weight
    fn reset_removed_weight(&mut self) -> Result<(), Error>;
    /// Gets the removed weight
    fn removed_weight(&mut self) -> Vec<usize>;
    /// Sets options for the preprocessor
    fn set_options(&mut self, opts: Options);
    /// Prints the preprocessed instance to stdout
    fn print_instance(&self);
    /// Reconstructs a solution and prints it to stdout
    fn print_solution(&self, sol: Assignment, weight: usize);
    /// Prints the reconstruction map to stdout
    fn print_map(&self);
    /// Prints the technique log to stdout
    fn print_technique_log(&self);
    /// Prints the info log to stdout
    fn print_info_log(&self);
    /// Prints statistics to stdout
    fn print_stats(&self);
    /// Gets statistics of the preprocessor
    fn stats(&self) -> Stats;
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

/// Statistics of the MaxPre preprocessor
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Stats {
    pub n_objs: usize,
    pub n_orig_hard_clauses: usize,
    pub n_orig_soft_clauses: Vec<usize>,
    pub max_orig_var: Option<Var>,
    pub n_prepro_hard_clauses: usize,
    pub n_prepro_soft_clauses: Vec<usize>,
    pub max_prepro_var: Option<Var>,
    pub removed_weight: Vec<usize>,
    pub prepro_time: Duration,
    pub reconst_time: Duration,
}
