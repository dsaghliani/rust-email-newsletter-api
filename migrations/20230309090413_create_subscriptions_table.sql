CREATE TABLE subscriptions(
    id uuid NOT NULL PRIMARY KEY,
    email varchar(255) COLLATE "case_insensitive" NOT NULL UNIQUE,
    name varchar(255) COLLATE "case_insensitive" NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT trigger_updated_at('subscriptions');
