/***************************************************************************************[Main.cc]
 Glucose -- Copyright (c) 2009-2014, Gilles Audemard, Laurent Simon
                                CRIL - Univ. Artois, France
                                LRI  - Univ. Paris Sud, France (2009-2013)
                                Labri - Univ. Bordeaux, France

 Syrup (Glucose Parallel) -- Copyright (c) 2013-2014, Gilles Audemard, Laurent Simon
                                CRIL - Univ. Artois, France
                                Labri - Univ. Bordeaux, France

Glucose sources are based on MiniSat (see below MiniSat copyrights). Permissions and copyrights of
Glucose (sources until 2013, Glucose 3.0, single core) are exactly the same as Minisat on which it 
is based on. (see below).

Glucose-Syrup sources are based on another copyright. Permissions and copyrights for the parallel
version of Glucose-Syrup (the "Software") are granted, free of charge, to deal with the Software
without restriction, including the rights to use, copy, modify, merge, publish, distribute,
sublicence, and/or sell copies of the Software, and to permit persons to whom the Software is 
furnished to do so, subject to the following conditions:

- The above and below copyrights notices and this permission notice shall be included in all
copies or substantial portions of the Software;
- The parallel version of Glucose (all files modified since Glucose 3.0 releases, 2013) cannot
be used in any competitive event (sat competitions/evaluations) without the express permission of 
the authors (Gilles Audemard / Laurent Simon). This is also the case for any competitive event
using Glucose Parallel as an embedded SAT engine (single core or not).


--------------- Original Minisat Copyrights

Copyright (c) 2003-2006, Niklas Een, Niklas Sorensson
Copyright (c) 2007-2010, Niklas Sorensson

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 **************************************************************************************************/

#include <errno.h>

#include <signal.h>
#include <zlib.h>
#include <sys/resource.h>

#include "utils/System.h"
#include "utils/ParseUtils.h"
#include "utils/Options.h"
#include "core/Dimacs.h"
#include "simp/SimpSolver.h"
#include <iostream>
#include <fstream>


#include "CallPythonFile.h"
#include <iostream>
#include <dlfcn.h> // For dynamic library loading on Unix-based systems
#include <cstring> // For string operations

#include <chrono>

using namespace std;
using namespace Glucose;

//=================================================================================================


typedef struct BddVarOrdering BddVarOrdering;

extern "C" {
    BddVarOrdering* create_bdd_var_ordering(const char* input);
    void free_bdd_var_ordering(BddVarOrdering* ptr);
}

void runRustFunction(const char* filePath) {
    // Load the Rust dynamic library
    void* rust_lib = dlopen("/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/target/release/librust_lib.so", RTLD_LAZY); // Update the path accordingly

    if (!rust_lib) {
        std::cerr << "Failed to load the Rust library: " << dlerror() << std::endl;
        return;
    }

    // Get pointers to the Rust functions
    auto create_bdd_var_ordering_ptr = reinterpret_cast<BddVarOrdering*(*)(const char*)>(dlsym(rust_lib, "create_var_ordering"));
    auto free_bdd_var_ordering_ptr = reinterpret_cast<void(*)(BddVarOrdering*)>(dlsym(rust_lib, "free_var_ordering"));

    if (!create_bdd_var_ordering_ptr || !free_bdd_var_ordering_ptr) {
        std::cerr << "Failed to get function pointers from the Rust library: " << dlerror() << std::endl;
        dlclose(rust_lib);
        return;
    }

    // Call the Rust function to create BddVarOrdering
    BddVarOrdering* bdd_var_ordering = create_bdd_var_ordering_ptr(filePath);

    // Check if the creation was successful
    if (!bdd_var_ordering) {
        std::cerr << "Failed to create BddVarOrdering in Rust" << std::endl;
        dlclose(rust_lib);
        return;
    }

    // Use the BddVarOrdering as needed

    // Free the BddVarOrdering when done
    free_bdd_var_ordering_ptr(bdd_var_ordering);

    // Unload the Rust library
    dlclose(rust_lib);
}

static const char* _certified = "CORE -- CERTIFIED UNSAT";

void printStats(Solver& solver)
{
    double cpu_time = cpuTime();
    double mem_used = 0;//memUsedPeak();
    printf("c restarts              : %" PRIu64" (%" PRIu64" conflicts in avg)\n", solver.starts,(solver.starts>0 ?solver.conflicts/solver.starts : 0));
    printf("c blocked restarts      : %" PRIu64" (multiple: %" PRIu64") \n", solver.nbstopsrestarts,solver.nbstopsrestartssame);
    printf("c last block at restart : %" PRIu64"\n",solver.lastblockatrestart);
    printf("c nb ReduceDB           : %" PRIu64"\n", solver.nbReduceDB);
    printf("c nb removed Clauses    : %" PRIu64"\n",solver.nbRemovedClauses);
    printf("c nb learnts DL2        : %" PRIu64"\n", solver.nbDL2);
    printf("c nb learnts size 2     : %" PRIu64"\n", solver.nbBin);
    printf("c nb learnts size 1     : %" PRIu64"\n", solver.nbUn);

    printf("c conflicts             : %-12" PRIu64"   (%.0f /sec)\n", solver.conflicts   , solver.conflicts   /cpu_time);
    printf("c decisions             : %-12" PRIu64"   (%4.2f %% random) (%.0f /sec)\n", solver.decisions, (float)solver.rnd_decisions*100 / (float)solver.decisions, solver.decisions   /cpu_time);
    printf("c propagations          : %-12" PRIu64"   (%.0f /sec)\n", solver.propagations, solver.propagations/cpu_time);
    printf("c conflict literals     : %-12" PRIu64"   (%4.2f %% deleted)\n", solver.tot_literals, (solver.max_literals - solver.tot_literals)*100 / (double)solver.max_literals);
    printf("c nb reduced Clauses    : %" PRIu64"\n",solver.nbReducedClauses);
    
    if (mem_used != 0) printf("Memory used           : %.2f MB\n", mem_used);
    printf("c CPU time              : %g s\n", cpu_time);
}

static Solver* solver;
// Terminate by notifying the solver and back out gracefully. This is mainly to have a test-case
// for this feature of the Solver as it may take longer than an immediate call to '_exit()'.
static void SIGINT_interrupt(int signum) { solver->interrupt(); }

// Note that '_exit()' rather than 'exit()' has to be used. The reason is that 'exit()' calls
// destructors and may cause deadlocks if a malloc/free function happens to be running (these
// functions are guarded by locks for multithreaded use).
static void SIGINT_exit(int signum) {
    printf("\n"); printf("*** INTERRUPTED ***\n");
    if (solver->verbosity > 0){
        printStats(*solver);
        printf("\n"); printf("*** INTERRUPTED ***\n"); }
    _exit(1); }

//Consists of the VecLists of the generated data that has been tracked and saves
//it into a list for easier transition to the plotter
static Solver::ListForInstances lists;

//Tracks the number of the instanc and the time taken
std::vector<std::tuple<int, double>> instances;


//TODO: anzahl an klauseln am anfang und ende
void saveToListAndCallPython(Solver& S, std::string instanceName){
    
    S.vecList.emplace_back(std::make_tuple(S.restarts, "_restarts"));
    S.vecList.emplace_back(std::make_tuple(S.conf, "_conflicts"));
    S.vecList.emplace_back(std::make_tuple(S.dec, "_decisions"));
    S.vecList.emplace_back(std::make_tuple(S.confLiterals, "_conflicLiterals"));
    S.vecList.emplace_back(std::make_tuple(S.blockedRestarts, "_blockedRestarts"));
    S.vecList.emplace_back(std::make_tuple(S.reducedDatabase, "_reducedDatabase"));
    S.vecList.emplace_back(std::make_tuple(S.propags, "_propagations"));
    lists.emplace_back(std::make_tuple(S.vecList, instanceName));

    S.vecList.clear();
    }




//=================================================================================================
// Main:


int main(int argc, char** argv)
{
    try {
      printf("c\nc This is glucose 4.0 --  based on MiniSAT (Many thanks to MiniSAT team)\nc\n");

      
      setUsageHelp("c USAGE: %s [options] <input-file> <result-output-file>\n\n  where input may be either in plain or gzipped DIMACS.\n");
        
        
#if defined(__linux__)
        fpu_control_t oldcw, newcw;
        _FPU_GETCW(oldcw); newcw = (oldcw & ~_FPU_EXTENDED) | _FPU_DOUBLE; _FPU_SETCW(newcw);
        //printf("c WARNING: for repeatability, setting FPU to use double precision\n");
#endif
        // Extra options:
        //
        IntOption    verb   ("MAIN", "verb",   "Verbosity level (0=silent, 1=some, 2=more).", 1, IntRange(0, 2));
        BoolOption   mod   ("MAIN", "model",   "show model.", false);
        IntOption    vv  ("MAIN", "vv",   "Verbosity every vv conflicts", 10000, IntRange(1,INT32_MAX));
        BoolOption   pre    ("MAIN", "pre",    "Completely turn on/off any preprocessing.", true);
        StringOption dimacs ("MAIN", "dimacs", "If given, stop after preprocessing and write the result to this file.");
        IntOption    cpu_lim("MAIN", "cpu-lim","Limit on CPU time allowed in seconds.\n", INT32_MAX, IntRange(0, INT32_MAX));
        IntOption    mem_lim("MAIN", "mem-lim","Limit on memory usage in megabytes.\n", INT32_MAX, IntRange(0, INT32_MAX));
 //       BoolOption opt_incremental ("MAIN","incremental", "Use incremental SAT solving",false);

         BoolOption    opt_certified      (_certified, "certified",    "Certified UNSAT using DRUP format", false);
         StringOption  opt_certified_file      (_certified, "certified-output",    "Certified UNSAT output file", "NULL");
         
        parseOptions(argc, argv, true);
        double      initial_time = cpuTime();

        // Use signal handlers that forcibly quit until the solver will be able to respond to
        // interrupts:
        signal(SIGINT, SIGINT_exit);
        signal(SIGXCPU,SIGINT_exit);


        // Set limit on CPU-time:
        if (cpu_lim != INT32_MAX){
            rlimit rl;
            getrlimit(RLIMIT_CPU, &rl);
            if (rl.rlim_max == RLIM_INFINITY || (rlim_t)cpu_lim < rl.rlim_max){
                rl.rlim_cur = cpu_lim;
                if (setrlimit(RLIMIT_CPU, &rl) == -1)
                    printf("c WARNING! Could not set resource limit: CPU-time.\n");
            } }

        // Set limit on virtual memory:
        if (mem_lim != INT32_MAX){
            rlim_t new_mem_lim = (rlim_t)mem_lim * 1024*1024;
            rlimit rl;
            getrlimit(RLIMIT_AS, &rl);
            if (rl.rlim_max == RLIM_INFINITY || new_mem_lim < rl.rlim_max){
                rl.rlim_cur = new_mem_lim;
                if (setrlimit(RLIMIT_AS, &rl) == -1)
                    printf("c WARNING! Could not set resource limit: Virtual memory.\n");
            } }
        
        if (argc == 1)
            printf("c Reading from standard input... Use '--help' for help.\n");     
        
        FILE* res = (argc >= 3) ? fopen(argv[argc-1], "wb") : NULL;
 
        // Change to signal-handlers that will only notify the solver and allow it to terminate
        // voluntarily:
        signal(SIGINT, SIGINT_interrupt);
        signal(SIGXCPU,SIGINT_interrupt);

/*
Put the names of the cnf file in the filePaths array, can be done better of course, 
but for testing purpose it is made that simple. Future improvement will be done.
*/        

        const char* filePaths[] = {
            "sgen.cnf",      
            "sgen.cnf"   
           // "fuhs-aprove-16.cnf"
        };

        int size = sizeof(filePaths) / sizeof(filePaths[0]);
        
        if(argc == 2){
            //Loop trough the files and create a new solver for each file
            for (int i = 0; i < size; ++i) {
            SimpSolver S = SimpSolver();

            double initial_time = cpuTime();

            S.parsing = 1;
            S.verbosity = verb;
            S.verbEveryConflicts = vv;
	        S.showModel = mod;
            S.certifiedUNSAT = opt_certified;
            if(S.certifiedUNSAT) {
            if(!strcmp(opt_certified_file,"NULL")) {
            S.certifiedOutput =  fopen("/dev/stdout", "wb");
            } else {
                S.certifiedOutput =  fopen(opt_certified_file, "wb");	    
            }
            fprintf(S.certifiedOutput,"o proof DRUP\n");
        }

        if (S.verbosity > 0){
            printf("c ========================================[ Problem Statistics ]===========================================\n");
            printf("c |                                                                                                           |\n"); }
        if (S.verbosity > 0){
            printf("c |  Number of variables:  %12d                                                                   |\n", S.nVars());
            printf("c |  Number of clauses:    %12d                                                                   |\n", S.nClauses()); }
        
        double parsed_time = cpuTime();
        if (S.verbosity > 0){
            printf("c |  Parse time:           %12.2f s                                                                 |\n", parsed_time - initial_time);
            printf("c |                                                                                                       |\n"); }
         S.parsing = 0;

         if(pre/* && !S.isIncremental()*/) {
	  printf("c | Preprocesing is fully done\n");
	  S.eliminate(true);
        double simplified_time = cpuTime();
        if (S.verbosity > 0){
            printf("c |  Simplification time:  %12.2f s                                                                 |\n", simplified_time - parsed_time);
 }
	}
	printf("c |                                                                                                       |\n");
        if (!S.okay()){
            if (S.certifiedUNSAT) fprintf(S.certifiedOutput, "0\n"), fclose(S.certifiedOutput);
            if (res != NULL) fprintf(res, "UNSAT\n"), fclose(res);
            if (S.verbosity > 0){
 	        printf("c =========================================================================================================\n");
               printf("Solved by simplification\n");
                printStats(S);
                printf("\n"); }
            printf("s UNSATISFIABLE\n");        
            exit(20);
        }

        if (dimacs){
            if (S.verbosity > 0)
                printf("c =======================================[ Writing DIMACS ]===============================================\n");
            S.toDimacs((const char*)dimacs);
            if (S.verbosity > 0)
                printStats(S);
            exit(0);
        }
            gzFile in = gzopen(filePaths[i],"rb"); 
            parse_DIMACS(in, S);
            gzclose(in);

            runRustFunction(filePaths[i]);

            vec<Lit> dummy;
            lbool ret = S.solveLimited(dummy);

             if (S.verbosity > 0){
            printStats(S);
            printf("\n"); }
            printf(ret == l_True ? "s SATISFIABLE\n" : ret == l_False ? "s UNSATISFIABLE\n" : "s INDETERMINATE\n");
            std::string instanceName = filePaths[i]; 
            saveToListAndCallPython(S, instanceName);
            instances.emplace_back(i+1,cpuTime());
            }
            vectorToPython(lists);
            solvedInstances(instances);
        }


    } catch (OutOfMemoryException&){
	        printf("c =========================================================================================================\n");
        printf("INDETERMINATE\n");
        exit(0);
    }
}
