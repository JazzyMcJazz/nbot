
add_conf() {
    # 1. file name
    # 2. container name
    # 3. port
    # 4..n. domains

    # Ensure the NAME and VALUE variables are passed
    if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ] || [ -z "$4" ]; then
        >&2 echo "Usage: add_conf FILE_NAME CONTAINER_NAME PORT DOMAIN [...MORE_DOMAINS]"
        exit 1
    fi

    FILE_NAME="$1"
    CONTAINER_NAME="$2"
    PORT="$3"
    DOMAIN="$4"
    DESTINATION="/etc/nginx/conf.d/${FILE_NAME}.conf"

    # POSIX compliant way to shift the arguments
    shift 4
    DOMAINS="${DOMAIN} "
    while [ "$#" -gt 0 ]; do
        DOMAINS="$DOMAINS$1 "
        shift
    done

    # Check if the template file exists
    if [[ ! -f "/template.conf" ]]; then
        >&2 echo "The template file /template.conf does not exist."
        exit 1
    fi

    # Copying the template file to the destination
    cp /template.conf /temp.conf

    # Replacing the placeholders with the provided values
    sed -i "s/{{port}}/${PORT}/g" /temp.conf
    sed -i "s/{{domain}}/${DOMAIN}/g" /temp.conf
    sed -i "s/{{domains}}/${DOMAINS}/g" /temp.conf
    sed -i "s/{{container_name}}/${CONTAINER_NAME}/g" /temp.conf

    # Move the file to the destination
    mv /temp.conf ${DESTINATION}

    echo "File copied and value replaced successfully."
    # The Nginx service reloads automatically when the folder is modified due to inotifywait
}

remove_conf() {
    # 1. file name

    # Ensure the NAME and VALUE variables are passed
    if [ -z "$1" ]; then
        >&2 echo "Usage: remove_conf FILE_NAME"
        exit 1
    fi

    FILE_NAME="$1"
    DESTINATION="/etc/nginx/conf.d/${FILE_NAME}.conf"

    # Check if the file exists
    EXISTS=$(ls /etc/nginx/conf.d | grep ${FILE_NAME}.conf)
    if [ -z "$EXISTS" ]; then
        >&2 echo "The file ${FILE_NAME}.conf does not exist."
        exit 1
    fi

    # Remove the file
    rm ${DESTINATION}

    echo "File removed successfully."
    # The Nginx service reloads automatically when the folder is modified due to inotifywait
}

generate_certs_certbot() {
    set -e

    # 1. email
    # 2..n. domains

    # Ensure the NAME and VALUE variables are passed
    if [ -z "$1" ] || [ -z "$2" ]; then
        >&2 echo "Usage: generate_certificate EMAIL ...DOMAINS"
        exit 1
    fi

    EMAIL="$1"

    shift 1
    DOMAINS=""
    while [ "$#" -gt 0 ]; do
        DOMAINS="$DOMAINS$1 "
        shift
    done

    # Generate the SSL certificate using Certbot
    certbot certonly --webroot -w /usr/share/nginx/html "${DOMAINS[@]}" --email "${EMAIL}" --agree-tos --non-interactive

    # Dry run
    # certbot certonly --dry-run --webroot -w ~/nginx_test "${D_FLAGS[@]}" --email "${EMAIL}" --agree-tos --non-interactive

    f_call=$1; shift; $f_call "$@"
}

generate_certs_openssl() {
    # 1. domain
    
    if [ -z "$1" ]; then
        >&2 echo "Usage: generate_certificate_openssl DOMAIN"
        exit 1
    fi

    DOMAIN="$1"

    mkdir -p /etc/letsencrypt/live/${DOMAIN}

    OUT="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"
    KEYOUT="/etc/letsencrypt/live/${DOMAIN}/privkey.pem"
    SUBJECT="/C=''/ST=''/L=''/O=''/OU=''/CN=${DOMAIN}"

    openssl x509 -checkend 86400 -noout -in ${OUT}
    EXIST=$?
    if [ $EXIST -eq 0 ]; then
        echo "Certificate is still valid"
        exit 0
    fi

    # Generate the SSL certificate using OpenSSL
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout ${KEYOUT} -out ${OUT} -subj "${SUBJECT}"
}

remove_certs() {
    set -e

    # 1. domain

    if [ -z "$1" ]; then
        >&2 echo "Usage: remove_certificate DOMAIN"
        exit 1
    fi

    DOMAIN="$1"
    DESTINATION="/etc/letsencrypt/live/${DOMAIN}"

    # Check if the file exists
    EXISTS=$(ls ${DESTINATION}) 
    if [ -z "$EXISTS" ]; then
        >&2 echo "The certificate for ${DOMAIN} does not exist."
        exit 1
    fi

    # Remove the file
    rm -rf ${DESTINATION}

    echo "Certificate removed successfully."
}

f_call=$1; shift; $f_call "$@"