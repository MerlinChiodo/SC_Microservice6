FROM rust:latest
#Test
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE2_URL}

ARG AMQP_ADDR
ENV AMQP_ADDR=${AMQP_ADDR}


WORKDIR /usr/src/app
RUN rustup default nightly
RUN apt-get install default-libmysqlclient-dev
RUN git clone https://github.com/SmartCityProjectGroup/SC_Microservice6 .

WORKDIR /usr/src/app/backend
RUN diesel migration run
WORKDIR /usr/src/app

RUN cargo install mzoon --git https://github.com/MoonZoon/MoonZoon --rev 15cb619faca5f78a47e08f4af4bfa595f0eb64b1 --root cargo_install_root --locked
RUN mv cargo_install_root/bin/mzoon mzoon
RUN cargo install diesel_cli --no-default-features --features mysql


CMD ./mzoon start
