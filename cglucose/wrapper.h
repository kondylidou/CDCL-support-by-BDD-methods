typedef struct CGlucose CGlucose;

CGlucose * cglucose_init (void);
void cglucose_assume (CGlucose *, int lit);
int cglucose_solve (CGlucose *);
int cglucose_val (CGlucose *, int lit);
void cglucose_add_to_clause (CGlucose * , int lit );
void cglucose_commit_clause(CGlucose * );
void cglucose_clean_clause(CGlucose * );
void cglucose_set_random_seed(CGlucose *, double seed );
unsigned long long cglucose_solver_nodes(CGlucose *);
unsigned long long cglucose_nb_learnt(CGlucose *);
void cglucose_print_incremental_stats(CGlucose *);
void cglucose_clean_learnt_clause(CGlucose * );
void cglucose_add_to_learnt_clause (CGlucose * wrapper, int lit);
void cglucose_commit_learnt_clause(CGlucose * );