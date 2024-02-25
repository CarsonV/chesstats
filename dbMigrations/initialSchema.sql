CREATE TABLE fishnet (
    time TIMESTAMPTZ NOT NULL,
    user_acquired INT NOT NULL,
    user_queued INT NOT NULL,
    user_oldest INT NOT NULL,
    system_acquired INT NOT NULL,
    system_queued INT NOT NULL,
    system_oldest INT NOT NULL
);

SELECT create_hypertable('fishnet', by_range('time'));