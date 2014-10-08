CREATE SEQUENCE services_id_seq;
CREATE TABLE services (
  id         integer not null default nextval('services_id_seq'),
  name       varchar(256) not null unique check (length(name) > 0),
  created_at timestamp without time zone not null
    default (now() at time zone 'utc'),

  primary key (id)
);
