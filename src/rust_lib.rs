use std::time::Instant;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::slice;
use std::usize;

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
        unsafe { let _ = Box::from_raw(ptr); };
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
pub extern "C" fn run(var_ordering_ptr: *mut BddVarOrdering, buckets_ptr: *mut Vec<Bucket>, clause_database_ptr: *mut ClauseDatabase) -> (*const i32, usize) { 
    // Safety: This is safe because we trust that the provided pointer is valid.
    let var_ordering = unsafe {&mut  *var_ordering_ptr };
    let buckets = unsafe {&mut *buckets_ptr };
    if buckets.is_empty() {
        return (std::ptr::null(),0);
    }
    let clause_database = unsafe {&mut  *clause_database_ptr };
    let mut learnts = Vec::new();
    var_ordering.build(buckets, clause_database, &mut learnts);
    println!("tmp_learnts in rust size {:?}", learnts.len());

    // we need to safely convert the vector of learnt clauses to a convertible data structure in C
    // as we cannot pass the whole vector of vectors (clauses are represented as vectors of integers)
    // we re convert the clauses in the format they were in the initial cnf file, with the zeros
    // defining the gap between each close.

    /* 
    for learnt in learnts {
        for lit in learnt {
            learnts_conversion.push(lit);
        }
        learnts_conversion.push(0);
    }
   
    println!("learnts in rust size {:?}", learnts_conversion.len());
    
    learnts_conversion.shrink_to_fit(); // Ensure minimal memory usage
    // Convert the vector into a heap-allocated array and return a pointer
    let ptr = learnts_conversion.as_mut_ptr();
    std::mem::forget(learnts_conversion); // Prevent Rust from cleaning up the memory
    ptr
*/

    let mut learnts_conversion: Vec<i32> = Vec::new();
    for learnt in learnts {
        for lit in learnt {
            learnts_conversion.push(lit);
        }
        learnts_conversion.push(0);
    }
   
    let length = learnts_conversion.len();
    let ptr = learnts_conversion.as_ptr();
    println!("Length of learnts converted in Rust: {:?}", learnts_conversion.len());

    std::mem::forget(learnts_conversion);

    (ptr,length)
}