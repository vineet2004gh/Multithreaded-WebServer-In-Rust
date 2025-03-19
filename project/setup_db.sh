#!/bin/bash

echo "Setting up PostgreSQL database for the web server..."

# Run migrations
echo "Running database migrations..."
diesel migration run

echo "Database setup completed!"
