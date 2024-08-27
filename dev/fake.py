import random
import sys
from faker import Faker
import sqlite3

fake = Faker(["en_IN", "it_IT", "en_US", "ja_JP"])


def main():
    path = "./test-random.db"
    n_samples = 10000
    table_names = {
        "Users": ["userId Integer", "age Integer", "salary Integer"],
        "Medical": ["patientId Integer", "bloodPressure Integer", "weight Integer"],
    }
    data = []
    try:
        n_samples = int(sys.argv[1])
        path = sys.argv[2]
    except (IndexError, ValueError):
        n_samples = 10000

    for table, columns in table_names.items():
        conn = sqlite3.connect(path)
        conn.execute(f"DROP TABLE IF EXISTS {table}")
        columns = f"({', '.join(columns)})"
        print(f"CREATE TABLE {table} {columns}")
        conn.execute(f"CREATE TABLE {table} {columns}")
        conn.commit()

    print(f"Using {n_samples} samples")

    for _ in range(n_samples):
        data.extend(
            [
                (
                    fake.iana_id(),
                    random.randint(5, 80),
                    random.randint(10_000, 100_00_000),
                )
            ]
        )

    conn.executemany(f"INSERT INTO Users VALUES (?, ?, ?)", data)
    conn.commit()
    data = []
    for _ in range(n_samples):
        data.extend(
            [(fake.iana_id(), random.randint(120, 200), random.randint(40, 120))]
        )
    conn.executemany(f"INSERT INTO Medical VALUES (?, ?, ?)", data)
    conn.commit()
    conn.close()


if __name__ == "__main__":
    main()
