FROM busybox:stable-glibc as builder
RUN echo "nobody:x:65534:65534:Nobody:/:" > /etc/nobody

FROM scratch
WORKDIR /
COPY --from=builder /etc/nobody /etc/passwd
USER nobody
COPY context-generator /context-generator
ENTRYPOINT ["/context-generator"]
