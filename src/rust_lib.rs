use std::time::Instant;
use std::ffi::CStr;
use std::os::raw::c_char;
use crate::{parser, variable_ordering::var_ordering::BddVarOrdering};

#[no_mangle]
pub extern "C" fn create_var_ordering(path: *const c_char) -> *mut BddVarOrdering {
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

    // Bucket Clustering
    //let buckets = var_ordering.group_clauses_into_buckets();
    //let _ = var_ordering.build(buckets, &mut sharing_manager);
    
    // Box the struct and return a raw pointer to it
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
