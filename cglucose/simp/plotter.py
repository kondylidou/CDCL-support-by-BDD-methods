from site import abs_paths
import matplotlib.pyplot as plt
import numpy as np
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


def plotFromC(data,time,name):
    plt.grid(True)
    plt.figure(figsize= FIGURE_SIZE)
    plt.plot(time, data, marker = MARKER, linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.xlabel('Time')
    plt.ylabel(name)
    plt.title('Data Plot')
    plt.savefig(PATH + name + '.png')
    plt.close()
        
    
def plotInstances(instance, timeTaken):
    figure, axis = plt.subplots()
    axis.plot(instance,timeTaken, marker = CACTUSPLOT_MARKER)
    axis.xaxis.set_major_locator(ticker.MaxNLocator(integer=True)) 
    axis.xaxis.set_major_formatter(ticker.FormatStrFormatter('%d'))   
    axis.set_xlabel("Instances")
    axis.set_ylabel("CPU Time in (s)")
    plt.savefig(PATH + 'plot.png', dpi=DPI, bbox_inches='tight')
    