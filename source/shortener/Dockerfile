FROM alpine:3.22.2

ARG UID
ARG GID

RUN apk add rust --no-cache
RUN apk add cargo --no-cache
RUN apk add rustfmt --no-cache
RUN apk add rust-src --no-cache
RUN apk add yaml-dev --no-cache
RUN apk add openssl-dev --no-cache

RUN mkdir /var/log/pvlink
RUN addgroup -g ${GID} developer
RUN adduser -u ${UID} -D -G developer developer
RUN chown developer:developer /var/log/pvlink
USER developer
ENV PATH=/usr/bin/:$PATH
RUN mkdir /home/developer/project
WORKDIR /home/developer/project

CMD /bin/sh -c "cargo run > build.txt"
