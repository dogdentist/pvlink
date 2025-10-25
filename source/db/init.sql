
CREATE TABLE IF NOT EXISTS tb_link (
    tb_link_id       BIGSERIAL PRIMARY KEY,
    tb_link_short    TEXT NOT NULL,
    tb_link_long     TEXT NOT NULL,
    tb_link_creation TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tb_link_expires  TIMESTAMPTZ,
    tb_link_clicks   BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_tb_link_short ON tb_link (tb_link_short);
CREATE INDEX IF NOT EXISTS idx_tb_link_expires ON tb_link (tb_link_expires);

CREATE TABLE IF NOT EXISTS tb_link_clicks (
    tb_link_clicks_link_id      BIGINT REFERENCES tb_link(tb_link_id) ON DELETE CASCADE,
    tb_link_clicks_country      TEXT,
    tb_link_clicks_clicks       BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (tb_link_clicks_link_id, tb_link_clicks_country)
);

CREATE INDEX IF NOT EXISTS idx_tb_link_clicks_link_country ON tb_link_clicks (tb_link_clicks_link_id, tb_link_clicks_country);

CREATE TABLE IF NOT EXISTS tb_user (
    tb_user_id       BIGSERIAL PRIMARY KEY,
    tb_user_username TEXT NOT NULL,
    tb_user_password TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tb_user_username ON tb_user (tb_user_username);

-- demo data

-- the passwor is admin
INSERT INTO tb_user (
    tb_user_username,
    tb_user_password
) VALUES (
  'admin',
  '$2a$10$jZAKbRQUn8kVKE8U62SOweTEfLFufgN.cfJ450wJ1PAvTtXDfzB7O'
);

INSERT INTO tb_link (
    tb_link_short,
    tb_link_long
) VALUES (
  'google',
  'https://google.com/'
);
