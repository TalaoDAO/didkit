#!/bin/sh
set -ex
cd "$(dirname "$0")"

test -s r1.jwk || { didkit generate-secp256k1-key > r1.jwk; }
test -s u1.jwk || { didkit generate-secp256k1-key > u1.jwk; }
test -s v1.jwk || { didkit generate-secp256k1-key > v1.jwk; }

did=$(didkit did-create ion -r r1.jwk -v s1.jwk -u u1.jwk)

test -s r2.jwk || { didkit generate-secp256k1-key > r2.jwk; }
test -s u2.jwk || { didkit generate-secp256k1-key > u2.jwk; }
test -s v2.jwk || { didkit generate-secp256k1-key > v2.jwk; }

#didkit did-recover "$did" -R r1.jwk -r r2.jwk -v v2.jwk -u u2.jwk || true

#didkit did-update "$did" -R r1.jwk -r r2.jwk -v v2.jwk -u u2.jwk || true

svc="$did#service-1"
didkit did-update -U u2.jwk -u u1.jwk set-service "$svc" -e http://localhost/ -t ExampleType || true
didkit did-update -U u2.jwk -u u1.jwk set-service "$svc" -e {} -t ExampleType || true
didkit did-update -U u2.jwk -u u1.jwk set-verification-method "$did#asdf" -t asdf --authentication -k v2.jwk

#didkit did-deactivate "$did" -k r2.jwk || true
