import os
import re
import time
import sqlite3
import subprocess
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

_FAIRLADY_CONSTANTS_FILE_PATH = os.path.abspath("./commom/constants.rs")
_PLOTS_OUTPUT_FOLDER = os.path.abspath("./docs")
_TELEMETRY_CYCLES = 30
_FILE_SIZES = [0.1, 0.5, 1, 2, 3, 4]

def parse_constants():
    with open(_FAIRLADY_CONSTANTS_FILE_PATH) as file:
        constants_declarations = file.readlines()
    constants = {}
    pattern = r'([A-Z_]+).*?"([^"]*)"'
    for line in constants_declarations:
        clean_line = line.replace('\n', '').strip()
        match = re.search(pattern, clean_line)
        if match:
            key = match.group(1)     
            value = match.group(2)   
            constants[key] = value
    return constants

def connect_to_database():
    DB_PATH = constants['SYSTEM_DATABASE_PATH']
    try:
        connection = sqlite3.connect(DB_PATH)
        cursor = connection.cursor()
        print(f"Successfully connected to SQLite database at: {DB_PATH}")
        
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
        
        # Converted map to list immediately so it can be safely used multiple times
        return {
            'tables': [x[0] for x in cursor.fetchall()],
            'connection': connection
        }
        
    except sqlite3.Error as e:
        print(f"Database error: {e}")

def get_performance_points(connection):
    cursor = connection.cursor()
    cursor.execute("SELECT * FROM perf_points;")
    
    rows = cursor.fetchall()
    column_names = [description[0] for description in cursor.description]
    
    perf_points_list = []
    for row in rows:
        row_dict = dict(zip(column_names, row))
        perf_points_list.append(row_dict)
        
    return perf_points_list

def strategies_telemetry(points):
    counter = {
        'aes': 0,
        'chacha': 0,
        'twofish': 0,
        'serpent': 0
    }
    for point in points:
        if point['strategy'] == 'aes': counter['aes']+=1
        if point['strategy'] == 'chacha': counter['chacha']+=1
        if point['strategy'] == 'serpent': counter['serpent']+=1
        if point['strategy'] == 'twofish': counter['twofish']+=1
    return counter

def preprocess_telemetry_dataframe(dataframe):
    preprocessed_df = dataframe.copy()
    preprocessed_df['execution_time_ns'] = (preprocessed_df['final_timestamp'] - preprocessed_df['init_timestamp'])
    return preprocessed_df

def plot_telemetry_data(data):
    df = pd.DataFrame(data)
    processed_df = preprocess_telemetry_dataframe(df)
    plot_encryption_performance(processed_df)

def plot_encryption_performance(preprocessed_dataframe):
    _, ax = plt.subplots(figsize=(10, 6))
    
    # Group by strategy and plot each
    for strategy, group in preprocessed_dataframe.groupby('strategy'):
        ax.scatter(
            group['payload_size'], 
            group['execution_time_ns'], 
            label=strategy, 
            alpha=0.8, 
            s=100, # Marker size
            edgecolors='none'
        )
        
    # Labels and Title
    ax.set_title('Encryption Performance: Payload Size vs. Execution Time', fontsize=14, pad=15)
    ax.set_xlabel('Payload Size (Bytes)', fontsize=12)
    ax.set_ylabel('Execution Time (Nanoseconds)', fontsize=12)
    
    # Format axes to use normal numbers with commas instead of scientific notation (e.g., 100,000,000 instead of 1e8)
    ax.xaxis.set_major_formatter(ticker.StrMethodFormatter('{x:,.0f}'))
    ax.yaxis.set_major_formatter(ticker.StrMethodFormatter('{x:,.0f}'))
    
    # Aesthetics
    ax.legend(title='Strategy', fontsize=11, title_fontsize=12)
    ax.grid(True, linestyle='--', alpha=0.5)
    plt.tight_layout()
    
    # Save the output image
    plt.savefig(os.path.join(_PLOTS_OUTPUT_FOLDER, 'encryption_performance_ns.png'), dpi=300)
    print("Encryption scatter plot successfully generated and saved as 'encryption_performance_ns.png'.")

def main():
    global constants
    constants = parse_constants()
    db = connect_to_database()
    
    if not db:
        raise Exception("Could not proceed without a database connection.")
    if 'perf_points' not in db['tables']:
        raise Exception("FAILED: did not find any perf_points table")
    
    custom_env = os.environ.copy()
    for i in range(_TELEMETRY_CYCLES):
        print()
        print(f"\033[94mCYCLE: {i+1}/{_TELEMETRY_CYCLES}\033[0m - ", end="")
        custom_env['FILE_SIZE_GB'] = str(_FILE_SIZES[i%len(_FILE_SIZES)])
        try:
            # Run the script and check for errors
            subprocess.run(
                "./generate_data.sh",
                env=custom_env,    # Passes the FILE_SIZE_GB variable
                check=True,         # Throws Python error if the Bash script exits with an error code
                text=True           # Ensures stdout/stderr are read as strings if captured
            )
        except subprocess.CalledProcessError as e:
            raise Exception(f"Data generation shell script crashed! Exit code: {e.returncode}")
        except FileNotFoundError:
            raise Exception("The script './generate_data.sh' was not found or is not executable.")
        sleep_time = int(_FILE_SIZES[i%len(_FILE_SIZES)]*20)
        sleep_time = sleep_time if sleep_time > 40 else 40
        print(f"Letting fairlady react for {sleep_time}s")
        time.sleep(sleep_time)

    print("=============================================")
    print()
    performance_points = get_performance_points(db['connection'])
    print(performance_points[0])
    print(strategies_telemetry(performance_points))
    plot_telemetry_data(performance_points)
    db['connection'].close()
    print("So Long, and Thanks for All the Fish!")

main()