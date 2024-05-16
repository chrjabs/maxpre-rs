//! Reproduction of `cpreprocessorinterface.h`

#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_uint};

pub const TRUE: c_char = 1;
pub const FALSE: c_char = 0;

#[inline]
pub fn map_bool(b: bool) -> c_char {
    if b {
        TRUE
    } else {
        FALSE
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CMaxPre {
    _unused: [u8; 0],
}

#[link(name = "maxpre", kind = "static")]
extern "C" {
    pub fn cmaxpre_signature() -> *const c_char;
    pub fn cmaxpre_init_start(top_weight: u64, inprocess_mode: c_char) -> *mut CMaxPre;
    pub fn cmaxpre_init_add_weight(arg1: *mut CMaxPre, weight: u64);
    pub fn cmaxpre_init_add_lit(arg1: *mut CMaxPre, lit: c_int);
    pub fn cmaxpre_init_finalize(arg1: *mut CMaxPre);
    pub fn cmaxpre_release(arg1: *mut CMaxPre);
    pub fn cmaxpre_preprocess(
        arg1: *mut CMaxPre,
        techniques: *const c_char,
        log_level: c_int,
        time_limit: f64,
        add_removed_weight: c_char,
        sort_labels_frequency: c_char,
    );
    pub fn cmaxpre_get_top_weight(arg1: *mut CMaxPre) -> u64;
    pub fn cmaxpre_get_n_prepro_clauses(arg1: *mut CMaxPre) -> c_uint;
    pub fn cmaxpre_get_prepro_weight(arg1: *mut CMaxPre, cl_idx: c_uint, obj_idx: c_uint) -> u64;
    pub fn cmaxpre_get_prepro_lit(arg1: *mut CMaxPre, cl_idx: c_uint, lit_idx: c_uint) -> c_int;
    pub fn cmaxpre_get_n_prepro_labels(arg1: *mut CMaxPre) -> c_uint;
    pub fn cmaxpre_get_prepro_label(arg1: *mut CMaxPre, lbl_idx: c_uint) -> c_int;
    pub fn cmaxpre_get_n_prepro_fixed(arg1: *mut CMaxPre) -> c_uint;
    pub fn cmaxpre_get_prepro_fixed_lit(arg1: *mut CMaxPre, lit_idx: c_uint) -> c_int;
    pub fn cmaxpre_assignment_reset(arg1: *mut CMaxPre);
    pub fn cmaxpre_assignment_add(arg1: *mut CMaxPre, lit: c_int);
    pub fn cmaxpre_reconstruct(arg1: *mut CMaxPre);
    pub fn cmaxpre_reconstructed_val(arg1: *mut CMaxPre, lit: c_int) -> c_int;
    pub fn cmaxpre_add_var(arg1: *mut CMaxPre, var: c_int) -> c_int;
    pub fn cmaxpre_add_lit(arg1: *mut CMaxPre, lit: c_int) -> c_char;
    pub fn cmaxpre_add_label(arg1: *mut CMaxPre, lbl: c_int, weight: u64) -> c_int;
    pub fn cmaxpre_alter_weight(arg1: *mut CMaxPre, lbl: c_int, weight: u64) -> c_char;
    pub fn cmaxpre_label_to_var(arg1: *mut CMaxPre, lbl: c_int) -> c_char;
    pub fn cmaxpre_reset_removed_weight(arg1: *mut CMaxPre) -> c_char;
    pub fn cmaxpre_get_removed_weight(arg1: *mut CMaxPre, obj_idx: c_uint) -> u64;
    pub fn cmaxpre_set_bve_gate_extraction(arg1: *mut CMaxPre, use_: c_char);
    pub fn cmaxpre_set_label_matching(arg1: *mut CMaxPre, use_: c_char);
    pub fn cmaxpre_set_skip_technique(arg1: *mut CMaxPre, value: c_int);
    pub fn cmaxpre_set_bve_sort_max_first(arg1: *mut CMaxPre, use_: c_char);
    pub fn cmaxpre_set_bve_local_grow_limit(arg1: *mut CMaxPre, limit: c_int);
    pub fn cmaxpre_set_bve_global_grow_limit(arg1: *mut CMaxPre, limit: c_int);
    pub fn cmaxpre_set_max_bbtms_vars(arg1: *mut CMaxPre, limit: c_int);
    pub fn cmaxpre_set_harden_in_model_search(arg1: *mut CMaxPre, harden: c_char);
    pub fn cmaxpre_set_model_search_iter_limit(arg1: *mut CMaxPre, limit: c_int);
    pub fn cmaxpre_get_original_variables(arg1: *mut CMaxPre) -> c_int;
    pub fn cmaxpre_get_upper_bound(arg1: *mut CMaxPre) -> u64;
    pub fn cmaxpre_print_instance_stdout(arg1: *mut CMaxPre);
    pub fn cmaxpre_print_solution_stdout(arg1: *mut CMaxPre, weight: u64);
    pub fn cmaxpre_print_map_stdout(arg1: *mut CMaxPre);
    pub fn cmaxpre_print_technique_log_stdout(arg1: *mut CMaxPre);
    pub fn cmaxpre_print_info_log_stdout(arg1: *mut CMaxPre);
    pub fn cmaxpre_print_preprocessor_stats_stdout(arg1: *mut CMaxPre);
}
