/*
    Simple file to translate C++ Data into PyObjects and call the Python file with the
    Data that has been generated. Maybe need to update your own Path down in the code, 
    if Segmentation fault is occuring, then maybe PyInitialize and PyFinalize is not working
    correctly. 
    
*/

#include "Python.h"
#include <memory>
#include "SimpSolver.h"
#include <list>
#include <string>

using namespace std;
using namespace Glucose;

const int VECLIST = 0;
const int NAME = 1;
const int CLAUSES_AT_START = 2;
const int CLAUSES_AT_END = 3;
const int NUMBER_OF_VARIABLES = 4;
const int LONGEST_ClAUSE_PRECALC = 5;
const int LONGEST_LEARNT_CLAUSE = 6;
const int CPU_TIME = 7;
const int BOOL_VAL = 8;
const int WITH_BDD = 9;


void iterateList(Solver::ListForInstances list, PyObject* pPlotFunction, PyObject* raw){
    for(const auto& vecTorOfLists : list){
    Solver::VecList list = std::get<VECLIST>(vecTorOfLists);
    std::string instanceName = std::get<NAME>(vecTorOfLists);
    PyObject* pyIntClausesS = PyLong_FromLong(std::get<CLAUSES_AT_START>(vecTorOfLists));
    PyObject* pyIntClausesE = PyLong_FromLong(std::get<CLAUSES_AT_END>(vecTorOfLists));
    PyObject* pyIntNumberV = PyLong_FromLong(std::get<NUMBER_OF_VARIABLES>(vecTorOfLists));
    PyObject* pyIntLongestClausePreCalc = PyLong_FromLong(std::get<LONGEST_ClAUSE_PRECALC>(vecTorOfLists));
    PyObject* pyIntLongestLearntClause = PyLong_FromLong(std::get<LONGEST_LEARNT_CLAUSE>(vecTorOfLists));
    PyObject* pyDoubleTime = PyFloat_FromDouble(std::get<CPU_TIME>(vecTorOfLists));
    PyObject* pyBoolVal = PyUnicode_DecodeUTF8(std::get<BOOL_VAL>(vecTorOfLists).c_str(), std::get<BOOL_VAL>(vecTorOfLists).size(), "strict");
    PyObject* pyBoolWithBDD = PyBool_FromLong(static_cast<long>(std::get<WITH_BDD>(vecTorOfLists)));

    for(const auto& elem : list){
        std::string name = std::get<NAME>(elem);
        std::string pngName = instanceName + name;
        Solver::VecTuple vec = std::get<0>(elem);
                  
        PyObject* pyList1 = PyList_New(vec.size());
        PyObject* pyList2 = PyList_New(vec.size());

        int i = 0;
        for (const auto& tuple : vec) {

        PyObject* pyInt = PyLong_FromLong(std::get<0>(tuple));
        PyObject* pyFloat = PyFloat_FromDouble(static_cast<double>(get<1>(tuple)));
        PyList_SetItem(pyList1, i, pyInt);
        PyList_SetItem(pyList2, i, pyFloat);
        i++;
        }

        // Convert the C-style string to a Python string object
        PyObject* pyName = PyUnicode_FromString(pngName.c_str());
        PyObject* pArgs = PyTuple_Pack(11, pyList1, pyList2, pyName, pyIntClausesS,pyIntClausesE, pyIntNumberV, pyIntLongestClausePreCalc, pyIntLongestLearntClause, pyDoubleTime, pyBoolVal, pyBoolWithBDD);
        PyObject_CallObject(pPlotFunction, pArgs);
        Py_XDECREF(pArgs);
        Py_XDECREF(pyList1);
        Py_XDECREF(pyList2);
        Py_XDECREF(pyName);
        }
        // Safe data to txt
        PyObject* iName = PyUnicode_FromString(instanceName.c_str());
        PyObject* pArgs2 = PyTuple_Pack(2, iName, pyBoolWithBDD);
        PyObject_CallObject(raw, pArgs2);
        Py_XDECREF(pArgs2);
        Py_XDECREF(iName);
    }
}

//Uses a Vector of tuples, uInt64 and Double. 
void vectorToPython(Solver::ListForInstances list){
    Py_Initialize();
      
        PyRun_SimpleString("import sys");
        //Put here the path to the simp path, need to update it with relative path
        PyRun_SimpleString("sys.path.append(\"/home/admin/Abschlussarbeit/CDCL-support-by-BDD-methods/cglucose/simp\")");
        PyObject* pModule = PyImport_ImportModule("plotter");
        PyObject* pPlotFunction = PyObject_GetAttrString(pModule, "plotFromC");
        PyObject* raw = PyObject_GetAttrString(pModule, "safeRawData");

    if (!pPlotFunction || !PyCallable_Check(pPlotFunction)) {
        PyErr_Print();
        Py_XDECREF(pPlotFunction);
        Py_DECREF(pModule);
        Py_Finalize();
        printf("Function not found");
    }

    iterateList(list, pPlotFunction, raw);
    
    Py_DECREF(raw);
    Py_DECREF(pPlotFunction);
    Py_DECREF(pModule);
    Py_Finalize();
}
