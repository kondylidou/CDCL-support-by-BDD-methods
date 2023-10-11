from curses import raw
from re import T
from site import abs_paths
import matplotlib.pyplot as plt
import os
import matplotlib.ticker as ticker

plt.ioff()
plt.rcParams['lines.antialiased'] = True

FIGURE_SIZE = (12, 12)
GRID_COLOR = 'b'
MARKER = "."
LINE_WIDTH = 0.5
MARKER_SIZE = 4
DPI = 300
LINE_STYLE = "-"
CACTUSPLOT_MARKER = "x"
PATH = os.path.join('/mnt/c/Abschlussarbeit/GitGLUCOSE/CDCL-support-by-BDD-methods/cglucose/simp/Images/')
FILE_RAW = "rawDog.txt"

dataListsFromC = []
timeValuesFromC = []

dataOrigin = []
extraInfo = []

def plotFromC(data,time,name, clausesAtStart, clausesAtEnd, NumberOfVariables, longestClause, longestLearntClause, timeSolved, result):
    dataListsFromC.append(data)
    timeValuesFromC.append(time)
    name_without_extension = name.split('$')[0]
    extension = name.split('$')[1]
    dataOrigin.append(extension)
    safePath = os.path.join(PATH + name_without_extension + "/")
    iterateArgs(clausesAtStart, clausesAtEnd, NumberOfVariables, longestClause, longestLearntClause, timeSolved, result)

    # Plot generation
    plt.figure(figsize= FIGURE_SIZE)
    plt.grid(True)        
    plt.figure(figsize= FIGURE_SIZE)
    plt.plot(time, data, marker = MARKER, linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.xlabel('Time')
    plt.ylabel(name)
    s1 = name + "      " + "Clauses at start: " + str(clausesAtStart) 
    s2 = "          Clauses at end: " + str(clausesAtEnd)
    s3 = "          Number of Variables: " + str(NumberOfVariables)
    s4 = "          Time used for Solving: " + str(timeSolved)
    plt.title(s1 + s2 + s3 + s4)
    
    info_text = 'Longest clause size on start: ' + str(longestClause) + '\n'
    info_text += 'Longest learnt clause: ' + str(longestLearntClause) + '\n'
    
    plt.annotate(info_text, xy=(0.2, -0.1), xycoords='axes fraction',
             fontsize=12, ha='center', va='center', bbox=dict(boxstyle='round,pad=0.5', facecolor='lightgray'))
    if not os.path.exists(PATH + name_without_extension):
    # If it doesn't exist, create the folder
        os.mkdir(PATH + name_without_extension)
        print(PATH + name_without_extension)
    
    plt.savefig(safePath + extension + '.png')
    plt.close()
        
def numberOfSolvedInstances(instance, timeTaken):
    figure, axis = plt.subplots()
    axis.plot(instance,timeTaken, marker = CACTUSPLOT_MARKER)
    axis.xaxis.set_major_locator(ticker.MaxNLocator(integer=True)) 
    axis.xaxis.set_major_formatter(ticker.FormatStrFormatter('%d'))   
    axis.set_xlabel("Instances")
    axis.set_ylabel("CPU Time in (s)")
    plt.savefig(PATH + 'plot.png', dpi=DPI, bbox_inches='tight')
    plt.close()
    
def safeRawData(name):
    #write raw data to the txt, so we can use the data afterwards as well
    rawDataPath = os.path.join(PATH + name + "/" + FILE_RAW)
    with open(rawDataPath, "w") as file:
        index = 0
        file.write(str(extraInfo) + "\n")
        while index < 7:
            dataList = dataListsFromC[index]
            timeList = timeValuesFromC[index]
            line = ','.join(map(str, dataList))
            lineTime = ','.join(map(str, timeList))
            file.write(dataOrigin[index] + ":\n" + line +"\n" +'\n')
            file.write(dataOrigin[index] + "_time"+ ":\n" + lineTime +"\n" +'\n')
            index +=1
    dataListsFromC.clear()
    timeValuesFromC.clear()
    dataOrigin.clear()
    extraInfo.clear()
    
def iterateArgs(*args):
    for arg in args:
        if(arg not in extraInfo):
            extraInfo.append(arg)

