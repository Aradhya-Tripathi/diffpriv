import random
import sys
from faker import Faker
import sqlite3

fake = Faker(["en_IN", "it_IT", "en_US", "ja_JP"])


def main():
    path = "./test-random.db"
    n_samples = 10000
    table_name = "Users"
    data = []
    try:
        n_samples = int(sys.argv[1])
        path = sys.argv[2]
    except (IndexError, ValueError):
        n_samples = 10000

    conn = sqlite3.connect(path)
    conn.execute(f"DROP TABLE IF EXISTS {table_name}")
    conn.execute(
        f"CREATE TABLE {table_name} (name TEXT, age Integer, company TEXT, salary Integer)"
    )
    conn.commit()

    print(f"Using {n_samples} samples")

    for _ in range(n_samples):
        data.extend(
            [
                (
                    fake.name(),
                    random.randint(5, 80),
                    fake.company(),
                    random.randint(10_000, 100_00_000),
                )
            ]
        )

    conn.executemany(f"INSERT INTO {table_name} VALUES (?, ?, ?, ?)", data)
    conn.commit()
    conn.close()


if __name__ == "__main__":
    main()
