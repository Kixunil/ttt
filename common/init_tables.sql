CREATE TABLE IF NOT EXISTS vms (
    name TEXT NOT NULL PRIMARY KEY,
    group_id UNSIGNED INT,
    FOREIGN KEY (group_id) REFERENCES groups(id)
);

CREATE TABLE IF NOT EXISTS groups (
    id UNSIGNED INT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS intervals (
    vm TEXT NOT NULL,
    begin UNSIGNED INT NOT NULL,
    duration UNSIGNED INT NOT NULL,
    PRIMARY KEY (begin),
    FOREIGN KEY (vm) REFERENCES vms(name)
);

CREATE TABLE IF NOT EXISTS current_vm (
    vm TEXT NOT NULL,
    begin UNSIGNED INT NOT NULL,
    dummy UNSIGNED INT PRIMARY KEY CHECK (dummy = 0),
    FOREIGN KEY (vm) REFERENCES vms(name)
);

CREATE TABLE IF NOT EXISTS period_definitions (
    id UNSIGNED INT NOT NULL PRIMARY KEY,
    duration UNSIGNED INT NOT NULL
);

CREATE TABLE IF NOT EXISTS period_records (
    id UNSIGNED INT NOT NULL PRIMARY KEY,
    period_definition_id UNSIGNED INT NOT NULL,
    begin UNSIGNED INT NOT NULL,
    FOREIGN KEY (period_definition_id) REFERENCES period_definitions(id)
);

CREATE TABLE IF NOT EXISTS total_vm (
    period_record_id UNSIGNED INT NOT NULL,
    vm TEXT NOT NULL,
    total UNSIGNED INT NOT NULL,
    PRIMARY KEY (period_record_id, vm)
    FOREIGN KEY (period_record_id) REFERENCES period_records(id),
    FOREIGN KEY (vm) REFERENCES vms(name)
);

CREATE TABLE IF NOT EXISTS total_group (
    period_record_id UNSIGNED INT NOT NULL,
    group_id UNSIGNED INT NOT NULL,
    total UNSIGNED INT,
    PRIMARY KEY (period_record_id, group_id),
    FOREIGN KEY (period_record_id) REFERENCES period_records(id),
    FOREIGN KEY (group_id) REFERENCES groups(id)
);
