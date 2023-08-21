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

void iterateList(Solver::ListForInstances list, PyObject* pPlotFunction){
    for(const auto& vecTorOfLists : list){
    Solver::VecList list = std::get<0>(vecTorOfLists);
    std::string instanceName = std::get<1>(vecTorOfLists);

    for(const auto& elem : list){
        std::string name = std::get<1>(elem);
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
        PyObject* pArgs = PyTuple_Pack(3, pyList1, pyList2, pyName);
        PyObject_CallObject(pPlotFunction, pArgs);
        Py_XDECREF(pArgs);
        }
    }
}

//Uses a Vector of tuples, uInt64 and Double. 
void vectorToPython(Solver::ListForInstances list){
    Py_Initialize();
      
        PyRun_SimpleString("import sys");
        //Put here the path to the simp path, need to update it with relative path
        PyRun_SimpleString("sys.path.append(\"/mnt/c/Abschlussarbeit/GitGLUCOSE/CDCL-support-by-BDD-methods/cglucose/simp\")");
        PyObject* pModule = PyImport_ImportModule("plotter");
        PyObject* pPlotFunction = PyObject_GetAttrString(pModule, "plotFromC");
        
    if (!pPlotFunction || !PyCallable_Check(pPlotFunction)) {
        PyErr_Print();
        Py_XDECREF(pPlotFunction);
        Py_DECREF(pModule);
        Py_Finalize();
        printf("Function not found");
    }

    iterateList(list, pPlotFunction);

    Py_DECREF(pPlotFunction);
    Py_DECREF(pModule);
   // Py_Finalize();
}

void solvedInstances(std::vector<std::tuple<int,double>> solvedInstances){
     Py_Initialize();

        PyRun_SimpleString("import sys");
        PyRun_SimpleString("sys.path.append(\"/mnt/c/Abschlussarbeit/CDCL-support-by-BDD-methods/cglucose/simp\")");
        PyObject* pModule = PyImport_ImportModule("plotter");
        PyObject* pPlotFunction = PyObject_GetAttrString(pModule, "plotInstances");

        PyObject* pyList1 = PyList_New(solvedInstances.size());
        PyObject* pyList2 = PyList_New(solvedInstances.size());

        int i = 0;
        for (const auto& tuple : solvedInstances) {

        PyObject* pyInt = PyLong_FromLong(std::get<0>(tuple));
        PyObject* pyFloat = PyFloat_FromDouble(static_cast<double>(get<1>(tuple)));
        PyList_SetItem(pyList1, i, pyInt);
        PyList_SetItem(pyList2, i, pyFloat);
        i++;
        }
        PyObject* pArgs = PyTuple_Pack(2, pyList1, pyList2);
        PyObject_CallObject(pPlotFunction, pArgs);
       
        Py_XDECREF(pArgs);
        Py_DECREF(pPlotFunction);
        Py_DECREF(pModule);
        Py_Finalize();
}