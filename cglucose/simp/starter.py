import os
import subprocess
from builtins import print
import matplotlib.pyplot as plt

#################################################################################################################
#  This script takes a folder with the KNF Formulas and runs them, each once for Glucose then for the BDDS      #
#                                                                                                               #
#               #############################################################################                   #
#               #                                                                           #                   #
#               #       Also make sure to have a reasonable amount of Files to process :)   #                   #
#               #                                                                           #                   #
#               #############################################################################                   #
#                                                                                                               #
#              This file has been written by Danail Raykov, in case of questions, write me on Github!           #
#################################################################################################################

#############################################################################################################################################
# Path to the .cnf files - Search up some CNF files and put them together in a folder! Change to your Folder that contains your cnf files!   
# This is the only path you need to change!
PATH_TO_CNF_FILES = '/home/admin/Abschlussarbeit/CDCL-support-by-BDD-methods/cglucose/simp/tester'
#############################################################################################################################################

def create_root():
    current_working_directory = os.getcwd()
    if not os.path.exists(current_working_directory + "/Tests"):
        os.mkdir(current_working_directory + "/Tests")
        print("Created Root folder... >>> " + current_working_directory + "/Tests")
        
    if not os.path.exists(current_working_directory + "/Tests/GlucoseData"):
        os.mkdir(current_working_directory + "/Tests/GlucoseData")
        print("Created Folder for Glucose data for your Plots and tests >>> " + current_working_directory + "/Tests/GlucoseData")
        
    if not os.path.exists(current_working_directory + "/Tests/BDDsData"):
        os.mkdir(current_working_directory + "/Tests/BDDsData")
        print("Created Folder for BDDs for your Plots and tests >>> " + current_working_directory + "/Tests/BDDsData")        
        
create_root()

ROOT_FOLDER = os.path.join(os.getcwd()+ "/Tests")
def createNeededFolders():
    if not os.path.exists(ROOT_FOLDER + "/KEYWORDS_plots"):
        os.mkdir(ROOT_FOLDER + "/KEYWORDS_plots")
        print("Created Folder for Keywords Plots data for your Plots and tests >>> " + ROOT_FOLDER + "/KEYWORDS_plots")
    
    if not os.path.exists(ROOT_FOLDER + "/Plots"):
        os.mkdir(ROOT_FOLDER + "/Plots")
        print("Created Folder for Plots >>> " + ROOT_FOLDER + "/Plots")
        
    if not os.path.exists(ROOT_FOLDER + "/FirstLinesExtracted"):
        os.mkdir(ROOT_FOLDER + "/FirstLinesExtracted")
        print("Created Folder for First Line extraction... >>> " + ROOT_FOLDER + "/Plots")
        
              
createNeededFolders()     

   



# Change path to your correct simp folder - /cglucose/simp/ - After that add a new Folder for the log files to be saved.
PATH_GLUC = os.path.join(ROOT_FOLDER + "/GlucoseData/")
PATH_BDD = os.path.join(ROOT_FOLDER + "/BDDsData/")

# Save the gotten data from the log files in single rows, for better Data analysis - These paths are used for the Data that you accumulated during the runtime!  BDD_Keywords.txt
SAVE_FILE_GLUC_KEYWORDS = os.path.join(ROOT_FOLDER + "/FirstLinesExtracted/Glucose_KeyWords.txt")
SAVE_FILE_BDD_KEYWORD = os.path.join(ROOT_FOLDER + "/FirstLinesExtracted/BDD_Keywords.txt")

# The keywords that are contained in the file
keywords = ['restarts:','conflicts:','decisions:','conflicLiterals:','blockedRestarts:','reducedDatabase:','propagations:']

# The name of your Log files
FILE_EXTENSION = "/rawData.txt"

# Location where you want your extracted info is
SAVE_FILE_GLUC = os.path.join(ROOT_FOLDER + "/FirstLinesExtracted/Glucose_ImportantInfoLine.txt")
print(SAVE_FILE_GLUC)
SAVE_FILE_BDD = os.path.join(ROOT_FOLDER + "/FirstLinesExtracted/BDD_ImportantInfoLine.txt")
print(SAVE_FILE_GLUC)

# Paths where you want to save the Plots for each Keyword >> ['restarts', 'conflicts', 'decisions', 'conflicLiterals', 'blockedRestarts', 'reducedDatabase', 'propagations']
# For each Keyword, a plot will be generated, to see the difference in both of the calculations
KEYWORDS_PLOT_SAVEPATH = os.path.join(ROOT_FOLDER + "/KEYWORDS_plots/")

# Path to see the mean value of the Data
MEAN_SAVEPATH = ROOT_FOLDER

FIGURE_SIZE = (10, 10)
GRID_COLOR = 'b'
MARKER = "."
LINE_WIDTH = 0.7
LINE_STYLE = "-"
CACTUSPLOT_MARKER = "x"

# Savepath for the 
SAVE_PATH_IMPINFOPLOTS = os.path.join(ROOT_FOLDER + "/Plots/")

files = os.listdir(PATH_TO_CNF_FILES)

# Path to the Glucose executable
executable_path = '/home/admin/Abschlussarbeit/CDCL-support-by-BDD-methods/cglucose/simp/glucose'

# For each file, a new Glucose instance will be started
def calc_files(startBDDs):
    index = 0
    for file in files:
        # If you want to work with the full file path, join it with the folder path
        file_path = os.path.join(PATH_TO_CNF_FILES, file)
        print(file_path)
        if os.path.isfile(file_path):
            # Process the file here - File path is the argument that is being given to Glucose -- If you have other arguments to glucose, you can add them here
            startGlucose(file_path, startBDDs)
            print("Saved file with index : " + str(index))
            index = index + 1
        

def startGlucose(path, startWithBDDs):
    try:
        arguments = [path,startWithBDDs]
        print(arguments)
        # Use subprocess.run to start the C++ executable
        result = subprocess.run([executable_path] + arguments)

        # Print the standard output and standard error of the C++ executable
        print("Standard Output:")
        print(result.stdout)

        print("Standard Error:")
        print(result.stderr)

    except subprocess.CalledProcessError as e:
        # If the C++ executable returns a non-zero exit code, you can handle the error here
        print(f"Error running {executable_path}: {e.returncode}")
    except FileNotFoundError:
        # Handle the case where the executable file is not found
        print(f"Executable not found: {executable_path}")


def startGluc_thenBDD():
    # First start files with BDDs
    calc_files("true")
    
    # Then with Glucose
    calc_files("false")


# RUNNING GLUCOSE HERE!
startGluc_thenBDD()


#####################################################################################
#                                                                                   #
#       After we got the files calculated, make the log files easier to handle      #
#                                                                                   #
#####################################################################################


folder_names_gluc = []
folder_names_bdd = []

def iterateFolders(empty_list, path):
    for foldername, subfolders, filenames in os.walk(path):
        empty_list.append(foldername)
        
iterateFolders(folder_names_gluc, PATH_GLUC)
iterateFolders(folder_names_bdd, PATH_BDD)

# Reads the first line of your log files, since it contains valuable Information for quick checks and createas the .txt file with them containing the filename as well 
def readFirstLineOfLogFile(saveFile, folder_names):
    with open(saveFile, 'w') as fl:
        files = 0
        for folder in folder_names:
            files += 1
            raw_path = folder + FILE_EXTENSION
            try:
                with open(raw_path, 'r') as file:
                    first_line = file.readline()
                    if first_line:
                        print("First line of the file:", first_line)
                        fl.write(folder + "\n")
                        fl.write(first_line + "\n")
                    else:
                        print("The file is empty.")
            except FileNotFoundError:
                print(f"File not found: {raw_path}")
            except Exception as e:
                print("An error occurred:", str(e))
        print(files)
        
        
readFirstLineOfLogFile(SAVE_FILE_GLUC, folder_names_gluc)
readFirstLineOfLogFile(SAVE_FILE_BDD, folder_names_bdd)


#################################################################################
#                                                                               #
#       Saving the other infos, specifly the last element of each Keyword       #
#                                                                               #
#################################################################################


# Since the first folder is the root folder, start from second Folder
folders_glucose = folder_names_gluc[1:]
folders_bdd = folder_names_bdd[1:]

# This Function takes the log files, and extracts the last Element for each Keyword -> The keywords that you saved in the log file
def get_other_info(folders, save_file_for_keywords):
    with open(save_file_for_keywords, 'w') as fl:
        for folder in folders:
            raw_path = folder + FILE_EXTENSION
            fl.write(folder + "\n")
            print(folder)
            with open(raw_path, 'r') as file:
                text = file.read()
                text.replace('\n','')
                split = text.split('\n')
                index = 0
                for line in split:
                    for keyword in keywords:
                        if(line == keyword):
                            nextLine = split[index+1]
                            elements = nextLine.split(',')
                            lastElem = 0
                            if(len(elements) > 1):
                                lastElem = int (elements[-1])
                            fl.write(keyword + str(lastElem) + " ")
                    index+=1
                fl.write('\n')
                
      
get_other_info(folders_glucose, SAVE_FILE_GLUC_KEYWORDS)
get_other_info(folders_bdd, SAVE_FILE_BDD_KEYWORD)



#############################################################################################
#                                                                                           #
#       The following script takes the created and parsed log files, and creates a Table,   # 
#       that contains the average accumulated data points.                                  #
#                                                                                           #
#       It also creates the plots from for the keywords log file.                           #
#                                                                                           #
#                                                                                           #
#############################################################################################


data_bdd = []
data_gluc = []

def extract_data(path, data):
    with open(path, 'r') as file:
        for line in file:
            if line.startswith('/'):
                segmented_line = line.strip()
                segmented_line = segmented_line.split('/')
                name_of_file = segmented_line[-1]
                data_line = file.readline()
                elements = data_line.split()
                result_list = []
                for element in elements:
                    key, value = element.split(':')
                    result_list.append((key, int(value)))
                data.append((name_of_file, result_list))


extract_data(SAVE_FILE_GLUC_KEYWORDS, data_gluc)
extract_data(SAVE_FILE_BDD_KEYWORD, data_bdd)

def clear(data_solver1, data_solver2):
    for (name, data_list) in data_solver1:
        current_tuple = (name, data_list)
        found = False
        for (name_2, data_list_2) in data_solver2:
            if name == name_2:
                found = True
                break
        if not found:
            data_solver1.remove((name, data_list))
            

def get_values_for_specific_term(data_keyword, data):
    extracted_values = []
    for (name, values) in data:
        for (key, value) in values:
            if key == data_keyword:
                extracted_values.append(value)
    return extracted_values


def get_average(values):
    average = 0
    for value in values:
        average += value

    return average // len(values)


average_per_keyword_bdd = []
average_per_keyword_gluc = []

plt.ioff()
plt.rcParams['lines.antialiased'] = True

def print_data(solver_type, data, save_list):
    print('\n' + '##############################################')
    print('Average values for saved Data... [' + solver_type + ']')
    print()
    keywords = ['restarts', 'conflicts', 'decisions', 'conflicLiterals', 'blockedRestarts', 'reducedDatabase', 'propagations']

    for keyword in keywords:
        values_for_keyword = get_values_for_specific_term(keyword, data)
        save_list.append(str(get_average(values_for_keyword)))
        print(keyword + ': ' + str(get_average(values_for_keyword)))
    print('##############################################' + '\n')


print_data('BDD', data_bdd,average_per_keyword_bdd)
print_data('Glucose', data_gluc,average_per_keyword_gluc)


def create_table():
    column_labels = ['BDD+Glucose', 'Glucose']
    row_labels = ['Restarts', 'Conflicts', 'Decisions', 'Conflict Literals',
                  'Blocked Restarts', 'Reduced Databases', 'Propagations']

    table_data = []
    index = 0
    while index < len(average_per_keyword_gluc):
        key_bdd = int(average_per_keyword_bdd[index])
        formatted_bdd = "{:,}".format(key_bdd)
        key_gluc = int(average_per_keyword_gluc[index])
        formatted_gluc = "{:,}".format(key_gluc)
        table_data.append((formatted_bdd, formatted_gluc))
        index += 1

    col_widths = [1, 1]
    fig, ax = plt.subplots()
    table = ax.table(cellText=table_data,
                     colLabels=column_labels,
                     rowLabels=row_labels,
                     loc='left',
                     colWidths=col_widths)
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    table.scale(1.2, 1.2)
    fig.subplots_adjust(left=0.73)
    ax.axis('off')
    plt.savefig(ROOT_FOLDER + "/MeanValues.png")
    plt.close()

create_table()

# Plot the data, cause some files can have absurd amount of propagations, decisions and so on...
def plot_keywords(data_solvertype1, data_solvertype2):
    keywords = ['restarts', 'conflicts', 'decisions', 'conflicLiterals', 'blockedRestarts', 'reducedDatabase', 'propagations']

    prettier = ['Restarts', 'Conflicts', 'Decisions', 'Conflict Literals',
                  'Blocked Restarts', 'Reduced Databases', 'Propagations']

    keyword_data_list_solver1 = []
    keyword_data_list_solver2 = []

    for keyword in keywords:
        keyword_data_list_solver1.append((keyword, get_values_for_specific_term(keyword, data_solvertype1)))
        keyword_data_list_solver2.append((keyword, get_values_for_specific_term(keyword, data_solvertype2)))


    # We have the data, now plot it
    i = 0
    while i < len(keywords):
        keyword = prettier[i]
        data_solver1 = keyword_data_list_solver1[i]
        data_solver2 = keyword_data_list_solver2[i]

        x_values_solver1 = list(range(0, len(data_solver1[1])))
        y_values_solver1 = sorted(data_solver1[1])

        x_values_solver2 = list(range(0, len(data_solver2[1])))
        y_values_solver2 = sorted(data_solver2[1])

        fig, ax = plt.subplots(figsize=(8, 6))
        plt.plot(x_values_solver1, y_values_solver1, label='Data', color='blue')
        plt.plot(x_values_solver2, y_values_solver2, label='Data', color='green')
        plt.xlabel('Solved Files')
        plt.ylabel('Anzahl an ' + keyword, labelpad=10)
        plt.title(keyword)
        plt.savefig(KEYWORDS_PLOT_SAVEPATH + keyword + '.png')
        plt.close()
        i += 1


plot_keywords(data_bdd, data_gluc)


#############################################################################################################
#                                                                                                           #
#       The following lines are responsible for plotting the Data and to ensure the Data is correct.        #
#       It takes the parsed first Lines from the previous extraction and plots the corresponding            #
#       data to understandable Plots.                                                                       #
#                                                                                                           #
#############################################################################################################



#########################################################################################################
#                                                                                                       #
#       To use this file, run your tests on Glucose. After you have gotten some Data                    #
#       from both of your achitectures that you want to test, change the Paths to your corresponding    #
#       paths, or make sure your log files have the same style.                                         #
#                                                                                                       #
#########################################################################################################

#Takes the Values and saves them internally
def extract_from_File(path, emptyList):
    # Open BDD file and extract the Information
    try:
        with open(path, 'r') as file:
            for line in file:
                if line.startswith('/'):
                    fileName = line.rstrip('\n').split('/')
                    correctFileName = fileName[-1]
                    values = file.readline().rstrip('\n')
                    #since it is a string value, form them into a list of int values
                    correctValues = (eval(values))
                    emptyList.append((correctFileName,correctValues))
        file.close()
    except FileNotFoundError:
        print(f"File not found: {path}")
    except Exception as e:
        print(f"An error occurred: {str(e)}")

    return 0


#sorting files in specific categories for easier analysis
def sortFilesInCategories(list_of_tuples):
    sat_files_with_time = []
    unsat_files_with_time = []
    indeterm_files_with_time = []

    sat_value = 0
    unsat_value = 0
    indeterminate_value = 0
    complete_list = []

    # Get total amount of results
    for (name, value) in list_of_tuples:
        result_value = value[-1]
        time_value = value[-2]
        if result_value == 'SAT':
            sat_value += 1
            sat_files_with_time.append((name, time_value, result_value))
        if result_value == 'UNSAT':
            unsat_value += 1
            unsat_files_with_time.append((name, time_value, result_value))
            # since the cpu usage may vary in some cases, set the time value to 900 when the formula has returned indeterminate
        if result_value == 'indeterminate':
            indeterminate_value += 1
            indeterm_files_with_time.append((name,900.0, result_value))
    complete_list.append(sat_files_with_time)
    complete_list.append(unsat_files_with_time)
    complete_list.append(indeterm_files_with_time)
    return(sat_value, unsat_value, indeterminate_value, sat_files_with_time,unsat_files_with_time,indeterm_files_with_time, complete_list)


# since it doesn't matter in which order the files have been calculated, sort them from lowest time to longest
def sortTuplesByTime(completeList, sat_list, unsat_list, indeterm_list):
    one_list = [item for sublist in completeList for item in sublist]
    full_sorted_list = sorted(one_list, key=lambda x: x[1])

    sort_sat = sorted(sat_list, key=lambda x: x[1])
    unsat_sort = sorted(unsat_list, key=lambda x: x[1])
    indeterm_sort = sorted(indeterm_list, key=lambda x: x[1])

    return (full_sorted_list, sort_sat,unsat_sort,indeterm_sort)
    

# returns the total time needed
def getTimeNeeded(list):
    total_time = 0.0
    for (name,time,result) in list:
        total_time += time
    return total_time

# Method to create the bar chart
def barChart(list, extensionname, savePath):
    categories = ['SAT', 'UNSAT', 'INDETERMINATE']
    sat = 0
    unsat = 0
    indeterm = 0

    # Iterateover BDD list
    for tuple in list:
        result_value = tuple[2]
        if result_value == 'SAT':
            sat += 1
        if result_value == 'UNSAT':
            unsat += 1
        if result_value == 'indeterminate':
            indeterm += 1

    values = [sat,unsat,indeterm]

    plt.bar(categories, values)

    for i in range(len(categories)):
        plt.text(categories[i], values[i], str(values[i]), ha='center', va='bottom')

    plt.ylabel('Amount')
    plt.title('Bar Chart hard combinatorial')

    s2 = "Total time needed to calculate: " + str(int(getTimeNeeded(list))//60) + " in minutes"
    
    plt.title(s2)
    plt.savefig(savePath + 'results_' + extensionname + '.png')
    plt.close()



# Plot the Bdds to Gluc in one Graph
def cactusPlot (list_bdd, list_gluc, savePath):
    data_points_bdd = [i for i in range(len(list_bdd))]
    data_points_gluc = [i for i in range(len(list_gluc))]

    yTime_values_bdd = []
    yTime_values_gluc = []

    for (name,time,result) in list_bdd:
        yTime_values_bdd.append(time)

    for (name,time,result) in list_gluc:
        yTime_values_gluc.append(time)


    plt.figure(figsize= FIGURE_SIZE)
    plt.plot(data_points_bdd, yTime_values_bdd, marker = '+', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.plot(data_points_gluc, yTime_values_gluc, marker = 'x', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color='green')
    plt.ylabel('Time Taken in sec')
    plt.xlabel('Solved Instances')
    plt.text(-50, 1080, "Blue: Glucose+BDDs", fontsize=12, verticalalignment="top", horizontalalignment="left",color="blue")
    plt.text(-50, 1040, "Green: Glucose", fontsize=12, verticalalignment="top", horizontalalignment="left", color="green")
    #plt.xlim([-10, 126])
    #plt.xticks([0, 25,50,75,100,125,150,175,200,225,250])
    plt.yticks(list(range(0, 1000, 100)))
    plt.savefig(savePath + "cactus_plot" + '.png')
    plt.close()



# Method to plot the sat and unsat files on each other, to see the difference in both of them
def plot_satUnsat_files(bdd_list, gluc_list, savePath):
    sat_files = []
    unsat_files = []

    concat_list_sat = []
    concat_list_unsat = []


    for (name,time,result) in gluc_list:
        if result == 'SAT':
            sat_files.append((name,time,result))

    for (name,time,result) in bdd_list:
        if result == 'UNSAT':
            unsat_files.append((name,time,result))

    for (name,time,result) in sat_files:
        for (name_bdd,time_bdd,result_bdd) in bdd_list:
            if name == name_bdd:
                concat_list_sat.append((name, time, time_bdd))
                break

    for (name,time,result) in unsat_files:
        for (name_gluc,time_gluc,result_gluc) in gluc_list:
            if name == name_gluc:
                concat_list_unsat.append((name, time_gluc, time))
                break
    
    sorted(concat_list_sat, key=lambda x: x[1])
    sorted(concat_list_unsat, key=lambda x: x[2])
    
    x_points_sat = list(range(0,len(concat_list_sat)))
    y_values_gluc = []
    y_values_bdd = []

    for tuple in concat_list_sat:
        y_values_gluc.append(tuple[1])
        y_values_bdd.append(tuple[2])

    plt.plot(x_points_sat, y_values_bdd, marker = '+', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.plot(x_points_sat, y_values_gluc, marker = 'x', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color='green')
    plt.ylabel('Time Taken in sec')
    plt.xlabel('Solved Instance')
    plt.title('SAT Files Time comparison')
    plt.text(-10, 1080, "Blue: Glucose+BDDs", fontsize=12, verticalalignment="top", horizontalalignment="left",color="blue")
    plt.text(-10, 1040, "Green: Glucose", fontsize=12, verticalalignment="top", horizontalalignment="left", color="green")
    plt.savefig(savePath + "cactus_plot_sat_fils" + '.png')
    plt.close()

    x_points_unsat = list(range(0,len(concat_list_unsat)))
    y_values_gluc_unsat = []
    y_values_bdd_unsat = []

    for tuple in concat_list_unsat:
        y_values_gluc_unsat.append(tuple[1])
        y_values_bdd_unsat.append(tuple[2])

    plt.plot(x_points_unsat, y_values_bdd_unsat, marker = '+', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.plot(x_points_unsat, y_values_gluc_unsat, marker = 'x', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color='green')
    plt.ylabel('Time Taken in sec')
    plt.xlabel('Solved Instance')
    plt.title('UNSAT Files Time comparison')
    plt.text(-10, 1080, "Blue: Glucose+BDDs", fontsize=12, verticalalignment="top", horizontalalignment="left",color="blue")
    plt.text(-10, 1040, "Green: Glucose", fontsize=12, verticalalignment="top", horizontalalignment="left", color="green")
    plt.savefig(savePath + "cactus_plot_unsat_fils" + '.png')
    plt.close()
    
    
def addTimeTogether(list):
    totalTime = 0.0
    for time in list:
        totalTime+=time

    return totalTime


# Plot the files, so on the x axis the Datapoints are for the same Formula. Shows the time difference for each file
def cactus_plot_version_2(list_, savePath):
    x_dataPoints = list(range(0,len(list_)))
    print(len(list_))
    plt.figure(figsize= FIGURE_SIZE)
    bdd_time = []
    gluc_time = []

    for tuple in list_:
        if tuple[1] > 900.00:
            bdd_time.append(900)
        else:
            bdd_time.append(tuple[1])

        if tuple[2] > 900.00:
            gluc_time.append(900.00)
        else:
            gluc_time.append(tuple[2])

    plt.plot(x_dataPoints, bdd_time, marker = '+', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color=GRID_COLOR)
    plt.scatter(x_dataPoints, gluc_time, marker = 'x', linewidth=LINE_WIDTH, linestyle=LINE_STYLE, color='green')
    plt.ylabel('Time Taken in sec')
    plt.xlabel('Solved Instance')
    plt.text(-50, 1080, "Blue: Glucose+BDDs", fontsize=12, verticalalignment="top", horizontalalignment="left",color="blue")
    plt.text(-50, 1040, "Green: Glucose", fontsize=12, verticalalignment="top", horizontalalignment="left", color="green")
    plt.yticks(list(range(0, 1000, 100)))
    plt.savefig(savePath + "cactus_plot_new" + '.png')
    plt.close()
    
def corresponding_name(list_bdd, list_gluc):
    for_same_name = []

    for (name_bdd,time_bdd,result_bdd) in list_bdd:
        found = False
        for(name_gluc,time_gluc,result_gluc) in list_gluc:
            if name_bdd == name_gluc:
                if((name_bdd,time_bdd,result_bdd) not in for_same_name):
                    for_same_name.append((name_bdd, time_bdd, time_gluc))
                    found = True
                    break
        if not found:
            print(name_bdd)
    return for_same_name
    

def makePlots(pathbdd, pathgluc, savepath):
    #Lists to store the information from glucose and BDD + Glucose
    tuples_bdd_results = []
    tuples_glucose = [] 

    # Get the Data from the generated file after Execution
    extract_from_File(pathbdd, tuples_bdd_results)
    extract_from_File(pathgluc, tuples_glucose)

    # Get the amount of Sat, unsat and indeterminate results for BDD
    (sat_amount,
     unsat_amount,
     indeterminate_amount,
     sat_files_with_time,
     unsat_files_with_time,
     indeterm_files_with_time,
     completeList
    ) = sortFilesInCategories(tuples_bdd_results)


    # For Glucose
    (sat_amount_gluc,
     unsat_amount_gluc,
     indeterminate_amount_gluc,
     sat_files_with_time_gluc,
     unsat_files_with_time_gluc,
     indeterm_files_with_time_gluc,
     completeList_gluc
    ) = sortFilesInCategories(tuples_glucose)
    (full_sorted_list, sort_sat,unsat_sort,indeterm_sort) = sortTuplesByTime(completeList,sat_files_with_time,unsat_files_with_time,indeterm_files_with_time)
    (full_sorted_list_gluc, sort_sat_gluc,unsat_sort_gluc,indeterm_sort_gluc) = sortTuplesByTime(completeList_gluc,sat_files_with_time_gluc,unsat_files_with_time_gluc,indeterm_files_with_time_gluc)


    # barChart(sat_amount_gluc,unsat_amount_gluc,indeterminate_amount_gluc, full_sorted_list_gluc, 'Glucose')
    barChart(full_sorted_list, 'BDD', savepath)
    barChart(full_sorted_list_gluc, 'Glucose', savepath)

    #create the cactusplot
    cactusPlot(full_sorted_list,full_sorted_list_gluc, savepath)

    #plot the sat and unsat files
    plot_satUnsat_files(full_sorted_list,full_sorted_list_gluc, savepath)

    # Plot for each file the same file from the other Solver on same x-value
    parsed_list_for_cactus = corresponding_name(full_sorted_list, full_sorted_list_gluc)
    cactus_plot_version_2(parsed_list_for_cactus, savepath)


makePlots(SAVE_FILE_BDD, SAVE_FILE_GLUC, SAVE_PATH_IMPINFOPLOTS)