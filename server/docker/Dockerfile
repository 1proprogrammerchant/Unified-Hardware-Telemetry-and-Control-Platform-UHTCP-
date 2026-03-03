FROM golang:1.20-alpine AS build
RUN apk add --no-cache git ca-certificates
WORKDIR /src
COPY . /src
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags='-s -w' -o /out/uhtcp-server ./cmd/server

FROM alpine:3.18
RUN apk add --no-cache ca-certificates
COPY --from=build /out/uhtcp-server /usr/local/bin/uhtcp-server
COPY config/default.yaml /etc/uhtcp/default.yaml
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/uhtcp-server", "--config", "/etc/uhtcp/default.yaml"]
