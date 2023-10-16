import sqlite3
import json
con = sqlite3.connect("./demo.db")
cur = con.cursor()
pid = 10
test_name = f"test_{pid}"
# cur.execute(f"INSERT INTO tests VALUES({pid}, {test_name})")
for res in cur.execute("SELECT * from tests"):
    print(res)
res = cur.execute("SELECT * FROM tests WHERE uuid = (SELECT MAX(uuid) from tests)")
last = res.fetchone()[1]
print(last)
# epoch = 0
# episode = 0
# test = 1
# data = [
#     (0, 0, 1, json.dumps({"epoch": 0, "episode": 0, "test": 0, "data": [{"command":1, "req_crc":0}]})),
#     (0, 0, 2, json.dumps({"epoch": 0, "episode": 0, "test": 0, "data": [{"command":1, "req_crc":0}]})),
# ]
# cur.executemany(f"INSERT INTO {test_name} VALUES(?, ?, ?, ?)", data)
# con.commit()
# test_name = "test_1223612"
for row in cur.execute(f"SELECT * FROM {last} WHERE episode = 1 ORDER BY test ASC"):
    print(row)
