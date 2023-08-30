/* automatically generated by rust-bindgen 0.60.1 */

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CGlucose {
    _unused: [u8; 0],
}
extern "C" {
    pub fn cglucose_init() -> *mut CGlucose;
}
extern "C" {
    pub fn cglucose_assume(arg1: *mut CGlucose, lit: ::std::os::raw::c_int);
}
extern "C" {
    pub fn cglucose_solve(arg1: *mut CGlucose) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn cglucose_val(arg1: *mut CGlucose, lit: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn cglucose_add_to_clause(arg1: *mut CGlucose, lit: ::std::os::raw::c_int);
}
extern "C" {
    pub fn cglucose_commit_clause(arg1: *mut CGlucose);
}
extern "C" {
    pub fn cglucose_clean_clause(arg1: *mut CGlucose);
}
extern "C" {
    pub fn cglucose_set_random_seed(arg1: *mut CGlucose, seed: f64);
}
extern "C" {
    pub fn cglucose_solver_nodes(arg1: *mut CGlucose) -> ::std::os::raw::c_ulonglong;
}
extern "C" {
    pub fn cglucose_nb_learnt(arg1: *mut CGlucose) -> ::std::os::raw::c_ulonglong;
}
extern "C" {
    pub fn cglucose_print_incremental_stats(arg1: *mut CGlucose);
}
extern "C" {
    pub fn cglucose_clean_learnt_clause(arg1: *mut CGlucose);
}
extern "C" {
    pub fn cglucose_add_to_learnt_clause(wrapper: *mut CGlucose, lit: ::std::os::raw::c_int);
}
extern "C" {
    pub fn cglucose_commit_learnt_clause(arg1: *mut CGlucose);
}
