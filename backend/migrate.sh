#!/bin/bash

echo "Running migration: Adding display_order to custom_pages table..."
echo "Please enter your MySQL credentials:"

read -p "Username (default: root): " db_user
db_user=${db_user:-root}

read -sp "Password: " db_pass
echo ""

read -p "Database name: " db_name

# Run the migration
mysql -u "$db_user" -p"$db_pass" "$db_name" < sql/updates/add_display_order_to_custom_pages.sql

echo "Migration completed." 