#Erstellt eine virtuelle rust maschine
FROM rust:1.40 as builder
#Ändert das Workdir auf myapp
WORKDIR /usr/src/myapp
#Kopiert die Dateien in die Workdir
COPY . .
#installiert deps
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies 
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
#startet den Rust Server
CMD ["myapp"]
