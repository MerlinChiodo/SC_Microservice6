CREATE TABLE Sessions (
    id SERIAL PRIMARY KEY,
    user_id BIGINT UNSIGNED NOT NULL,
    token VARCHAR(1000) NOT NULL ,
    expires DATETIME NOT NULL ,

    FOREIGN KEY (user_id)
                      REFERENCES Users(id)
                      ON DELETE CASCADE
);