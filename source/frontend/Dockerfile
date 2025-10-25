FROM alpine:3.22.2

ARG UID
ARG GID

RUN apk add curl --no-cache
RUN apk add bash --no-cache
RUN apk add libstdc++ --no-cache
RUN apk add libgcc --no-cache

RUN curl -fsSL https://bun.sh/install | bash && \
    mv /root/.bun/bin/bun /usr/local/bin/bun

RUN mkdir /var/log/pvlink
RUN addgroup -g ${GID} developer
RUN adduser -u ${UID} -D -G developer developer
RUN chown developer:developer /var/log/pvlink
USER developer
ENV PATH=/usr/bin/:$PATH
RUN mkdir /home/developer/project
WORKDIR /home/developer/project

CMD /bin/sh -c "bun install && bun run dev >> build.txt"
