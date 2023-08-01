FROM ubuntu:22.04
LABEL authors="bppleman"

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y \
    curl \
    ca-certificates \
    git \
    gcc \
    libclang-dev \
    libgtk-3-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile complete

COPY entry.sh /usr/local/bin/entry.sh

COPY mgit.bundle /root/mgit.bundle

RUN chmod +x /usr/local/bin/entry.sh

CMD ["/usr/local/bin/entry.sh"]
