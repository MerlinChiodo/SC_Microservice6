FROM rust:latest
WORKDIR /usr/src/app
RUN git clone https://github.com/SmartCityProjectGroup/SC_Microservice6 .
RUN cargo install mzoon --git https://github.com/MoonZoon/MoonZoon --rev 15cb619faca5f78a47e08f4af4bfa595f0eb64b1 --root cargo_install_root --locked
RUN mv cargo_install_root/bin/mzoon mzoon

CMD ./mzoon start