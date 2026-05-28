import os
import re
import sqlite3
import matplotlib

_FAIRLADY_CONSTANTS_FILE_PATH = os.path.abspath("./commom/constants.rs")

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

def main():
    global constants
    constants = parse_constants()
    db = connect_to_database()
    
    if not db:
        raise Exception("Could not proceed without a database connection.")
    if 'perf_points' not in db['tables']:
        raise Exception("FAILED: did not find any perf_points table")
    
    performance_points = get_performance_points(db['connection'])
    for i in performance_points:
        print(i)
    print(strategies_telemetry(performance_points))

    db['connection'].close()

main()