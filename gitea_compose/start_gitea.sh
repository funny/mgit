#!/usr/bin/env zsh

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
set -e
source $SCRIPT_DIR/env.sh
set +e

echo $GITEA_USERNAME $GITEA_PASSWORD
# 管理员用户名
GITEA_USERNAME=${GITEA_USERNAME:-"mgit"}
# 管理员密码
GITEA_PASSWORD=${GITEA_PASSWORD:-"mgit"}
# 管理员邮箱
EMAIL=${EMAIL:-"mgit@xmsofunny.com"}

# 以下内容不要修改

BASIC_AUTH=$(echo -n "${GITEA_USERNAME}:${GITEA_PASSWORD}" | base64)

docker compose -f $SCRIPT_DIR/docker-compose.yml -p mgit_test down

docker compose -f $SCRIPT_DIR/docker-compose.yml -p mgit_test up -d

wait_for_service() {
    local HOST=$1
    local PORT=$2
    local TIMEOUT=30  # 设置超时时间
    local INTERVAL=1  # 设置每次检查的间隔

    local SUCCESS_MSG=$3
    local FAILURE_MSG=$4
    local WAITING_MSG=$5

    while [[ $TIMEOUT -gt 0 ]]; do
        docker exec gitea nc -z $HOST $PORT
        local RESULT=$?

        if [[ $RESULT -eq 0 ]]; then
            echo "${SUCCESS_MSG}"
            return 0
        else
            echo "${WAITING_MSG}"
            sleep $INTERVAL
            TIMEOUT=$((TIMEOUT - INTERVAL))
        fi
    done

    # 如果超时则打印错误信息并返回非0
    echo "${FAILURE_MSG}"
    return 1
}

# 等待gitea web
HOST=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' gitea)
wait_for_service "${HOST}" 3000 "Gitea web is up!" "Timed out waiting for Gitea web to be accessible!" "Waiting for Gitea web to be accessible..."
if [[ $? -ne 0 ]]; then
    echo "Gitea web is timeout!"
    exit 1
fi

# 等待postgres
HOST=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' postgres)
wait_for_service "${HOST}" 5432 "Postgres is up!" "Timed out waiting for Postgres to be accessible!" "Waiting for Postgres to be accessible..."

if [[ $? -ne 0 ]]; then
    echo "Postgres is timeout!"
    exit 1
fi

create_admin() {
    local CONTAINER_NAME="gitea"
    # 创建管理员用户，并获取输出
    OUTPUT=$(docker exec "${CONTAINER_NAME}" gitea admin user create --admin --username "${GITEA_USERNAME}" --password "${GITEA_PASSWORD}" --email "${EMAIL}")

    echo "${OUTPUT}"
}

# 创建管理员用户，并获取token
create_admin

function create_token() {
    local OUTPUT=$(curl -sX 'POST' \
    "http://localhost:3000/api/v1/users/${GITEA_USERNAME}/tokens" \
    -H 'accept: application/json' \
    -H "authorization: Basic ${BASIC_AUTH}" \
    -H 'Content-Type: application/json' \
    -d '{
        "name": "admin-token",
        "scopes": [
            "write:activitypub",
            "write:admin",
            "write:issue",
            "write:misc",
            "write:notification",
            "write:organization",
            "write:package",
            "write:repository",
            "write:user"
        ]
    }')
    echo -n $OUTPUT | jq -r ".sha1"
}

create_token

function migrate() {
    local REPO_ADDR=$1
    local REPO_NAME=$2
    local REPO_IS_LOCAL=$3

    echo "migrate ${REPO_NAME} ${REPO_ADDR} ${REPO_IS_LOCAL}"

    if [[ $REPO_IS_LOCAL == "true" ]]; then
        local SOURCE_ADDR="${REPO_ADDR}"
        REPO_ADDR="/var/lib/gitea/git/repos/${REPO_NAME}"
        docker exec -it gitea mkdir -p "/var/lib/gitea/git/repos"
        docker cp "${SOURCE_ADDR}" gitea:"${REPO_ADDR}"
    fi

    curl -X 'POST' \
    'http://localhost:3000/api/v1/repos/migrate' \
    -H 'accept: application/json' \
    -H "authorization: Basic ${BASIC_AUTH}" \
    -H 'Content-Type: application/json' \
    -d '{
        "clone_addr": "'"${REPO_ADDR}"'",
        "repo_name": "'"${REPO_NAME}"'",
        "issues": false,
        "labels": false,
        "milestones": false,
        "mirror": false,
        "private": false,
        "pull_requests": false,
        "releases": true,
        "service": "git",
        "wiki": false
    }'
}

for INDEX in {1..${#REPOS_ADDR[@]}};
do
    migrate ${REPOS_ADDR[INDEX]} ${REPOS_NAME[INDEX]} ${REPOS_IS_LOCAL[INDEX]}
done
