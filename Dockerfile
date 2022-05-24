FROM debian:buster-slim
COPY . .
RUN apt-get update && apt-get install -y
#startet den Rust Server
CMD ["cargo run --bin SmartCity_Auth"]
