CREATE TABLE IF NOT EXISTS meals (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    last_planned_at TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ingredients (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS meal_ingredients (
    meal_id       INTEGER NOT NULL REFERENCES meals(id) ON DELETE CASCADE,
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    quantity      TEXT,
    PRIMARY KEY (meal_id, ingredient_id)
);

CREATE TABLE IF NOT EXISTS week_plans (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    year        INTEGER NOT NULL,
    week_number INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    UNIQUE (year, week_number)
);

CREATE TABLE IF NOT EXISTS plan_meals (
    plan_id INTEGER NOT NULL REFERENCES week_plans(id) ON DELETE CASCADE,
    meal_id INTEGER NOT NULL REFERENCES meals(id) ON DELETE CASCADE,
    UNIQUE (plan_id, meal_id)
);
