#!/bin/sh

if [ ! -e "pvlink-ca.crt" ] || [ ! -e "pvlink-ca.key" ]; then
    cat > pvlink.cnf << EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = pvlink.localhost
O = pvlink

[v3_req]
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
basicConstraints = CA:FALSE

[alt_names]
DNS.1 = pvlink.localhost
DNS.2 = admin.pvlink.localhost
DNS.3 = traefik.admin.pvlink.localhost
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

    cat > ca.cnf << EOF
[req]
distinguished_name = req_distinguished_name
prompt = no

[req_distinguished_name]
CN = pvlink CA
O = pvlink

[v3_ca]
keyUsage = digitalSignature, keyCertSign, cRLSign
basicConstraints = CA:TRUE
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
EOF

    openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes \
      -keyout pvlink-ca.key -out pvlink-ca.crt \
      -subj "/CN=pvlink CA/O=pvlink" \
      -extensions v3_ca -config ca.cnf

    openssl req -newkey rsa:2048 -nodes -keyout pvlink.key \
        -out pvlink.csr \
        -config pvlink.cnf

    openssl x509 -req -in pvlink.csr -CA pvlink-ca.crt -CAkey pvlink-ca.key -CAcreateserial \
        -out pvlink.crt -days 365 -sha256 \
        -extfile pvlink.cnf -extensions v3_req

    echo "CA file has been generated, please trust it in your system"

    rm -f ca.cnf pvlink.cnf pvlink.csr
fi

if [ ! -e "env/certs/pvlink.crt" ]; then
    cp pvlink.crt env/certs/pvlink.crt
fi

if [ ! -e "env/certs/pvlink.key" ]; then
    cp pvlink.key env/certs/pvlink.key
fi
