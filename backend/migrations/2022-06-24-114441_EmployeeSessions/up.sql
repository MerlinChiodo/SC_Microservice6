CREATE TABLE EmployeeSessions(
    id SERIAL PRIMARY KEY,
    e_id BIGINT UNSIGNED NOT NULL,
    token VARCHAR(1000) NOT NULL,
    expires DATETIME NOT NULL,
    FOREIGN KEY (e_id)
                             REFERENCES EmployeeLogins(id)
                             ON DELETE CASCADE
)