-- See setup.sh for how this is invoked

CREATE ROLE wikijump
WITH
    LOGIN
    ENCRYPTED PASSWORD 'NoNumbersInEmails100';

CREATE DATABASE wikijump
WITH
    OWNER = wikijump
    ENCODING = 'utf-8';
