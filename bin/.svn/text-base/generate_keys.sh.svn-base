#!/bin/bash

openssl genrsa -out files/key.pem
openssl rsa -in files/key.pem -pubout -out files/public.pem
openssl rsa -in files/key.pem -noout -modulus | sed 's/Modulus=//' > files/modulus.pem
