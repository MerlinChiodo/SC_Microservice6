CREATE TABLE EmployeeLogins (
    id SERIAL PRIMARY KEY,
    info_id BIGINT UNSIGNED NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    hash VARCHAR(1000) NOT NULL,

    FOREIGN KEY (info_id)
                            REFERENCES EmployeeInfo(id)
                            ON DELETE CASCADE
)