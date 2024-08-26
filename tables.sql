CREATE TABLE GenerationData (
    data_fecha_acualizado INTEGER PRIMARY KEY
);

CREATE TABLE FuelCost (
    data_fecha_acualizado INTEGER NOT NULL,
    place TEXT NOT NULL,
    value INTEGER NOT NULL,
    FOREIGN KEY (data_fecha_acualizado) REFERENCES GenerationData(data_fecha_acualizado)
);

CREATE TABLE ByFuel (
    data_fecha_acualizado INTEGER NOT NULL,
    fuel TEXT NOT NULL,
    value INTEGER NOT NULL,
    FOREIGN KEY (data_fecha_acualizado) REFERENCES GenerationData(data_fecha_acualizado)
);

CREATE TABLE Metrics (
    data_fecha_acualizado INTEGER NOT NULL,
    "index" TEXT NOT NULL,
    "desc" TEXT NOT NULL,
    value TEXT NOT NULL, -- Use TEXT to accommodate different types of values (number or date)
    FOREIGN KEY (data_fecha_acualizado) REFERENCES GenerationData(data_fecha_acualizado)
);

CREATE TABLE LoadPerSite (
    data_fecha_acualizado INTEGER NOT NULL,
    "index" TEXT NOT NULL,
    "type_field" TEXT NOT NULL,
    "desc" TEXT NOT NULL,
    site_total INTEGER NOT NULL,
    PRIMARY KEY (data_fecha_acualizado, "index"),
    FOREIGN KEY (data_fecha_acualizado) REFERENCES GenerationData(data_fecha_acualizado)
);

CREATE TABLE Units (
    data_fecha_acualizado INTEGER NOT NULL,
    load_per_site_index TEXT NOT NULL,
    "index" TEXT NOT NULL,
    "unit" TEXT NOT NULL,
    mw INTEGER NOT NULL,
    mvar TEXT NOT NULL,
    cost REAL NOT NULL,
    parent_id TEXT NOT NULL,
    FOREIGN KEY (data_fecha_acualizado, load_per_site_index) REFERENCES LoadPerSite(data_fecha_acualizado, "index")
);

-- LUMA data
CREATE TABLE RegionData (
    time INTEGER NOT NULL,
    name TEXT NOT NULL,
    total_clients INTEGER NOT NULL,
    total_clients_without_service INTEGER NOT NULL,
    total_clients_with_service INTEGER NOT NULL,
    total_clients_affected_by_planned_outage INTEGER NOT NULL,
    percentage_clients_without_service REAL NOT NULL,
    percentage_clients_with_service REAL NOT NULL,
    PRIMARY KEY (time, name)
);

CREATE TABLE Totals (
    time INTEGER PRIMARY KEY,
    total_clients_without_service INTEGER NOT NULL,
    total_clients INTEGER NOT NULL,
    total_clients_with_service INTEGER NOT NULL,
    total_percentage_without_service REAL NOT NULL,
    total_clients_affected_by_planned_outage INTEGER NOT NULL,
    total_percentage_with_service REAL NOT NULL
);