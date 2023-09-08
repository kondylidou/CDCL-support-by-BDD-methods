use std::time::Instant;

use crate::{parser, GlucoseWrapper, sharing::sharing_manager::SharingManager, variable_ordering::var_ordering::BddVarOrdering, bindings::CGlucose};

#[no_mangle]
pub extern "C" fn init(s: *mut CGlucose) {
    let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/0b1041a1e55af6f3d2c63462a7400bd2-fermat-907547022132073.cnf";

    let start = Instant::now();
    // create the Dimacs instance
    let expressions = parser::parse_dimacs_cnf_file(path).unwrap();
    println!(
        "Time elapsed to parse the CNF formula : {:?}",
        start.elapsed()
    );

    let start = Instant::now();
    
    let glucose = GlucoseWrapper::new(s);
    // build the sharing manager
    let mut sharing_manager = SharingManager::new(glucose);
    // build the variable ordering
    let mut var_ordering = BddVarOrdering::new(expressions);
    println!(
        "Time elapsed to create the variable ordering : {:?}",
        start.elapsed()
    );

    // Bucket Clustering
    let buckets = var_ordering.group_clauses_into_buckets();
    let _ = var_ordering.build(buckets, &mut sharing_manager);
}
