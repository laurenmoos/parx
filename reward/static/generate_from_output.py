import base64
import json
import os
import time

file_path = 'output.txt'  # Replace with the path to your text file


def try_parse_action(line):
    return line.replace("NAV->Tracer:  ", '')


def transform_and_clean(trace):
    return json.dumps(trace)


def format_episode(episode):
    formatted_strings = []

    for item in episode:
        formatted_strings.extend(item.split('\n'))

    # Now join these formatted strings with double line breaks
    return '\n\n'.join(formatted_strings)


def try_load_json(row):
    try:
        # this is necessary to make it valid json
        clean_json = row.replace('\'', '\"').replace('None', "0")
        row = json.loads(clean_json)
        return row
        # Process the loaded JSON data
        # ...
        # If loading or processing fails, an exception will be raised
        # and the code execution will jump to the except block below.
    except (json.JSONDecodeError, FileNotFoundError):
        pass  # Skip to the next iteration if JSON loading fails or file not found


with open(file_path, 'r') as file:
    lines = file.readlines()
    traces = []
    episodes = []
    action = None
    for i, line in enumerate(lines):
        if line.startswith('Episode'):
            if traces:
                episodes.append(traces)
                traces = []
            continue
        if line.startswith('NAV->Tracer:'):
            action = try_parse_action(line)
        else:
            check = try_load_json(line)
            trace = ''
            if action and check:
                trace += action
                trace += transform_and_clean(check)
                traces.append(trace)
    episodes.append(traces)
    # xs = [unpack_checkpoints(trace) for trace in traces]
    # Usage
    directory = "/tmp/"  # Current directory
    base_filename = time.strftime('%Y%m%d%H%M%S')

    for i, episode in enumerate(episodes):
        file_name = os.path.join(directory, f"{base_filename}_{i}.txt")
        with open(file_name, 'w') as f:
            f.write(format_episode(episode))
