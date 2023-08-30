#include "simp/SimpSolver.h"
#include "simp/SolverHelper.h"
#include "utils/System.h"
#include "core/Solver.h"

#include <stdlib.h>

namespace Glucose {

struct Wrapper {
  SimpSolver * solver;
    Wrapper () : solver (new SimpSolver ()){ }
    ~Wrapper () { delete solver; }
};

}

using namespace Glucose;

extern "C" {

#include "wrapper.h"

CGlucose * cglucose_init (void) {
  return (CGlucose*) new Wrapper ();
}

void cglucose_add_to_clause (CGlucose * wrapper, int lit) {
  int var = abs(lit)-1;
  while (var >= ((Wrapper*) wrapper)->solver->nVars()){
    ((Wrapper*) wrapper)->solver->newVar();
  } 
    
  ((Wrapper*) wrapper)->solver->addToTmpClause ( (lit > 0) ? mkLit(var) : ~mkLit(var) );
}

void cglucose_clean_clause(CGlucose * wrapper) {
    ((Wrapper*) wrapper)->solver->cleanTmpClauseVec();
}

void cglucose_commit_clause(CGlucose * wrapper) {
    bool ret = ((Wrapper*) wrapper)->solver->addTmpClause ();
}

void cglucose_assume (CGlucose * wrapper, int lit) {
  Lit c_lit;
  int var = abs(lit)-1;
  ((Wrapper*) wrapper)->solver->addToAssumptionsVec ( (lit > 0) ? mkLit(var) : ~mkLit(var) );
}

int cglucose_solve (CGlucose * wrapper) {
  bool ret = ((Wrapper*) wrapper)->solver->solveWithAssumpLink (false, true);
  ((Wrapper*) wrapper)->solver->clearAssumptions ();
  return !ret;
}

int cglucose_val (CGlucose * wrapper, int lit) {
  return ((Wrapper*) wrapper)->solver->getVal (lit);
}

unsigned long long cglucose_solver_nodes (CGlucose * ptr){
  return ((Wrapper*) ptr)->solver->decisions;
}

unsigned long long cglucose_nb_learnt(CGlucose * ptr){
  return ((Wrapper*) ptr)->solver->getNbLearnt();
}

void cglucose_set_random_seed(CGlucose * ptr, double seed ){
  ((Wrapper*) ptr)->solver->random_seed = seed;
}

void cglucose_print_incremental_stats(CGlucose * wrapper) {
    ((Wrapper*) wrapper)->solver->printIncrementalStats();
}

void cglucose_clean_leanrt_clause(CGlucose * wrapper) {
    ((Wrapper*) wrapper)->solver->cleanTmpLearntClauseVec();
}

void cglucose_add_to_learnt_clause (CGlucose * wrapper, int lit) {
  int var = abs(lit)-1;
  ((Wrapper*) wrapper)->solver->addToTmpLearntClause ( (lit > 0) ? mkLit(var) : ~mkLit(var) );
}

void cglucose_commit_learnt_clause(CGlucose * wrapper) {
    bool ret = ((Wrapper*) wrapper)->solver->addTmpLearntClause ();
}

}
