#!/bin/bash


sudo rm -rf /var/www/html/*
bun run build
sudo mv dist/* /var/www/html/

echo "All Done!"
