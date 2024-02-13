CREATE TABLE fishnet (
    time TIMESTAMPTZ NOT NULL,
    user_acquired INT,
    user_queued INT,
    user_oldest INT,
    system_acquired INT,
    system_queued INT,
    system_oldest INT
);

SELECT create_hypertable('fishnet', by_range('time'));