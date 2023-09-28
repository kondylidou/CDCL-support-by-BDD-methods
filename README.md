# Project README

This README provides instructions for installing the necessary dependencies for the project on a Linux system. The project requires Rust and Python, along with specific development libraries. 
It contains instructions on how to build and run the project. Follow these steps to execute the project successfully.

# Installation Instructions

Please follow the steps below to set up the required environment on your Linux system:

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Rust Build Essentials
sudo apt-get install build-essential

# 3. Install Python 3.10
sudo apt-get install python3.10

# 4. Install Python Development Tools
sudo apt-get install python3-dev
```

## Verify Installation

To verify that all the necessary components have been installed correctly, you can run the following commands:
```bash
rustc --version
python3 --version
```

# Execution Instructions

## Prerequisites

Before running the project, make sure you have the following prerequisites installed on your system:

- Rust: Install Rust and its build tools by following the instructions mentioned in above.
- `cargo`: Rust's package manager, should be available after installing Rust.

## Building the Project

1. Open a terminal and navigate to the project's main folder.

2. Build the Rust code using `cargo`:

```bash
cargo build --release
```

## Updating Library Paths

You will need to update library paths in specific C++ source files. Follow these steps:

1. Open the Main.cc file located in the cglucose/simp folder.

2. Locate the line where the path to the Rust library (librust_lib.so) is specified. It might look like this:

```bash
// Change this path to the actual path of librust.so
void* rust_lib = dlopen("/home/lkondylidou/Desktop/PhD/CDCL-support-by-BDD-methods/target/release/librust_lib.so", RTLD_LAZY); // Update the path accordingly
```
Replace /path/to/target/release/librust.so with the actual path to librust.so on your system.

3. Save the changes to Main.cc.

4. Similarly, open the Solver.cc file located in the cglucose/core folder and update the path to librust_lib.so as done in step 2.

## Building the C++ Component

1. Navigate to the cglucose/simp folder in your terminal:

```bash
cd cglucose/simp
```

2. Compile the C++ code:

```bash
make clean
make
```

##Running the Project

1. After successfully building the C++ component, execute the project:

```bash
./glucose r
```


# CDCL support by BDD methods
The projects' second phase is to use the BDD library as pre-/inprocessing in order to support the CDCL process and improve the results already acquired from phase one of this project.

# Master's Thesis

The initial project was developed in terms of a Master's thesis. Goal of the Master's thesis was to develop a BDD library and run it in parallel with the Glucose SAT solver. 

## Project Description

In this project a Bdd solver is used to enhance the performance of a CDCL Solver, Glucose.

## Language and Communication

Glucose is implemented in C and the Bdd library in Rust. Bindings are implemented to make the connection between the two architectures and unsafe Rust was used to be able to send and receive data to and from the Solver.

## Phase one - Statistics

| **INSTANCES**     | **1**    | **2**    | **3**     | **4**    | **5**    | **6**    | **7**     | **8**    | **9**   | **10**  | **11**  | **12**   | **13**  | **14**  | **15**  | **16**    | **17**    | **18**    | **19**    | **20**    | **21**     | **22**     |
| ----------------- | -------- | -------- | --------- | -------- | -------- | -------- | --------- | -------- | ------- | ------- | ------- | -------- | ------- | ------- | ------- | --------- | --------- | --------- | --------- | --------- | ---------- | ---------- |
| **Conflicts**     |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Glucose & Bdd** | 815021   | 3349555  | 2828465   | 1930235  | 2750230  | 2617831  | 868478    | 381915   | 371880  | 108491  | 212508  | 1431773  | 139598  | 66490   | 63138   | 1457250   | 3905203   | 1026505   | 25073     | 2775146   | 1613544    | 5672671    |
| **Glucose**       | 993400   | 3721064  | 2568174   | 4000059  |          |          | 958372    | 436589   | 433332  | 117719  | 262445  |          | 144749  | 73052   | 63959   | 3574234   | 4563188   | 1053133   | 359473    | 3957170   | 1504243    | 8156999    |
|                   |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Restarts**      |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Glucose & Bdd** | 181      | 2402     | 1003      | 8323     | 3540     | 3426     | 1848      | 1091     | 450     | 3       | 6       | 2466     | 33      | 240     | 251     | 3569      | 13763     | 1455      | 99        | 5212      | 2903       | 14637      |
| **Glucose**       | 403      | 2442     | 1419      | 15560    |          |          | 1975      | 1363     | 517     | 16      | 44      |          | 38      | 258     | 237     | 11330     | 15610     | 1463      | 1131      | 6871      | 2685       | 25987      |
|                   |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Decisions**     |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Glucose & Bdd** | 921199   | 3831292  | 3167460   | 7112733  | 329853   | 3162192  | 996270    | 454538   | 426644  | 124282  | 236851  | 1681017  | 161032  | 140290  | 141812  | 5417599   | 9538999   | 1400259   | 113484    | 3702842   | 2493479    | 8352456    |
| **Glucose**       | 1154557  | 4250647  | 2891036   | 14163981 |          |          | 1046344   | 477882   | 496668  | 134822  | 294032  |          | 167169  | 154852  | 138143  | 8433076   | 9830124   | 1425159   | 1009042   | 5177071   | 2330413    | 10867359   |
|                   |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Propagations**  |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Glucose & Bdd** | 13166813 | 50402234 | 128173479 | 34879173 | 73253571 | 72081669 | 216373277 | 89185043 | 6539937 | 4471824 | 3750738 | 31199705 | 5741075 | 2320187 | 2214369 | 136380284 | 233523000 | 232795077 | 10265517  | 545751889 | 2556104275 | 1413781175 |
| **Glucose**       | 17345146 | 57559751 | 117390764 | 73783304 |          |          | 234791125 | 93719721 | 7618381 | 4814367 | 4650756 |          | 5942285 | 2595344 | 2199866 | 211221581 | 287389734 | 246886506 | 152522909 | 790686819 | 2396361500 | 1762124974 |
|                   |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Time CPU**      |          |          |           |          |          |          |           |          |         |         |         |          |         |         |         |           |           |           |           |           |            |            |
| **Glucose & Bdd** | 80.3048  | 758.65   | 1320.23   | 831.531  | 616.2867 | 513.216  | 363.681   | 111.699  | 108.102 | 6.8004  | 7.5187  | 718.229  | 10.6695 | 2.30239 | 1.99624 | 188.334   | 440.765   | 597.611   | 6.75345   | 453.81    | 973.12     | 680.69     |
| **Glucose**       | 91.188   | 805.638  | 1195.44   | 1051.37  |          |          | 386.621   | 132.908  | 124.701 | 7.53427 | 28.6805 |          | 11.0598 | 2.40416 | 2.05326 | 374.874   | 474.788   | 579.587   | 59.1897   | 825.31    | 832.89     | 790.32     |

## Benchmarks and Testing
Benchmarks from the SAT competition were used from years 2008 until 2013.
