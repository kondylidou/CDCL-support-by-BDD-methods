use std::time::Instant;
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::{parser, variable_ordering::{var_ordering::BddVarOrdering, bucket::Bucket}, sharing::clause_database::ClauseDatabase};

#[no_mangle]
pub extern "C" fn init(path: *const c_char) -> *mut BddVarOrdering {
    // Convert the C string to a Rust &str
    let path_str = unsafe {
        CStr::from_ptr(path)
            .to_str()
            .expect("Failed to convert C string to &str")
    };
    println!("Solving file: {}", path_str);
    let start = Instant::now();
    // create the Dimacs instance
    let expressions = parser::parse_dimacs_cnf_file(path_str).unwrap();
    println!(
        "Time elapsed to parse the CNF formula : {:?}",
        start.elapsed()
    );

    let start = Instant::now();
   
    let var_ordering = BddVarOrdering::new(expressions);
    println!(
        "Time elapsed to create the variable ordering : {:?}",
        start.elapsed()
    );
    Box::into_raw(Box::new(var_ordering))
}

// Define a function to free the memory allocated for the BddVarOrdering
#[no_mangle]
pub extern "C" fn free_var_ordering(ptr: *mut BddVarOrdering) {
    if !ptr.is_null() {
        // Deallocate the memory when it's no longer needed
        unsafe { Box::from_raw(ptr) };
    }
}

#[no_mangle]
pub extern "C" fn create_buckets(var_ordering_ptr: *mut BddVarOrdering) -> *mut Vec<Bucket> {
    let start = Instant::now();

    // Safety: This is safe because we trust that the provided pointer is valid.
    let var_ordering = unsafe {&mut  *var_ordering_ptr };

    let buckets = var_ordering.group_clauses_into_buckets();
    println!(
        "Time elapsed to create the buckets : {:?}",
        start.elapsed()
    );
    Box::into_raw(Box::new(buckets))
}

#[no_mangle]
pub extern "C" fn initialize_clause_database() -> *mut ClauseDatabase {
    Box::into_raw(Box::new(ClauseDatabase::new()))
}

#[no_mangle]
pub extern "C" fn run(var_ordering_ptr: *mut BddVarOrdering, buckets_ptr: *mut Vec<Bucket>, clause_database_ptr: *mut ClauseDatabase) -> *const Vec<i32> { 
    // Safety: This is safe because we trust that the provided pointer is valid.
    let var_ordering = unsafe {&mut  *var_ordering_ptr };
    let buckets = unsafe {&mut *buckets_ptr };
    println!("{:?}", buckets.len());
    let clause_database = unsafe {&mut  *clause_database_ptr };
    let mut learnts = Vec::new();
    var_ordering.build(buckets, clause_database, &mut learnts);
    println!("l{:?}", learnts.len());
    println!("b{:?}", buckets.len());
    let learnts_ptr = learnts.as_ptr();
    learnts_ptr
}