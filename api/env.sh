#!/bin/bash

export PORT=3333
export ENV=development
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/ferris_key

export ADMIN_PASSWORD=admin
export ADMIN_USERNAME=admin
export ADMIN_EMAIL=admin@example.com

export ALLOWED_ORIGINS=http://localhost:3000,http://localhost:3001
export PORTAL_URL=http://localhost:5555
