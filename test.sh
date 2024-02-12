cargo run -- run -n jenkins \
    --app dind \
    --image docker:dind \
    --privileged true \
    --env DOCKER_TLS_CERTDIR=/certs \
    --volume jenkins-docker-certs:/certs/client \
    --volume jenkins-data:/var/jenkins_home \
    --app jenkins \
    --image dind-jenkins:latest \
    --env DOCKER_HOST=tcp://docker:2376 \
    --env DOCKER_CERT_PATH=/certs/client \
    --env DOCKER_TLS_VERIFY=1 \
    --port 8080 \
    --volume jenkins_home:/var/jenkins_home \
    --volume jenkins-docker-certs:/certs/client:ro
    # --domain jenkins.treeleaf.dev \
    # --email "LETSENCRYPT_HOST=ci.treeleaf.dev" \