#!/bin/sh

DB_NAME="pvlink"
DB_USERNAME="pvlink"

SHORT_LINK="$1"
LONG_LINK="$2"
EXPIRES="$3"

NOW=$(date '+%Y-%m-%d %H:%M:%S')

if [ -z "$EXPIRES" ]; then
  SQL="INSERT INTO tb_link (\
        tb_link_short, \
        tb_link_long, \
        tb_link_creation\
    ) VALUES (
    '$SHORT_LINK',
    '$LONG_LINK',
    '$NOW'
  );"
else
  SQL="INSERT INTO tb_link (\
        tb_link_short, \
        tb_link_long, \
        tb_link_creation, \
        tb_link_expires\
    ) VALUES (
    '$SHORT_LINK',
    '$LONG_LINK',
    '$NOW',
    '$EXPIRES'
  );"
fi

psql -U $DB_USERNAME -d $DB_NAME -c "$SQL"
